use std::path::PathBuf;

use anyhow::Result;
use walkdir::WalkDir;

use crate::config::RunConfig;
use crate::sort;

pub fn collect_candidate_files(
    config: &RunConfig,
    expanded_inputs: &[PathBuf],
) -> Result<Vec<PathBuf>> {
    let mut raw_candidates = Vec::new();

    if config.verbose {
        eprintln!("Collecting candidate files...");
    }

    for item in expanded_inputs {
        if item.is_file() {
            raw_candidates.push(item.clone());

            if config.verbose {
                eprintln!("Input item is file: \"{}\"", item.display());
            }

            continue;
        }

        if item.is_dir() {
            if config.verbose {
                eprintln!(
                    "Input item is directory, searching: \"{}\" (Recursive: {})",
                    item.display(),
                    config.recursive
                );
            }

            let allow_hidden_by_include_globs = config
                .include_globs
                .iter()
                .any(|pattern| pattern_implies_hidden(pattern));

            let root_is_hidden = is_hidden_name(item.file_name());
            let mut walker = WalkDir::new(item);

            if !config.recursive {
                walker = walker.max_depth(1);
            }

            let mut iter = walker.into_iter();
            while let Some(entry) = iter.next() {
                let entry = match entry {
                    Ok(entry) => entry,
                    Err(_) => continue,
                };

                if !config.include_hidden && !allow_hidden_by_include_globs && entry.depth() > 0 {
                    let name_is_hidden = is_hidden_name(Some(entry.file_name()));
                    let should_prune = name_is_hidden && (root_is_hidden || entry.depth() == 1);

                    if should_prune {
                        if entry.file_type().is_dir() {
                            iter.skip_current_dir();
                        }

                        continue;
                    }
                }

                if entry.file_type().is_file() {
                    raw_candidates.push(entry.path().to_path_buf());
                }
            }

            continue;
        }

        eprintln!(
            "Warning: Input item is neither a file nor a directory, skipping: \"{}\"",
            item.display()
        );
    }

    let mut candidates = raw_candidates;
    candidates.sort_by(|a, b| sort::version_path_cmp(a, b));
    candidates.dedup();

    if config.verbose {
        eprintln!("Total unique candidate files found: {}", candidates.len());
    }

    Ok(candidates)
}

fn is_hidden_name(name: Option<&std::ffi::OsStr>) -> bool {
    name.and_then(|name| name.to_str())
        .is_some_and(|name| name.starts_with('.'))
}

fn pattern_implies_hidden(pattern: &str) -> bool {
    pattern.starts_with('.') || pattern.contains("/.") || pattern.contains("\\.")
}
