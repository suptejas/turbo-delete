use indicatif::ProgressBar;
use jwalk::DirEntry;
use owo_colors::{AnsiColors, OwoColorize};
use rayon::iter::{IntoParallelRefIterator, ParallelBridge, ParallelIterator};
use rusty_pool::ThreadPool;
use std::{collections::BTreeMap, path::PathBuf, time::Instant};

fn main() {
    let start = Instant::now();

    let args = std::env::args().collect::<Vec<String>>();

    let file_path = args.get(1).unwrap_or_else(|| {
        eprintln!(
            "{} {}\n\n{}:\n{} {}",
            " ERROR ".on_color(AnsiColors::BrightRed).black(),
            "Please provide a folder path.".bright_yellow(),
            "Examples".underline(),
            "turbodelete".bright_cyan(),
            "./node_modules/".bright_black(),
        );
        std::process::exit(1);
    });

    // different methods to test out...
    // 1. only delete all folders using rm_dir_all
    // 2. delete folders and files using rm_file

    let mut tree: BTreeMap<u64, Vec<PathBuf>> = BTreeMap::new();

    // get complete list of folders
    let entries: Vec<DirEntry<((), ())>> = jwalk::WalkDir::new(file_path)
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

    std::fs::remove_dir_all(args.get(1).unwrap()).unwrap_or_else(|err| {
        eprintln!("Could not delete {} - {err}", args.get(1).unwrap());
        std::process::exit(1);
    });

    bar.println(format!("{}", start.elapsed().as_secs_f32()));
}
