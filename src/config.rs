use anyhow::{Result, bail};

use crate::cli;
use crate::cli::MetadataSort;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OutputFormat {
    Xml,
    Text,
}

#[derive(Debug)]
pub struct RunConfig {
    pub output: Option<std::path::PathBuf>,
    pub recursive: bool,
    pub format: OutputFormat,
    pub exts: Vec<String>,
    pub ignore_exts: Vec<String>,
    pub include_globs: Vec<String>,
    pub exclude_globs: Vec<String>,
    pub show_tree: bool,
    pub include_hidden: bool,
    pub purge_pycache: bool,
    pub verbose: bool,
    pub debug: bool,
    pub show_dir_list: bool,
    pub show_metadata: bool,
    pub metadata_sort: MetadataSort,
    pub include_binary: bool,
    pub clean_concat_files: bool,
    pub inputs: Vec<String>,
    pub had_user_args: bool,
}

impl RunConfig {
    pub fn from_cli(cli: cli::Cli, had_user_args: bool) -> Result<Self> {
        let mut inputs = cli.run.inputs;
        if inputs.is_empty() {
            inputs.push(".".to_string());
        }

        let recursive = match (cli.run.recursive, cli.run.no_recursive) {
            (true, true) => bail!("Cannot pass both --recursive and --no-recursive."),
            (false, true) => false,
            _ => true,
        };

        let format = if cli.run.text {
            OutputFormat::Text
        } else {
            OutputFormat::Xml
        };

        let exts = normalize_exts(cli.run.exts);
        let ignore_exts = normalize_exts(cli.run.ignore_exts);
        let exclude_globs = normalize_exclude_globs(cli.run.exclude_globs);

        Ok(Self {
            output: cli.run.output,
            recursive,
            format,
            exts,
            ignore_exts,
            include_globs: cli.run.include_globs,
            exclude_globs,
            show_tree: cli.run.tree,
            include_hidden: cli.run.hidden,
            purge_pycache: !cli.run.no_purge_pycache,
            verbose: cli.run.verbose,
            debug: cli.run.debug,
            show_dir_list: !cli.run.no_dir_list,
            show_metadata: !cli.run.no_metadata,
            metadata_sort: cli.run.metadata_sort,
            include_binary: cli.run.include_binary,
            clean_concat_files: !cli.run.no_clean_concat,
            inputs,
            had_user_args,
        })
    }

    pub fn print_summary(&self) {
        if !self.verbose {
            return;
        }

        eprintln!("----------------------------------------");
        eprintln!("Configuration:");
        eprintln!("Inputs: {}", self.inputs.join(" "));

        match &self.output {
            Some(path) => eprintln!("Output File (requested): \"{}\"", path.display()),
            None => eprintln!("Output File (requested): <auto>"),
        }

        eprintln!("Format: {}", self.format.as_str());
        eprintln!("Recursive: {}", self.recursive);
        eprintln!("Include Hidden: {}", self.include_hidden);
        eprintln!("Show Tree: {}", self.show_tree);
        eprintln!("Show Dir List: {}", self.show_dir_list);
        eprintln!("Show Metadata: {}", self.show_metadata);
        eprintln!("Metadata Sort: {:?}", self.metadata_sort);
        eprintln!("Purge Pycache (in CWD): {}", self.purge_pycache);
        eprintln!("Include Binary: {}", self.include_binary);
        eprintln!("Clean _concat-* in CWD: {}", self.clean_concat_files);
        eprintln!("Had User Args: {}", self.had_user_args);

        if self.exts.is_empty() {
            eprintln!("Include Extensions: All");
        } else {
            eprintln!("Include Extensions: {}", self.exts.join(" "));
        }

        if self.include_globs.is_empty() {
            eprintln!("Include Globs: All");
        } else {
            eprintln!("Include Globs: {}", self.include_globs.join(" "));
        }

        if self.exclude_globs.is_empty() {
            eprintln!("Exclude Globs: None");
        } else {
            eprintln!("Exclude Globs: {}", self.exclude_globs.join(" "));
        }

        if self.ignore_exts.is_empty() {
            eprintln!("Ignore Extensions: None");
        } else {
            eprintln!("Ignore Extensions: {}", self.ignore_exts.join(" "));
        }

        eprintln!("Debug Mode: {}", self.debug);
        eprintln!("----------------------------------------");
    }
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

impl OutputFormat {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Xml => "xml",
            Self::Text => "text",
        }
    }
}
