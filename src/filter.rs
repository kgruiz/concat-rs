use std::collections::HashSet;
use std::path::{Path, PathBuf};

use anyhow::Result;
use globset::{Glob, GlobSet, GlobSetBuilder};

use crate::config::RunConfig;
use crate::text_detect;

pub struct FilterContext {
    pub explicit_file_inputs: HashSet<PathBuf>,
}

pub fn filter_candidates(
    config: &RunConfig,
    ctx: &FilterContext,
    candidates: &[PathBuf],
    output_path: Option<&Path>,
) -> Result<Vec<PathBuf>> {
    let include_set = build_globset(&config.include_globs)?;
    let include_hidden_set = build_globset(
        &config
            .include_globs
            .iter()
            .filter(|pattern| pattern_implies_hidden(pattern))
            .cloned()
            .collect::<Vec<_>>(),
    )?;

    let exclude_set = build_globset(&config.exclude_globs)?;

    let mut matched = Vec::new();

    if config.verbose {
        eprintln!("Filtering candidate files...");
    }

    for file_path in candidates {
        if let Some(output_path) = output_path
            && file_path == output_path
        {
            if config.verbose {
                eprintln!(
                    "Skipped file: \"{}\" (is the output file)",
                    file_path.display()
                );
            }

            continue;
        }

        let is_hidden = is_hidden_path(file_path);
        if is_hidden && !config.include_hidden {
            let explicit_or_include_hidden = ctx.explicit_file_inputs.contains(file_path)
                || include_hidden_set
                    .as_ref()
                    .is_some_and(|set| set.is_match(file_path));

            if !explicit_or_include_hidden {
                if config.verbose {
                    eprintln!(
                        "Skipped file: \"{}\" (hidden and not explicitly included)",
                        file_path.display()
                    );
                }

                continue;
            }
        }

        if !config.exts.is_empty() {
            let ext = path_ext_lower(file_path);
            if !config.exts.iter().any(|allowed| allowed == &ext) {
                if config.verbose {
                    eprintln!(
                        "Skipped file: \"{}\" (extension mismatch: '{}' not in {{{}}})",
                        file_path.display(),
                        ext,
                        config.exts.join(" ")
                    );
                }

                continue;
            }
        }

        if !config.ignore_exts.is_empty() {
            let ext = path_ext_lower(file_path);
            if config.ignore_exts.iter().any(|ignored| ignored == &ext) {
                if config.verbose {
                    eprintln!(
                        "Skipped file: \"{}\" (ignored extension: '{}')",
                        file_path.display(),
                        ext
                    );
                }

                continue;
            }
        }

        if let Some(set) = &include_set
            && !set.is_match(file_path)
        {
            if config.verbose {
                eprintln!(
                    "Skipped file: \"{}\" (include glob mismatch)",
                    file_path.display()
                );
            }

            continue;
        }

        if let Some(set) = &exclude_set {
            let basename = file_path.file_name().unwrap_or_default();

            if set.is_match(file_path) || set.is_match(basename) {
                if config.verbose {
                    eprintln!(
                        "Skipped file: \"{}\" (exclude glob match)",
                        file_path.display()
                    );
                }

                continue;
            }
        }

        if !config.include_binary && !text_detect::is_probably_text(file_path)? {
            if config.verbose {
                eprintln!("Skipped file: \"{}\" (not text)", file_path.display());
            }

            continue;
        }

        matched.push(file_path.clone());

        if config.verbose {
            eprintln!("Matched file: \"{}\"", file_path.display());
        }
    }

    if config.verbose {
        eprintln!("Total matched files: {}", matched.len());
    }

    Ok(matched)
}

fn build_globset(patterns: &[String]) -> Result<Option<GlobSet>> {
    if patterns.is_empty() {
        return Ok(None);
    }

    let mut builder = GlobSetBuilder::new();
    for pattern in patterns {
        let glob = Glob::new(pattern)?;
        builder.add(glob);
    }

    Ok(Some(builder.build()?))
}

fn pattern_implies_hidden(pattern: &str) -> bool {
    pattern.starts_with('.') || pattern.contains("/.") || pattern.contains("\\.")
}

fn is_hidden_path(path: &Path) -> bool {
    path.components().any(|component| match component {
        std::path::Component::Normal(name) => is_hidden_name(Some(name)),
        _ => false,
    })
}

fn is_hidden_name(name: Option<&std::ffi::OsStr>) -> bool {
    name.and_then(|name| name.to_str())
        .is_some_and(|name| name.starts_with('.'))
}

fn path_ext_lower(path: &Path) -> String {
    path.extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or_default()
        .trim_start_matches('.')
        .to_ascii_lowercase()
}
