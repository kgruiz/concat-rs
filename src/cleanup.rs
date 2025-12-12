use std::path::Path;

use anyhow::Result;

pub fn purge_python_cache_in_cwd(verbose: bool) -> Result<()> {
    if verbose {
        eprintln!("Searching for and removing __pycache__ directories and .pyc files in .");
    }

    let cwd = std::env::current_dir()?;

    let mut pycache_dirs = Vec::new();
    let mut pyc_files = Vec::new();

    for entry in walkdir::WalkDir::new(&cwd).into_iter().flatten() {
        if entry.depth() == 0 {
            continue;
        }

        let path = entry.path();

        if entry.file_type().is_dir()
            && path.file_name() == Some(std::ffi::OsStr::new("__pycache__"))
        {
            pycache_dirs.push(path.to_path_buf());
        }

        if entry.file_type().is_file() && path.extension() == Some(std::ffi::OsStr::new("pyc")) {
            pyc_files.push(path.to_path_buf());
        }
    }

    for file in pyc_files {
        let _ = std::fs::remove_file(&file);
    }

    for dir in pycache_dirs {
        let _ = std::fs::remove_dir_all(&dir);
    }

    Ok(())
}

pub fn delete_concat_outputs_in_cwd(verbose: bool) -> Result<()> {
    if verbose {
        eprintln!("Deleting existing _concat-* files");
    }

    let cwd = std::env::current_dir()?;
    for entry in std::fs::read_dir(&cwd)? {
        let entry = match entry {
            Ok(entry) => entry,
            Err(_) => continue,
        };

        let path = entry.path();
        if !path.is_file() {
            continue;
        }

        if let Some(file_name) = path.file_name().and_then(|name| name.to_str())
            && file_name.starts_with("_concat-")
        {
            let _ = std::fs::remove_file(&path);
        }
    }

    Ok(())
}

pub fn remove_existing_output_file(output_path: &Path, verbose: bool) -> Result<()> {
    if output_path.exists() {
        if verbose {
            eprintln!(
                "Removing existing output file: \"{}\"",
                output_path.display()
            );
        }

        std::fs::remove_file(output_path)?;
    }

    Ok(())
}
