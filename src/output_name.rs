use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use crate::config::{OutputFormat, RunConfig};

pub fn resolve_output_path(config: &RunConfig, expanded_inputs: &[PathBuf]) -> Result<PathBuf> {
    let cwd = std::env::current_dir().context("Failed to get current working directory")?;

    let (mut path, user_provided) = match &config.output {
        Some(output) => (output.clone(), true),
        None => (default_output_name(config, expanded_inputs)?, false),
    };

    let required_ext = match config.format {
        OutputFormat::Xml => "xml",
        OutputFormat::Text => "txt",
    };

    let has_required_ext = path
        .extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| ext.eq_ignore_ascii_case(required_ext));

    if !has_required_ext {
        path.set_extension(required_ext);
    }

    let absolute = if path.is_absolute() {
        path
    } else {
        cwd.join(path)
    };

    if user_provided && let Some(parent) = absolute.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("Cannot create output directory \"{}\".", parent.display()))?;
    }

    Ok(normalize_path(&absolute))
}

fn default_output_name(config: &RunConfig, expanded_inputs: &[PathBuf]) -> Result<PathBuf> {
    let base = if !config.exts.is_empty() {
        if config.exts.len() == 1 {
            format!("_concat-{}", config.exts[0])
        } else {
            "_concat-output".to_string()
        }
    } else if !config.had_user_args {
        let cwd = std::env::current_dir()?
            .canonicalize()
            .unwrap_or(std::env::current_dir()?);
        let project_base = cwd
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("output");

        if project_base.is_empty() {
            "_concat-output".to_string()
        } else {
            format!("_concat-{project_base}")
        }
    } else if expanded_inputs.len() == 1 && expanded_inputs[0].is_dir() {
        let single_dir = &expanded_inputs[0];
        let base = single_dir
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("output");
        if base.is_empty() {
            "_concat-output".to_string()
        } else {
            format!("_concat-{base}")
        }
    } else {
        "_concat-output".to_string()
    };

    Ok(PathBuf::from(base))
}

fn normalize_path(path: &Path) -> PathBuf {
    let mut normalized = PathBuf::new();

    for component in path.components() {
        match component {
            std::path::Component::CurDir => {}
            std::path::Component::ParentDir => {
                normalized.pop();
            }
            component => normalized.push(component.as_os_str()),
        }
    }

    normalized
}
