use clap::ValueEnum;
use clap::{Args, Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(name = "concat", version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,

    #[command(flatten)]
    pub run: RunArgs,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    Clean(CleanArgs),
}

#[derive(Args, Debug, Default)]
pub struct RunArgs {
    #[arg(short = 'o', long = "output")]
    pub output: Option<std::path::PathBuf>,

    #[arg(short = 'r', long = "recursive")]
    pub recursive: bool,

    #[arg(short = 'n', long = "no-recursive")]
    pub no_recursive: bool,

    #[arg(short = 't', long = "text")]
    pub text: bool,

    #[arg(short = 'x', long = "ext")]
    pub exts: Vec<String>,

    #[arg(short = 'g', long = "ignore-ext")]
    pub ignore_exts: Vec<String>,

    #[arg(short = 'I', long = "include")]
    pub include_globs: Vec<String>,

    #[arg(short = 'e', short_alias = 'E', long = "exclude")]
    pub exclude_globs: Vec<String>,

    #[arg(short = 'T', long = "tree")]
    pub tree: bool,

    #[arg(short = 'H', long = "hidden")]
    pub hidden: bool,

    #[arg(short = 'P', long = "no-purge-pycache")]
    pub no_purge_pycache: bool,

    #[arg(short = 'v', long = "verbose")]
    pub verbose: bool,

    #[arg(short = 'd', long = "debug")]
    pub debug: bool,

    #[arg(short = 'l', long = "no-dir-list")]
    pub no_dir_list: bool,

    #[arg(short = 'b', long = "include-binary")]
    pub include_binary: bool,

    #[arg(short = 'M', long = "no-metadata")]
    pub no_metadata: bool,

    #[arg(long = "metadata-sort", value_enum, default_value_t = MetadataSort::Lines)]
    pub metadata_sort: MetadataSort,

    #[arg(short = 'C', long = "no-clean-concat")]
    pub no_clean_concat: bool,

    #[arg(value_name = "FILE|DIR|GLOB", num_args = 0..)]
    pub inputs: Vec<String>,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, ValueEnum)]
pub enum MetadataSort {
    Lines,
    Characters,
    Natural,
}

impl Default for MetadataSort {
    fn default() -> Self {
        Self::Lines
    }
}

#[derive(Args, Debug, Default)]
pub struct CleanArgs {
    #[arg(short = 'r', long = "recursive")]
    pub recursive: bool,

    #[arg(short = 'n', long = "no-recursive")]
    pub no_recursive: bool,

    #[arg(short = 'I', long = "include")]
    pub include_globs: Vec<String>,

    #[arg(short = 'e', short_alias = 'E', long = "exclude")]
    pub exclude_globs: Vec<String>,

    #[arg(short = 'x', long = "ext")]
    pub exts: Vec<String>,

    #[arg(short = 'g', long = "ignore-ext")]
    pub ignore_exts: Vec<String>,

    #[arg(short = 'H', long = "hidden")]
    pub hidden: bool,

    #[arg(short = 'v', long = "verbose")]
    pub verbose: bool,

    #[arg(value_name = "DIR", num_args = 0..)]
    pub dirs: Vec<String>,
}
