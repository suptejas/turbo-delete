use jwalk::DirEntry;
use owo_colors::{AnsiColors, OwoColorize};
use rayon::iter::{ParallelBridge, ParallelIterator};
use std::{collections::BTreeMap, path::PathBuf, time::Instant};

#[tokio::main]
async fn main() {
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

    for entry in entries {
        if tree.contains_key(&(entry.depth as u64)) {
            tree.get_mut(&(entry.depth as u64))
                .unwrap()
                .push(entry.path());
        } else {
            tree.insert(entry.depth as u64, vec![entry.path()]);
        }
    }

    for (depth, entries) in tree.iter().rev() {
        println!("{depth}");
    }

    println!("{}", start.elapsed().as_secs_f32());

    // println!("{:#?}", tree.keys());

    // let mut workers = FuturesUnordered::new();

    // while !entries.is_empty() {
    // let chunk = entries.split_off(entries.len().saturating_sub(5));

    // // move chunk into your task
    // workers.push(tokio::task::spawn_blocking(
    //     move || -> Result<(), String> {
    //         for item in chunk.iter() {
    //             std::fs::remove_dir_all(item).unwrap();
    //         }
    //         Ok(())
    //     },
    // ));
    // }

    // while let Some(_) = workers.next().await {
    // println!("done");
    // }
    // }
}
