use anyhow::Result;

use crate::config::RunConfig;

pub fn run(config: RunConfig) -> Result<()> {
    let expanded = crate::inputs::expand_inputs(&config);
    let output_path = crate::output_name::resolve_output_path(&config, &expanded.items)?;

    if config.purge_pycache {
        crate::cleanup::purge_python_cache_in_cwd(config.verbose)?;
    }

    if config.clean_concat_files {
        crate::cleanup::delete_concat_outputs_in_cwd(config.verbose)?;
    }

    crate::cleanup::remove_existing_output_file(&output_path, config.verbose)?;

    let candidates = crate::discover::collect_candidate_files(&config, &expanded.items)?;

    let ctx = crate::filter::FilterContext {
        explicit_file_inputs: expanded.explicit_files,
    };

    let matched = crate::filter::filter_candidates(&config, &ctx, &candidates, Some(&output_path))?;

    let tree = if config.show_tree {
        Some(crate::tree::build_tree(
            std::path::Path::new("."),
            config.include_hidden,
        )?)
    } else {
        None
    };

    crate::output::write_output(&config, &output_path, &matched, tree.as_deref())?;

    if config.verbose {
        eprintln!(
            "Concatenation complete. Output written to \"{}\".",
            output_path.display()
        );
    }

    Ok(())
}
