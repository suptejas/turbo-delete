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
use jwalk::DirEntry;
use owo_colors::{AnsiColors, OwoColorize};
use rayon::iter::{IntoParallelRefIterator, ParallelBridge, ParallelIterator};
use rusty_pool::ThreadPool;
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    time::Instant,
};

// change a file to be writable
pub fn set_writable(path: &Path) {
    let mut perms = std::fs::metadata(path).unwrap().permissions();

    perms.set_readonly(false);

    std::fs::set_permissions(path, perms).unwrap();
}

pub fn set_folder_writable(path: &Path) {
    // get complete list of folders
    let entries: Vec<DirEntry<((), ())>> = jwalk::WalkDir::new(&path)
        .follow_links(true)
        .skip_hidden(false)
        .into_iter()
        .filter(|v| {
            v.as_ref()
                .unwrap_or_else(|err| {
                    eprintln!(
                        "{} {}",
                        " ERROR ".on_color(AnsiColors::BrightRed).black(),
                        err
                    );
                    std::process::exit(1);
                })
                .path()
                .is_file()
        })
        .map(|v| {
            v.unwrap_or_else(|err| {
                eprintln!(
                    "{} {}",
                    " ERROR ".on_color(AnsiColors::BrightRed).black(),
                    err
                );
                std::process::exit(1);
            })
        })
        .collect::<Vec<DirEntry<((), ())>>>();

    entries.par_iter().for_each(|entry| {
        set_writable(&entry.path());
    });
}

fn main() {
    let start = Instant::now();

    let args = std::env::args().collect::<Vec<String>>();

    let mut file_path: String = args
        .get(1)
        .unwrap_or_else(|| {
            eprintln!(
                "{} {}\n\n{}:\n{} {}",
                " ERROR ".on_color(AnsiColors::BrightRed).black(),
                "Please provide a folder path.".bright_yellow(),
                "Examples".underline(),
                "turbodelete".bright_cyan(),
                "./node_modules/".bright_black(),
            );
            std::process::exit(1);
        })
        .to_string();

    // different methods to test out...
    // 1. only delete all folders using rm_dir_all
    // 2. delete folders and files using rm_file

    let mut tree: BTreeMap<u64, Vec<PathBuf>> = BTreeMap::new();

    if file_path.ends_with('"') {
        file_path.pop();
    }

    let path = PathBuf::from(&file_path);

    // get complete list of folders
    let entries: Vec<DirEntry<((), ())>> = jwalk::WalkDir::new(&path)
        .follow_links(true)
        .skip_hidden(false)
        .into_iter()
        .par_bridge()
        .filter(|v| v.as_ref().unwrap().path().is_dir())
        .map(|v| v.unwrap())
        .collect();

    let bar = ProgressBar::new(entries.len() as u64);

    for entry in entries {
        tree.entry(entry.depth as u64)
            .or_insert_with(Vec::new)
            .push(entry.path());
    }

    let pool = ThreadPool::default();

    let mut handles = vec![];

    for (_, entries) in tree.into_iter().rev() {
        let bar = bar.clone();

        handles.push(pool.evaluate(move || {
            entries.par_iter().for_each(|entry| {
                let _ = std::fs::remove_dir_all(entry);

                bar.inc(1);
            });
        }));
    }

    for handle in handles {
        handle.await_complete();
    }

    set_folder_writable(&path);

    std::fs::remove_dir_all(path).unwrap_or_else(|err| {
        eprintln!(
            "{} {}",
            " ERROR ".on_color(AnsiColors::BrightRed).black(),
            err
        );
        std::process::exit(1);
    });

    bar.println(format!("{}", start.elapsed().as_secs_f32()));
}
