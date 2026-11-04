use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::display;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Source {
    AppStore,
    Manual,
}

impl Source {
    pub fn label(self) -> &'static str {
        match self {
            Source::AppStore => "App Store",
            Source::Manual => "Manual",
        }
    }
}

fn home() -> PathBuf {
    PathBuf::from(std::env::var("HOME").expect("HOME environment variable not set"))
}

fn brew_cask_apps() -> Option<HashSet<String>> {
    let output = Command::new("brew")
        .args(["info", "--json=v2", "--installed"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).ok()?;

    let mut apps = HashSet::new();
    for cask in json["casks"].as_array()? {
        let Some(artifacts) = cask["artifacts"].as_array() else {
            continue;
        };
        for artifact in artifacts {
            let Some(entries) = artifact["app"].as_array() else {
                continue;
            };
            let names = entries
                .iter()
                .filter_map(|e| e.as_str().or_else(|| e["target"].as_str()))
                .chain(artifact["target"].as_str());
            for name in names {
                if let Some(file) = Path::new(name).file_name().and_then(|n| n.to_str()) {
                    apps.insert(file.to_string());
                }
            }
        }
    }
    Some(apps)
}

fn is_app(path: &Path) -> bool {
    path.extension().is_some_and(|e| e == "app")
}

fn find_apps(dir: &Path) -> Vec<PathBuf> {
    let mut apps = Vec::new();
    let Ok(entries) = fs::read_dir(dir) else {
        return apps;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if is_app(&path) {
            apps.push(path);
        } else if path.is_dir() {
            if let Ok(sub) = fs::read_dir(&path) {
                apps.extend(sub.flatten().map(|e| e.path()).filter(|p| is_app(p)));
            }
        }
    }
    apps
}

fn bundle_id(app: &Path) -> Option<String> {
    let value = plist::Value::from_file(app.join("Contents/Info.plist")).ok()?;
    value
        .as_dictionary()?
        .get("CFBundleIdentifier")?
        .as_string()
        .map(str::to_string)
}

fn is_apple_app(app: &Path) -> bool {
    bundle_id(app).is_some_and(|id| id.starts_with("com.apple."))
}

fn is_app_store_app(app: &Path) -> bool {
    app.join("Contents/_MASReceipt/receipt").exists()
}

pub fn run() {
    let cask_apps = match brew_cask_apps() {
        Some(apps) => apps,
        None => {
            eprintln!(
                "Warning: could not query Homebrew; treating all apps as not installed via brew."
            );
            HashSet::new()
        }
    };

    let mut results: Vec<(Source, String, PathBuf)> = Vec::new();
    let mut brew_count = 0;

    for dir in [PathBuf::from("/Applications"), home().join("Applications")] {
        for app in find_apps(&dir) {
            let file_name = app.file_name().and_then(|n| n.to_str()).unwrap_or_default();
            if cask_apps.contains(file_name) {
                brew_count += 1;
                continue;
            }
            if is_apple_app(&app) {
                continue;
            }
            let source = if is_app_store_app(&app) {
                Source::AppStore
            } else {
                Source::Manual
            };
            let name = file_name.trim_end_matches(".app").to_string();
            results.push((source, name, app));
        }
    }

    if results.is_empty() {
        println!("All third-party apps are managed by Homebrew.");
        return;
    }

    results.sort_by_key(|r| (r.0, r.1.to_lowercase()));

    display::print_apps_header();
    for (source, name, path) in &results {
        display::print_apps_row(name, source.label(), &path.display().to_string());
    }

    let store = results.iter().filter(|r| r.0 == Source::AppStore).count();
    let manual = results.len() - store;
    println!("{}", "─".repeat(90));
    println!(
        "{} not from Homebrew ({} manual, {} App Store) · {} from Homebrew",
        results.len(),
        manual,
        store,
        brew_count
    );
    println!();
}
