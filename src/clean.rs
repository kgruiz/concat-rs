use std::path::{Path, PathBuf};

use anyhow::{Result, bail};
use globset::{Glob, GlobSet, GlobSetBuilder};

use crate::cli;
use crate::sort;

pub fn run(args: cli::CleanArgs) -> Result<()> {
    let recursive = match (args.recursive, args.no_recursive) {
        (true, true) => bail!("Cannot pass both --recursive and --no-recursive."),
        (false, true) => false,
        _ => true,
    };

    let dirs = if args.dirs.is_empty() {
        vec![PathBuf::from(".")]
    } else {
        args.dirs.into_iter().map(PathBuf::from).collect()
    };

    let exts = normalize_exts(args.exts);
    let ignore_exts = normalize_exts(args.ignore_exts);
    let include_set = build_globset(&args.include_globs)?;
    let exclude_set = build_globset(&normalize_exclude_globs(args.exclude_globs))?;

    let mut candidates = Vec::new();
    for dir in dirs {
        candidates.extend(find_concat_files(&dir, recursive));
    }

    candidates.sort_by(|a, b| sort::version_path_cmp(a, b));
    candidates.dedup();

    for file in candidates {
        if !file.is_file() {
            continue;
        }

        if !args.hidden && is_hidden_path(&file) {
            continue;
        }

        let ext = path_ext_lower(&file);
        if !exts.is_empty() && !exts.iter().any(|allowed| allowed == &ext) {
            continue;
        }

        if ignore_exts.iter().any(|ignored| ignored == &ext) {
            continue;
        }

        if let Some(set) = &include_set
            && !set.is_match(&file)
        {
            continue;
        }

        if let Some(set) = &exclude_set
            && set.is_match(&file)
        {
            continue;
        }

        if args.verbose {
            println!("Deleting {}", file.display());
        }

        let _ = std::fs::remove_file(&file);
    }

    Ok(())
}

fn find_concat_files(dir: &Path, recursive: bool) -> Vec<PathBuf> {
    let mut found = Vec::new();

    if recursive {
        for entry in walkdir::WalkDir::new(dir).into_iter().flatten() {
            if !entry.file_type().is_file() {
                continue;
            }

            if entry
                .file_name()
                .to_str()
                .is_some_and(|name| name.starts_with("_concat-"))
            {
                found.push(entry.path().to_path_buf());
            }
        }

        return found;
    }

    for entry in std::fs::read_dir(dir).into_iter().flatten().flatten() {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }

        if path
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.starts_with("_concat-"))
        {
            found.push(path);
        }
    }

    found
}

fn build_globset(patterns: &[String]) -> Result<Option<GlobSet>> {
    if patterns.is_empty() {
        return Ok(None);
    }

    let mut builder = GlobSetBuilder::new();
    for pattern in patterns {
        builder.add(Glob::new(pattern)?);
    }

    Ok(Some(builder.build()?))
}

fn normalize_exts(exts: Vec<String>) -> Vec<String> {
    exts.into_iter()
        .map(|ext| ext.trim().trim_start_matches('.').to_ascii_lowercase())
        .filter(|ext| !ext.is_empty())
        .collect()
}

fn normalize_exclude_globs(globs: Vec<String>) -> Vec<String> {
    globs
        .into_iter()
        .map(|glob| {
            let trimmed = glob.trim().to_string();

            let has_wildcard =
                trimmed.contains('*') || trimmed.contains('?') || trimmed.contains('[');
            let has_slash = trimmed.contains('/');

            if !has_slash && !has_wildcard {
                return format!("**/{trimmed}");
            }

            trimmed
        })
        .filter(|glob| !glob.is_empty())
        .collect()
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
