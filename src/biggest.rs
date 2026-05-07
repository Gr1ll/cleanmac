use std::collections::HashMap;
use std::path::PathBuf;

use indicatif::{ProgressBar, ProgressStyle};

use crate::display;

fn home() -> PathBuf {
    PathBuf::from(std::env::var("HOME").expect("HOME environment variable not set"))
}

fn expand(path: &str) -> PathBuf {
    if path == "~" {
        home()
    } else if let Some(rest) = path.strip_prefix("~/") {
        home().join(rest)
    } else {
        PathBuf::from(path)
    }
}

pub fn run(path: Option<String>, depth: Option<usize>) {
    let root = path.as_deref().map(expand).unwrap_or_else(home);

    eprintln!("Scanning {} …", root.display());

    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} {msg}")
            .unwrap(),
    );

    let mut dir_sizes: HashMap<PathBuf, u64> = HashMap::new();
    let mut file_sizes: Vec<(PathBuf, u64)> = Vec::new();
    let mut scanned: u64 = 0;

    let root_depth = root.components().count();

    for entry in walkdir::WalkDir::new(&root)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| {
            e.map_err(|_| ()).ok()
        })
    {
        let meta = match entry.metadata() {
            Ok(m) => m,
            Err(_) => continue,
        };

        scanned += 1;
        if scanned % 2000 == 0 {
            pb.set_message(format!("Scanned {} items…", scanned));
        }

        if !meta.is_file() {
            continue;
        }

        let size = meta.len();
        let file_path = entry.path().to_path_buf();
        file_sizes.push((file_path.clone(), size));

        let mut ancestor = file_path.parent();
        while let Some(parent) = ancestor {
            if !parent.starts_with(&root) {
                break;
            }
            *dir_sizes.entry(parent.to_path_buf()).or_insert(0) += size;
            if parent == root {
                break;
            }
            ancestor = parent.parent();
        }
    }

    pb.finish_and_clear();

    let max_display_depth = depth.unwrap_or(usize::MAX);

    let mut candidates: Vec<(PathBuf, u64, bool)> = Vec::new();

    for (dir_path, size) in &dir_sizes {
        if dir_path == &root {
            continue;
        }
        let entry_depth = dir_path.components().count().saturating_sub(root_depth);
        if entry_depth <= max_display_depth {
            candidates.push((dir_path.clone(), *size, true));
        }
    }

    for (file_path, size) in &file_sizes {
        let entry_depth = file_path.components().count().saturating_sub(root_depth);
        if entry_depth <= max_display_depth {
            candidates.push((file_path.clone(), *size, false));
        }
    }

    candidates.sort_by(|a, b| b.1.cmp(&a.1));
    candidates.dedup_by(|a, b| a.0.starts_with(&b.0) && b.2);

    let top: Vec<_> = candidates.into_iter().take(15).collect();

    if top.is_empty() {
        println!("No items found in {}", root.display());
        return;
    }

    display::print_biggest_header();
    for (i, (path, size, _)) in top.iter().enumerate() {
        display::print_biggest_row(i + 1, *size, &path.display().to_string());
    }
    println!();
}
