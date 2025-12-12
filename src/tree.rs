use std::path::{Path, PathBuf};

use anyhow::Result;

use crate::sort;

pub fn build_tree(root: &Path, include_hidden: bool) -> Result<String> {
    let mut out = String::new();
    build_tree_inner(root, include_hidden, "", true, &mut out)?;

    if out.ends_with('\n') {
        out.pop();
    }

    Ok(out)
}

fn build_tree_inner(
    dir: &Path,
    include_hidden: bool,
    prefix: &str,
    is_last: bool,
    out: &mut String,
) -> Result<()> {
    if dir != Path::new(".") && dir != Path::new("") {
        let name = dir.file_name().unwrap_or_default().to_string_lossy();
        let connector = if is_last { "└── " } else { "├── " };
        out.push_str(prefix);
        out.push_str(connector);
        out.push_str(&name);
        out.push('\n');
    }

    let entries = list_dir_entries(dir, include_hidden)?;
    let child_prefix = if dir == Path::new(".") || dir == Path::new("") {
        prefix.to_string()
    } else if is_last {
        format!("{prefix}    ")
    } else {
        format!("{prefix}│   ")
    };

    for (index, entry) in entries.iter().enumerate() {
        let is_last_child = index + 1 == entries.len();
        if entry.is_dir() {
            build_tree_inner(entry, include_hidden, &child_prefix, is_last_child, out)?;
        } else {
            let name = entry.file_name().unwrap_or_default().to_string_lossy();
            let connector = if is_last_child {
                "└── "
            } else {
                "├── "
            };
            out.push_str(&child_prefix);
            out.push_str(connector);
            out.push_str(&name);
            out.push('\n');
        }
    }

    Ok(())
}

fn list_dir_entries(dir: &Path, include_hidden: bool) -> Result<Vec<PathBuf>> {
    let mut entries = Vec::new();
    for entry in std::fs::read_dir(dir)? {
        let entry = match entry {
            Ok(entry) => entry,
            Err(_) => continue,
        };

        let path = entry.path();
        let name = match path.file_name().and_then(|name| name.to_str()) {
            Some(name) => name,
            None => continue,
        };

        if !include_hidden && name.starts_with('.') {
            continue;
        }

        entries.push(path);
    }

    entries.sort_by(|a, b| sort::version_path_cmp(a, b));
    Ok(entries)
}
