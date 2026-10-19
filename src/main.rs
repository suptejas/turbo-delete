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
    fs::{File, OpenOptions, TryLockError},
    io::{self, Read, Seek, Write},
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
        "{}:\n  {} {}\n\n{}:\n  --contents  delete only the contents, keep the folders themselves\n  --confirm   ask before deleting and keep the window open on errors.\n              Calls started at the same time (e.g. by the Explorer context menu\n              for several selected items) are merged into one confirmation.",
        "Usage".underline(),
        "turbo-delete".bright_cyan(),
        "[--contents] [--confirm] <path>...".bright_black(),
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
fn remove_leftovers(target: &Target) -> io::Result<()> {
    if !target.is_dir {
        return remove_non_dir(&target.path, target.is_link);
    }

    if !target.contents_only {
        return std::fs::remove_dir_all(&target.path);
    }

    let mut result = Ok(());

    for entry in std::fs::read_dir(&target.path)? {
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

fn is_done(target: &Target) -> bool {
    if target.contents_only {
        std::fs::read_dir(&target.path)
            .map(|mut entries| entries.next().is_none())
            .unwrap_or(false)
    } else {
        std::fs::symlink_metadata(&target.path).is_err()
    }
}

struct Target {
    path: PathBuf,
    is_dir: bool,
    is_link: bool,
    contents_only: bool,
}

fn resolve_target(arg: &str, contents_only: bool) -> Result<Target, String> {
    let mut arg = arg.to_string();

    // a trailing backslash before the closing quote escapes the quote ("C:\foo\")
    if arg.ends_with('"') {
        arg.pop();
    }

    // std::path::absolute does not resolve links, unlike canonicalize
    let path = std::path::absolute(&arg).map_err(|err| format!("{}: {}", arg, err))?;

    let metadata =
        std::fs::symlink_metadata(&path).map_err(|err| format!("{}: {}", path.display(), err))?;

    let is_link = metadata.file_type().is_symlink();
    let is_dir = metadata.is_dir();

    if contents_only && is_link {
        return Err(format!(
            "{} is a link, refusing to empty its target",
            path.display()
        ));
    }

    if contents_only && !is_dir {
        return Err(format!("{} is not a folder", path.display()));
    }

    Ok(Target {
        path,
        is_dir,
        is_link,
        contents_only,
    })
}

fn open_lock_file(path: &Path) -> io::Result<File> {
    OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(path)
}

fn drain_queue(queue: &mut File) -> io::Result<Vec<String>> {
    let mut content = String::new();
    queue.seek(io::SeekFrom::Start(0))?;
    queue.read_to_string(&mut content)?;
    queue.set_len(0)?;

    Ok(content
        .lines()
        .filter(|line| !line.is_empty())
        .map(str::to_string)
        .collect())
}

// The Explorer context menu starts one process per selected item. The first one
// becomes the leader and collects the paths of the others, which hand over their
// path and exit, so there is only one confirmation for the whole selection.
//
// Every check of the leader lock happens while holding the queue lock, and the
// leader gives up its lock only after draining the queue under the same lock, so
// no handed over path can get lost.
//
// Returns None if the paths were handed over to another process.
fn merge_with_parallel_calls(own: Vec<String>) -> io::Result<Option<Vec<String>>> {
    let dir = std::env::temp_dir().join("turbo-delete");
    std::fs::create_dir_all(&dir)?;

    let queue_lock = open_lock_file(&dir.join("queue.lock"))?;
    let leader_lock = open_lock_file(&dir.join("leader.lock"))?;
    let mut queue = open_lock_file(&dir.join("queue.txt"))?;

    queue_lock.lock()?;

    match leader_lock.try_lock() {
        Ok(()) => {}
        Err(TryLockError::WouldBlock) => {
            queue.seek(io::SeekFrom::End(0))?;
            for arg in &own {
                writeln!(queue, "{}", arg)?;
            }
            queue.flush()?;
            queue_lock.unlock()?;
            return Ok(None);
        }
        Err(TryLockError::Error(err)) => return Err(err),
    }

    // leftovers of a crashed leader are not ours
    drain_queue(&mut queue)?;
    queue_lock.unlock()?;

    let mut targets = own;
    let mut quiet_rounds = 0;

    loop {
        std::thread::sleep(Duration::from_millis(150));

        queue_lock.lock()?;
        let handed_over = drain_queue(&mut queue)?;

        if handed_over.is_empty() {
            quiet_rounds += 1;
        } else {
            quiet_rounds = 0;
            targets.extend(handed_over);
        }

        if quiet_rounds >= 4 {
            leader_lock.unlock()?;
            queue_lock.unlock()?;
            return Ok(Some(targets));
        }

        queue_lock.unlock()?;
    }
}

fn main() {
    let mut contents_only = false;
    let mut confirm = false;
    let mut args: Vec<String> = Vec::new();

    for arg in std::env::args().skip(1) {
        match arg.as_str() {
            "--contents" => contents_only = true,
            "--confirm" => confirm = true,
            "-h" | "--help" => {
                print_usage();
                return;
            }
            _ => args.push(arg),
        }
    }

    if args.is_empty() {
        print_error("Please provide a path.".bright_yellow());
        print_usage();
        std::process::exit(1);
    }

    if confirm {
        // hand over absolute paths, the leader may run in another directory
        let absolute = args
            .iter()
            .map(|arg| {
                std::path::absolute(arg.trim_end_matches('"'))
                    .map(|path| path.display().to_string())
                    .unwrap_or_else(|_| arg.clone())
            })
            .collect();

        args = match merge_with_parallel_calls(absolute) {
            Ok(Some(merged)) => merged,
            Ok(None) => return,
            Err(err) => {
                print_error(format!("Could not merge parallel calls: {}", err));
                args
            }
        };
    }

    let mut targets: Vec<Target> = Vec::new();
    let mut skipped = 0;

    for arg in &args {
        match resolve_target(arg, contents_only) {
            Ok(target) => {
                if target.path.parent().is_none() {
                    fail(
                        format!("Refusing to delete a drive root: {}", target.path.display()),
                        confirm,
                    );
                }
                targets.push(target);
            }
            Err(err) => {
                skipped += 1;
                print_error(err);
            }
        }
    }

    targets.sort_by(|a, b| a.path.cmp(&b.path));
    targets.dedup_by(|a, b| a.path == b.path);

    if targets.is_empty() {
        fail("Nothing to delete", confirm);
    }

    let start = Instant::now();

    // collect everything without following links, so junctions and symlinks
    // inside the folders are removed as links and their targets stay untouched
    let mut files: Vec<(PathBuf, bool)> = Vec::new();
    let mut dirs: BTreeMap<usize, Vec<PathBuf>> = BTreeMap::new();
    let mut walk_errors = 0;

    for target in &targets {
        if !target.is_dir {
            files.push((target.path.clone(), target.is_link));
            continue;
        }

        for entry in jwalk::WalkDir::new(&target.path)
            .follow_links(false)
            .skip_hidden(false)
        {
            match entry {
                Ok(entry) => {
                    if entry.file_type.is_dir() {
                        if !(target.contents_only && entry.depth == 0) {
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
    }

    let total = files.len() + dirs.values().map(Vec::len).sum::<usize>();

    if confirm {
        println!(
            "{}",
            if contents_only {
                "Permanently delete the contents of:"
            } else {
                "Permanently delete:"
            }
        );

        for target in targets.iter().take(15) {
            println!("  {}", target.path.display().bright_cyan());
        }

        if targets.len() > 15 {
            println!("  ... and {} more", targets.len() - 15);
        }

        if skipped > 0 {
            println!("{} paths were skipped, see above", skipped);
        }

        print!("{} items in total. Continue? [y/N] ", total);
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
        if targets.iter().all(is_done) {
            break;
        }

        std::thread::sleep(Duration::from_millis(100 * attempt));

        for target in targets.iter().filter(|target| !is_done(target)) {
            if let Err(err) = remove_leftovers(target) {
                last_error = Some(err);
            }
        }
    }

    let unfinished: Vec<&Target> = targets.iter().filter(|target| !is_done(target)).collect();

    if !unfinished.is_empty() {
        let remaining: usize = unfinished
            .iter()
            .map(|target| {
                jwalk::WalkDir::new(&target.path)
                    .follow_links(false)
                    .skip_hidden(false)
                    .into_iter()
                    .count()
            })
            .sum();

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

    if skipped > 0 {
        fail(format!("{} paths were skipped", skipped), confirm);
    }
}
