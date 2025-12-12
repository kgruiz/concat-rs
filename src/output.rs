use std::collections::BTreeMap;
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};

use anyhow::Result;
use base64::Engine as _;

use crate::cli::MetadataSort;
use crate::config::{OutputFormat, RunConfig};
use crate::sort;
use crate::text_detect;

#[derive(Debug, Clone)]
struct FileMetadata {
    path: PathBuf,
    lines: usize,
    characters: usize,
    is_binary: bool,
    read_error: Option<String>,
}

pub fn write_output(
    config: &RunConfig,
    output_path: &Path,
    matched_files: &[PathBuf],
    tree: Option<&str>,
) -> Result<()> {
    let file = std::fs::File::create(output_path)?;
    let mut out = BufWriter::new(file);
    write_output_to_writer(config, matched_files, tree, &mut out)?;
    out.flush()?;
    Ok(())
}

pub fn render_output(
    config: &RunConfig,
    matched_files: &[PathBuf],
    tree: Option<&str>,
) -> Result<String> {
    let mut buffer = Vec::new();
    {
        let mut out = BufWriter::new(&mut buffer);
        write_output_to_writer(config, matched_files, tree, &mut out)?;
        out.flush()?;
    }

    Ok(String::from_utf8_lossy(&buffer).into_owned())
}

fn write_output_to_writer(
    config: &RunConfig,
    matched_files: &[PathBuf],
    tree: Option<&str>,
    out: &mut dyn Write,
) -> Result<()> {
    let metadata = if config.show_metadata {
        Some(collect_file_metadata(matched_files, config.metadata_sort))
    } else {
        None
    };

    match config.format {
        OutputFormat::Xml => {
            write_xml_output(config, matched_files, tree, metadata.as_deref(), out)
        }
        OutputFormat::Text => {
            write_text_output(config, matched_files, tree, metadata.as_deref(), out)
        }
    }
}

fn write_xml_output(
    config: &RunConfig,
    matched_files: &[PathBuf],
    tree: Option<&str>,
    metadata: Option<&[FileMetadata]>,
    out: &mut dyn Write,
) -> Result<()> {
    writeln!(out, "<?xml version=\"1.0\" encoding=\"UTF-8\"?>")?;
    writeln!(out, "<concatenation>")?;

    if let Some(tree) = tree {
        writeln!(out, "  <directoryTree context=\".\">")?;
        writeln!(out, "    <representation><![CDATA[")?;
        write_cdata_body(out, tree)?;
        writeln!(out, "]]></representation>")?;
        writeln!(out, "  </directoryTree>")?;
    }

    if config.show_dir_list {
        write_matched_dir_list_xml(out, matched_files)?;
    }

    if let Some(metadata) = metadata {
        write_file_metadata_xml(out, metadata)?;
    }

    writeln!(out, "  <fileContents count=\"{}\">", matched_files.len())?;

    if matched_files.is_empty() {
        writeln!(out, "    <message>No files matched the criteria.</message>")?;
    } else {
        for file_path in matched_files {
            let absolute_path = canonical_or_fallback(file_path);
            let absolute_path_string = absolute_path.to_string_lossy();

            writeln!(out, "    <file>")?;
            writeln!(
                out,
                "      <path>{}</path>",
                xml_escape_text(&absolute_path_string)
            )?;

            let bytes = std::fs::read(file_path);
            let (content, encoding_attr) = match bytes {
                Ok(bytes) => {
                    if config.include_binary && !text_detect::bytes_are_probably_text(&bytes) {
                        (
                            base64::engine::general_purpose::STANDARD.encode(bytes),
                            Some("base64"),
                        )
                    } else {
                        (sanitize_xml_text(&String::from_utf8_lossy(&bytes)), None)
                    }
                }
                Err(_) => (
                    format!("Error reading file content for {}", file_path.display()),
                    None,
                ),
            };

            if let Some(encoding) = encoding_attr {
                writeln!(out, "      <content encoding=\"{encoding}\"><![CDATA[")?;
            } else {
                writeln!(out, "      <content><![CDATA[")?;
            }

            write_cdata_body(out, &content)?;
            writeln!(out, "]]></content>")?;
            writeln!(out, "    </file>")?;
        }
    }

    writeln!(out, "  </fileContents>")?;
    writeln!(out, "</concatenation>")?;

    out.flush()?;
    Ok(())
}

fn write_matched_dir_list_xml(out: &mut dyn Write, matched_files: &[PathBuf]) -> Result<()> {
    writeln!(out, "  <matchedFilesDirStructureList>")?;

    let cwd = std::env::current_dir()?
        .canonicalize()
        .unwrap_or(std::env::current_dir()?);

    let mut grouped: BTreeMap<PathBuf, Vec<String>> = BTreeMap::new();
    for file in matched_files {
        let full = canonical_or_fallback(file);
        let dir = full.parent().unwrap_or(Path::new("")).to_path_buf();
        let base = full
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        if base.is_empty() {
            continue;
        }

        grouped.entry(dir).or_default().push(base);
    }

    let mut dirs: Vec<PathBuf> = grouped.keys().cloned().collect();
    dirs.sort_by(|a, b| sort::version_path_cmp(a, b));

    for dir in dirs {
        let relative_dir = if dir == cwd {
            ".".to_string()
        } else if let Ok(stripped) = dir.strip_prefix(&cwd) {
            stripped.to_string_lossy().to_string()
        } else {
            dir.to_string_lossy().to_string()
        };

        let files = grouped.get(&dir).cloned().unwrap_or_default();
        let files_joined = files
            .into_iter()
            .map(|name| format!("\"{name}\""))
            .collect::<Vec<_>>()
            .join(", ");

        writeln!(
            out,
            "    <dirEntry>\"{}\": [{}]</dirEntry>",
            xml_escape_text(&relative_dir),
            xml_escape_text(&files_joined)
        )?;
    }

    writeln!(out, "  </matchedFilesDirStructureList>")?;
    Ok(())
}

fn write_file_metadata_xml(out: &mut dyn Write, metadata: &[FileMetadata]) -> Result<()> {
    writeln!(out, "  <fileMetadata count=\"{}\">", metadata.len())?;

    if metadata.is_empty() {
        writeln!(out, "    <message>No files matched the criteria.</message>")?;
        writeln!(out, "  </fileMetadata>")?;
        return Ok(());
    }

    for entry in metadata {
        let path = entry.path.to_string_lossy().to_string();
        let binary_attr = if entry.is_binary { "true" } else { "false" };

        writeln!(out, "    <file binary=\"{binary_attr}\">")?;
        writeln!(out, "      <path>{}</path>", xml_escape_text(&path))?;

        if let Some(error) = &entry.read_error {
            writeln!(out, "      <error>{}</error>", xml_escape_text(error))?;
        } else {
            writeln!(out, "      <lines>{}</lines>", entry.lines)?;
            writeln!(out, "      <characters>{}</characters>", entry.characters)?;
        }

        writeln!(out, "    </file>")?;
    }

    writeln!(out, "  </fileMetadata>")?;
    Ok(())
}

fn write_text_output(
    config: &RunConfig,
    matched_files: &[PathBuf],
    tree: Option<&str>,
    metadata: Option<&[FileMetadata]>,
    out: &mut dyn Write,
) -> Result<()> {
    if let Some(tree) = tree {
        writeln!(
            out,
            "--------------------------------------------------------------------------------"
        )?;
        writeln!(out, "# Directory Tree (from current directory)")?;
        writeln!(
            out,
            "********************************************************************************"
        )?;
        writeln!(out, ".")?;
        writeln!(out, "{tree}")?;
        writeln!(
            out,
            "================================================================================"
        )?;
        writeln!(out)?;
    }

    if let Some(metadata) = metadata {
        write_file_metadata_text(out, metadata)?;
    }

    writeln!(
        out,
        "--------------------------------------------------------------------------------"
    )?;
    writeln!(out, "# File Contents ({} files)", matched_files.len())?;
    writeln!(
        out,
        "********************************************************************************"
    )?;

    if matched_files.is_empty() {
        writeln!(out, "No files matched the criteria.")?;
        writeln!(
            out,
            "================================================================================"
        )?;
        out.flush()?;
        return Ok(());
    }

    for (index, file_path) in matched_files.iter().enumerate() {
        let current_file = index + 1;
        let absolute_path = canonical_or_fallback(file_path);
        let absolute_path_string = absolute_path.to_string_lossy();

        writeln!(out)?;
        writeln!(
            out,
            "--------------------------------------------------------------------------------"
        )?;
        writeln!(
            out,
            "# File {current_file}/{}: {absolute_path_string}",
            matched_files.len()
        )?;
        writeln!(
            out,
            "********************************************************************************"
        )?;

        let bytes = std::fs::read(file_path);
        match bytes {
            Ok(bytes) => {
                if config.include_binary && !text_detect::bytes_are_probably_text(&bytes) {
                    writeln!(out, "[BINARY FILE: base64]")?;
                    let encoded = base64::engine::general_purpose::STANDARD.encode(bytes);
                    writeln!(out, "{encoded}")?;
                } else {
                    let text = String::from_utf8_lossy(&bytes);
                    write!(out, "{text}")?;

                    if !bytes.ends_with(b"\n") {
                        writeln!(out)?;
                    }
                }

                writeln!(out)?;
                writeln!(out, "# EOF: {absolute_path_string}")?;
                writeln!(
                    out,
                    "================================================================================"
                )?;
            }
            Err(_) => {
                eprintln!("Error: Cannot read file '{}'.", absolute_path_string);
                writeln!(
                    out,
                    "================================================================================"
                )?;
            }
        }
    }

    out.flush()?;
    Ok(())
}

fn write_file_metadata_text(out: &mut dyn Write, metadata: &[FileMetadata]) -> Result<()> {
    writeln!(
        out,
        "--------------------------------------------------------------------------------"
    )?;
    writeln!(out, "# File Metadata ({} files)", metadata.len())?;
    writeln!(
        out,
        "********************************************************************************"
    )?;

    if metadata.is_empty() {
        writeln!(out, "No files matched the criteria.")?;
        writeln!(
            out,
            "================================================================================"
        )?;
        writeln!(out)?;
        return Ok(());
    }

    for (index, entry) in metadata.iter().enumerate() {
        let path = entry.path.to_string_lossy();
        let binary_marker = if entry.is_binary { " [binary]" } else { "" };

        if let Some(error) = &entry.read_error {
            writeln!(
                out,
                "{}: {}{binary_marker} (error: {})",
                index + 1,
                path,
                error
            )?;
            continue;
        }

        writeln!(
            out,
            "{}: {}{binary_marker} (lines: {}, chars: {})",
            index + 1,
            path,
            entry.lines,
            entry.characters
        )?;
    }

    writeln!(
        out,
        "================================================================================"
    )?;
    writeln!(out)?;
    Ok(())
}

fn canonical_or_fallback(path: &Path) -> PathBuf {
    std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf())
}

fn collect_file_metadata(matched_files: &[PathBuf], sort_by: MetadataSort) -> Vec<FileMetadata> {
    let mut metadata: Vec<FileMetadata> = matched_files
        .iter()
        .map(|path| build_file_metadata(path))
        .collect();

    match sort_by {
        MetadataSort::Lines => {
            metadata.sort_by(|a, b| b.lines.cmp(&a.lines).then_with(|| a.path.cmp(&b.path)));
        }
        MetadataSort::Characters => {
            metadata.sort_by(|a, b| {
                b.characters
                    .cmp(&a.characters)
                    .then_with(|| a.path.cmp(&b.path))
            });
        }
        MetadataSort::Natural => {}
    }

    metadata
}

fn build_file_metadata(path: &Path) -> FileMetadata {
    let absolute_path = canonical_or_fallback(path);
    let bytes = std::fs::read(path);

    match bytes {
        Ok(bytes) => {
            let is_text = text_detect::bytes_are_probably_text(&bytes);

            if is_text {
                let text = String::from_utf8_lossy(&bytes);
                let lines = text.lines().count();
                let characters = text.chars().count();

                FileMetadata {
                    path: absolute_path,
                    lines,
                    characters,
                    is_binary: false,
                    read_error: None,
                }
            } else {
                let lines = count_lines_in_bytes(&bytes);

                FileMetadata {
                    path: absolute_path,
                    lines,
                    characters: bytes.len(),
                    is_binary: true,
                    read_error: None,
                }
            }
        }
        Err(err) => FileMetadata {
            path: absolute_path,
            lines: 0,
            characters: 0,
            is_binary: false,
            read_error: Some(err.to_string()),
        },
    }
}

fn count_lines_in_bytes(bytes: &[u8]) -> usize {
    if bytes.is_empty() {
        return 0;
    }

    let newline_count = bytes.iter().filter(|&&byte| byte == b'\n').count();

    if bytes.ends_with(b"\n") {
        newline_count
    } else {
        newline_count + 1
    }
}

fn xml_escape_text(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

fn sanitize_xml_text(input: &str) -> String {
    input
        .chars()
        .map(|ch| {
            let code = ch as u32;
            let allowed = ch == '\t'
                || ch == '\n'
                || ch == '\r'
                || (0x20..=0xD7FF).contains(&code)
                || (0xE000..=0xFFFD).contains(&code)
                || (0x10000..=0x10FFFF).contains(&code);

            if allowed { ch } else { '\u{FFFD}' }
        })
        .collect()
}

fn write_cdata_body(out: &mut dyn Write, input: &str) -> Result<()> {
    for (index, part) in input.split("]]>").enumerate() {
        if index > 0 {
            write!(out, "]]]]><![CDATA[>")?;
        }

        write!(out, "{part}")?;
    }

    Ok(())
}
