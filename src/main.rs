/*
  Copyright 2022 Tejas Ravishankar

  Licensed under the Apache License, Version 2.0 (the "License");
  you may not use this file except in compliance with the License.
  You may obtain a copy of the License at

      http://www.apache.org/licenses/LICENSE-2.0

  Unless required by applicable law or agreed to in writing, software
  distributed under the License is distributed on an "AS IS" BASIS,
  WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
  See the License for the specific language governing permissions and
  limitations under the License.
*/

use indicatif::ProgressBar;
use owo_colors::{AnsiColors, OwoColorize};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use std::{
    collections::BTreeMap,
    io::{self, Write},
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

fn print_error(msg: impl std::fmt::Display) {
    eprintln!(
        "{} {}",
        " ERROR ".on_color(AnsiColors::BrightRed).black(),
        msg
    );
}

fn print_usage() {
    eprintln!(
        "{}:\n  {} {}\n\n{}:\n  --contents  delete only the contents, keep the folder itself\n  --confirm   ask before deleting and keep the window open on errors",
        "Usage".underline(),
        "turbo-delete".bright_cyan(),
        "[--contents] [--confirm] <folder>".bright_black(),
        "Options".underline(),
    );
}

fn wait_for_enter() {
    print!("\nPress Enter to close...");
    let _ = io::stdout().flush();
    let _ = io::stdin().read_line(&mut String::new());
}

fn fail(msg: impl std::fmt::Display, pause: bool) -> ! {
    print_error(msg);
    if pause {
        wait_for_enter();
    }
    std::process::exit(1);
}

// clear the read-only attribute (only used on real files and folders, never on links)
fn set_writable(path: &Path) -> io::Result<()> {
    let mut perms = std::fs::symlink_metadata(path)?.permissions();

    perms.set_readonly(false);

    std::fs::set_permissions(path, perms)
}

fn retry_writable(path: &Path, remove: fn(&Path) -> io::Result<()>) -> io::Result<()> {
    match remove(path) {
        Err(err) if err.kind() == io::ErrorKind::PermissionDenied => {
            set_writable(path)?;
            remove(path)
        }
        result => result,
    }
}

// remove a file or a link. Links (symlinks and junctions) are removed themselves,
// their target is never touched.
fn remove_non_dir(path: &Path, is_link: bool) -> io::Result<()> {
    if is_link {
        // directory links need remove_dir, file links need remove_file
        return std::fs::remove_dir(path).or_else(|_| std::fs::remove_file(path));
    }

    retry_writable(path, |p| std::fs::remove_file(p))
}

// std::fs::remove_dir_all does not follow links, so it is safe as a fallback
fn remove_leftovers(path: &Path, contents_only: bool) -> io::Result<()> {
    if !contents_only {
        return std::fs::remove_dir_all(path);
    }

    let mut result = Ok(());

    for entry in std::fs::read_dir(path)? {
        let entry = entry?;
        let file_type = entry.file_type()?;

        let removed = if file_type.is_dir() {
            std::fs::remove_dir_all(entry.path())
        } else {
            remove_non_dir(&entry.path(), file_type.is_symlink())
        };

        if let Err(err) = removed {
            result = Err(err);
        }
    }

    result
}

fn is_done(path: &Path, contents_only: bool) -> bool {
    if contents_only {
        std::fs::read_dir(path)
            .map(|mut entries| entries.next().is_none())
            .unwrap_or(false)
    } else {
        std::fs::symlink_metadata(path).is_err()
    }
}

fn main() {
    let mut contents_only = false;
    let mut confirm = false;
    let mut target: Option<String> = None;

    for arg in std::env::args().skip(1) {
        match arg.as_str() {
            "--contents" => contents_only = true,
            "--confirm" => confirm = true,
            "-h" | "--help" => {
                print_usage();
                return;
            }
            _ if target.is_none() => target = Some(arg),
            _ => fail(format!("Unexpected argument: {}", arg), false),
        }
    }

    let mut file_path = target.unwrap_or_else(|| {
        print_error("Please provide a folder path.".bright_yellow());
        print_usage();
        std::process::exit(1);
    });

    // a trailing backslash before the closing quote escapes the quote ("C:\foo\")
    if file_path.ends_with('"') {
        file_path.pop();
    }

    // std::path::absolute does not resolve links, unlike canonicalize
    let path = std::path::absolute(&file_path)
        .unwrap_or_else(|err| fail(format!("{}: {}", file_path, err), confirm));

    let metadata = std::fs::symlink_metadata(&path)
        .unwrap_or_else(|err| fail(format!("{}: {}", path.display(), err), confirm));

    if path.parent().is_none() {
        fail(
            format!("Refusing to delete a drive root: {}", path.display()),
            confirm,
        );
    }

    if metadata.file_type().is_symlink() {
        if contents_only {
            fail(
                format!(
                    "{} is a link, refusing to empty its target",
                    path.display()
                ),
                confirm,
            );
        }

        // only the link is removed, not the folder it points to
        remove_non_dir(&path, true)
            .unwrap_or_else(|err| fail(format!("{}: {}", path.display(), err), confirm));
        println!("Removed link {}", path.display());
        return;
    }

    if !metadata.is_dir() {
        fail(format!("{} is not a folder", path.display()), confirm);
    }

    let start = Instant::now();

    // collect everything without following links, so junctions and symlinks
    // inside the folder are removed as links and their targets stay untouched
    let mut files: Vec<(PathBuf, bool)> = Vec::new();
    let mut dirs: BTreeMap<usize, Vec<PathBuf>> = BTreeMap::new();
    let mut walk_errors = 0;

    for entry in jwalk::WalkDir::new(&path)
        .follow_links(false)
        .skip_hidden(false)
    {
        match entry {
            Ok(entry) => {
                if entry.file_type.is_dir() {
                    if !(contents_only && entry.depth == 0) {
                        dirs.entry(entry.depth).or_default().push(entry.path());
                    }
                } else {
                    files.push((entry.path(), entry.file_type.is_symlink()));
                }
            }
            Err(err) => {
                walk_errors += 1;
                print_error(err);
            }
        }
    }

    let total = files.len() + dirs.values().map(Vec::len).sum::<usize>();

    if confirm {
        print!(
            "{} {} ({} items)? [y/N] ",
            if contents_only {
                "Permanently delete the contents of"
            } else {
                "Permanently delete"
            },
            path.display().bright_cyan(),
            total
        );
        let _ = io::stdout().flush();

        let mut answer = String::new();
        let _ = io::stdin().read_line(&mut answer);

        if !matches!(
            answer.trim().to_lowercase().as_str(),
            "y" | "yes" | "j" | "ja"
        ) {
            println!("Aborted.");
            return;
        }
    }

    let bar = ProgressBar::new(total as u64);

    // files and links first, then the folders from the deepest level up
    files.par_iter().for_each(|(file, is_link)| {
        let _ = remove_non_dir(file, *is_link);
        bar.inc(1);
    });

    for (_, level) in dirs.iter().rev() {
        level.par_iter().for_each(|dir| {
            let _ = retry_writable(dir, |p| std::fs::remove_dir(p));
            bar.inc(1);
        });
    }

    bar.finish_and_clear();

    // files that were still open (e.g. by a virus scanner) may vanish with a short delay
    let mut last_error = None;

    for attempt in 0..3 {
        if is_done(&path, contents_only) {
            break;
        }

        std::thread::sleep(Duration::from_millis(100 * attempt));

        if let Err(err) = remove_leftovers(&path, contents_only) {
            last_error = Some(err);
        }
    }

    if !is_done(&path, contents_only) {
        let remaining = jwalk::WalkDir::new(&path)
            .follow_links(false)
            .skip_hidden(false)
            .into_iter()
            .count();

        fail(
            format!(
                "{} items could not be deleted (in use or access denied){}",
                remaining,
                last_error.map(|err| format!(": {}", err)).unwrap_or_default()
            ),
            confirm,
        );
    }

    if walk_errors > 0 {
        println!(
            "{} entries could not be read while scanning, but were removed afterwards",
            walk_errors
        );
    }

    println!(
        "Deleted {} items in {:.2}s",
        total,
        start.elapsed().as_secs_f32()
    );
}
