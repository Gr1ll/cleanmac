use std::fs;
use std::path::PathBuf;

use bytesize::ByteSize;
use dialoguer::Confirm;
use indicatif::{ProgressBar, ProgressStyle};

use crate::display;

struct CacheTarget {
    name: &'static str,
    path: PathBuf,
}

fn home() -> PathBuf {
    PathBuf::from(std::env::var("HOME").expect("HOME environment variable not set"))
}

fn targets() -> Vec<CacheTarget> {
    let h = home();
    vec![
        CacheTarget { name: "~/Library/Caches", path: h.join("Library/Caches") },
        CacheTarget { name: "Xcode DerivedData", path: h.join("Library/Developer/Xcode/DerivedData") },
        CacheTarget { name: "CoreSimulator Caches", path: h.join("Library/Developer/CoreSimulator/Caches") },
        CacheTarget { name: "~/.npm", path: h.join(".npm") },
        CacheTarget { name: "~/.cargo/registry", path: h.join(".cargo/registry") },
        CacheTarget { name: "~/.cargo/git", path: h.join(".cargo/git") },
        CacheTarget { name: "Homebrew Caches", path: h.join("Library/Caches/Homebrew") },
    ]
}

fn dir_size(path: &PathBuf) -> u64 {
    walkdir::WalkDir::new(path)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter_map(|e| e.metadata().ok())
        .filter(|m| m.is_file())
        .map(|m| m.len())
        .sum()
}

fn delete_contents(path: &PathBuf, label: &str) {
    let entries = match fs::read_dir(path) {
        Ok(e) => e,
        Err(e) => { eprintln!("  Failed to read {}: {}", label, e); return; }
    };
    let mut deleted = 0u64;
    let mut failed = 0u64;
    for entry in entries.filter_map(|e| e.ok()) {
        let p = entry.path();
        let result = if p.is_dir() { fs::remove_dir_all(&p) } else { fs::remove_file(&p) };
        match result {
            Ok(_) => deleted += 1,
            Err(_) => failed += 1,
        }
    }
    if failed > 0 {
        println!("  Cleared {} ({} items skipped — in use or protected)", label, failed);
    } else {
        println!("  Cleared {} ({} items)", label, deleted);
    }
}

pub fn run(all: bool, dry_run: bool) {
    let targets = targets();

    let pb = ProgressBar::new(targets.len() as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} Scanning [{bar:40.cyan/blue}] {pos}/{len}  {msg}")
            .unwrap()
            .progress_chars("=>-"),
    );

    let mut results: Vec<(&CacheTarget, u64)> = Vec::new();
    for target in &targets {
        pb.set_message(target.name);
        let size = if target.path.exists() { dir_size(&target.path) } else { 0 };
        results.push((target, size));
        pb.inc(1);
    }
    pb.finish_and_clear();

    display::print_cache_header();
    for (target, size) in &results {
        display::print_cache_row(target.name, *size, &target.path.display().to_string());
    }

    let total: u64 = results.iter().map(|(_, s)| s).sum();
    println!("{}", "─".repeat(90));
    println!("{:<26} {:>12}", "Total", ByteSize(total).display().si().to_string());
    println!();

    if dry_run {
        println!("Dry run — nothing deleted.");
        return;
    }

    let deletable: Vec<_> = results.iter().filter(|(t, s)| *s > 0 && t.path.exists()).collect();
    if deletable.is_empty() {
        println!("Nothing to delete.");
        return;
    }

    println!();
    for (target, size) in &deletable {
        let should_delete = if all {
            true
        } else {
            Confirm::new()
                .with_prompt(format!("Delete {} ({})?", target.name, ByteSize(*size).display().si()))
                .default(false)
                .interact()
                .unwrap_or(false)
        };

        if should_delete {
            delete_contents(&target.path, target.name);
        }
    }
}
