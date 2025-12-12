use anyhow::Result;

use crate::config::RunConfig;

pub fn run(config: RunConfig) -> Result<()> {
    let expanded = crate::inputs::expand_inputs(&config);
    let candidates = crate::discover::collect_candidate_files(&config, &expanded.items)?;

    let ctx = crate::filter::FilterContext {
        explicit_file_inputs: expanded.explicit_files,
    };

    let matched = crate::filter::filter_candidates(&config, &ctx, &candidates, None)?;

    if config.verbose {
        eprintln!("Matched {} files.", matched.len());
    }

    Ok(())
}
