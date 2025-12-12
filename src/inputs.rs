use std::collections::HashSet;
use std::path::PathBuf;

use glob::glob;

use crate::config::RunConfig;

pub struct ExpandedInputs {
    pub items: Vec<PathBuf>,
    pub explicit_files: HashSet<PathBuf>,
}

pub fn expand_inputs(config: &RunConfig) -> ExpandedInputs {
    let mut expanded = Vec::new();

    for input in &config.inputs {
        if has_glob_metacharacters(input) {
            let mut matched = Vec::new();
            for path in glob(input).into_iter().flatten().flatten() {
                matched.push(path);
            }

            if matched.is_empty() {
                eprintln!("Warning: Input glob pattern matched no files: {input}");
            } else {
                expanded.extend(matched);
            }

            continue;
        }

        expanded.push(PathBuf::from(input));
    }

    let mut items = Vec::new();
    let mut explicit_files = HashSet::new();

    for input in expanded {
        if !input.exists() {
            eprintln!(
                "Warning: Input item not found, skipping: \"{}\"",
                input.display()
            );
            continue;
        }

        let resolved = match std::fs::canonicalize(&input) {
            Ok(path) => path,
            Err(_) => {
                eprintln!(
                    "Warning: Cannot resolve path for input item, skipping: \"{}\"",
                    input.display()
                );
                continue;
            }
        };

        if resolved.is_file() {
            explicit_files.insert(resolved.clone());
        }

        items.push(resolved);
    }

    ExpandedInputs {
        items,
        explicit_files,
    }
}

fn has_glob_metacharacters(input: &str) -> bool {
    input.contains('*') || input.contains('?') || input.contains('[')
}
