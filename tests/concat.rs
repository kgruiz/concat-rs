use std::fs;

use assert_cmd::cargo::cargo_bin_cmd;
use tempfile::{Builder, TempDir};

fn non_hidden_tempdir() -> anyhow::Result<TempDir> {
    Ok(Builder::new().prefix("concat-test-").tempdir()?)
}

#[test]
fn no_args_defaults_to_cwd_named_xml() -> anyhow::Result<()> {
    let dir = non_hidden_tempdir()?;
    let cwd_name = dir.path().file_name().unwrap().to_string_lossy();
    let expected = dir.path().join(format!("_concat-{cwd_name}.xml"));

    let mut cmd = cargo_bin_cmd!("concat");
    cmd.current_dir(dir.path()).assert().success();

    assert!(
        expected.is_file(),
        "Expected output file at {}",
        expected.display()
    );
    Ok(())
}

#[test]
fn single_dir_input_defaults_to_dir_named_xml() -> anyhow::Result<()> {
    let dir = non_hidden_tempdir()?;
    let src = dir.path().join("src");
    fs::create_dir_all(&src)?;
    fs::write(src.join("a.rs"), "fn main() {}\n")?;

    let expected = dir.path().join("_concat-src.xml");

    let mut cmd = cargo_bin_cmd!("concat");
    cmd.current_dir(dir.path()).arg("src").assert().success();

    assert!(
        expected.is_file(),
        "Expected output file at {}",
        expected.display()
    );
    Ok(())
}

#[test]
fn text_mode_uses_txt_extension() -> anyhow::Result<()> {
    let dir = non_hidden_tempdir()?;
    let src = dir.path().join("src");
    fs::create_dir_all(&src)?;
    fs::write(src.join("a.txt"), "hello\n")?;

    let expected = dir.path().join("_concat-src.txt");

    let mut cmd = cargo_bin_cmd!("concat");
    cmd.current_dir(dir.path())
        .args(["-t", "src"])
        .assert()
        .success();

    let out = fs::read_to_string(&expected)?;
    assert!(out.contains("# File Contents (1 files)"));
    Ok(())
}

#[test]
fn metadata_xml_is_included_by_default() -> anyhow::Result<()> {
    let dir = non_hidden_tempdir()?;
    let src = dir.path().join("src");
    fs::create_dir_all(&src)?;
    fs::write(src.join("a.txt"), "a\nb\n")?;

    let expected = dir.path().join("_concat-src.xml");

    let mut cmd = cargo_bin_cmd!("concat");
    cmd.current_dir(dir.path()).arg("src").assert().success();

    let out = fs::read_to_string(&expected)?;

    assert!(out.contains("<fileMetadata count=\"1\">"));
    assert!(out.contains("<lines>2</lines>"));
    assert!(out.contains("<characters>4</characters>"));
    Ok(())
}

#[test]
fn metadata_text_output_is_included_by_default() -> anyhow::Result<()> {
    let dir = non_hidden_tempdir()?;
    let src = dir.path().join("src");
    fs::create_dir_all(&src)?;
    fs::write(src.join("a.txt"), "hello")?;

    let expected = dir.path().join("_concat-src.txt");

    let mut cmd = cargo_bin_cmd!("concat");
    cmd.current_dir(dir.path())
        .args(["-t", "src"])
        .assert()
        .success();

    let out = fs::read_to_string(&expected)?;

    assert!(out.contains("# File Metadata (1 files)"));
    assert!(out.contains("(lines: 1, chars: 5)"));
    Ok(())
}

#[test]
fn metadata_can_be_disabled() -> anyhow::Result<()> {
    let dir = non_hidden_tempdir()?;
    let src = dir.path().join("src");
    fs::create_dir_all(&src)?;
    fs::write(src.join("a.txt"), "hello\n")?;

    let expected = dir.path().join("_concat-src.xml");

    let mut cmd = cargo_bin_cmd!("concat");
    cmd.current_dir(dir.path())
        .args(["--no-metadata", "src"])
        .assert()
        .success();

    let out = fs::read_to_string(&expected)?;

    assert!(!out.contains("<fileMetadata"));
    Ok(())
}

#[test]
fn ext_filter_sets_output_name() -> anyhow::Result<()> {
    let dir = non_hidden_tempdir()?;
    let src = dir.path().join("src");
    fs::create_dir_all(&src)?;
    fs::write(src.join("a.rs"), "fn a() {}\n")?;
    fs::write(src.join("b.txt"), "b\n")?;

    let expected = dir.path().join("_concat-rs.xml");

    let mut cmd = cargo_bin_cmd!("concat");
    cmd.current_dir(dir.path())
        .args(["-x", "rs", "src"])
        .assert()
        .success();

    let out = fs::read_to_string(&expected)?;
    assert!(out.contains("<fileContents count=\"1\">"));
    Ok(())
}

#[test]
fn exclude_filename_is_applied_anywhere_in_tree() -> anyhow::Result<()> {
    let dir = non_hidden_tempdir()?;
    let src = dir.path().join("src");
    fs::create_dir_all(&src)?;
    let keep = src.join("keep.rs");
    let skip = src.join("skip.rs");

    fs::write(&keep, "keep\n")?;
    fs::write(&skip, "skip\n")?;

    let expected = dir.path().join("_concat-rs.xml");

    let mut cmd = cargo_bin_cmd!("concat");
    cmd.current_dir(dir.path())
        .args(["-x", "rs", "-e", "skip.rs", "src"])
        .assert()
        .success();

    let out = fs::read_to_string(&expected)?;
    let keep_abs = fs::canonicalize(&keep)?.to_string_lossy().to_string();
    let skip_abs = fs::canonicalize(&skip)?.to_string_lossy().to_string();

    assert!(out.contains(&keep_abs));
    assert!(!out.contains(&skip_abs));
    Ok(())
}

#[test]
fn include_binary_writes_base64_marker_in_text() -> anyhow::Result<()> {
    let dir = non_hidden_tempdir()?;
    let src = dir.path().join("src");
    fs::create_dir_all(&src)?;

    fs::write(src.join("a.txt"), "hello\n")?;
    fs::write(src.join("bin.dat"), b"\0\0\0binary")?;

    let expected = dir.path().join("_concat-src.txt");

    let mut cmd = cargo_bin_cmd!("concat");
    cmd.current_dir(dir.path())
        .args(["-t", "-b", "src"])
        .assert()
        .success();

    let out = fs::read_to_string(&expected)?;
    assert!(out.contains("[BINARY FILE: base64]"));
    Ok(())
}

#[test]
fn include_glob_can_select_hidden_files_without_hidden_flag() -> anyhow::Result<()> {
    let dir = non_hidden_tempdir()?;
    let src = dir.path().join("src");
    fs::create_dir_all(&src)?;

    let hidden = src.join(".env");
    fs::write(&hidden, "SECRET=1\n")?;

    let expected = dir.path().join("_concat-src.xml");

    let mut cmd = cargo_bin_cmd!("concat");
    cmd.current_dir(dir.path())
        .args(["-I", "**/.env", "src"])
        .assert()
        .success();

    let out = fs::read_to_string(&expected)?;
    let hidden_abs = fs::canonicalize(&hidden)?.to_string_lossy().to_string();
    assert!(out.contains(&hidden_abs));
    Ok(())
}

#[test]
fn clean_subcommand_respects_extension_filter() -> anyhow::Result<()> {
    let dir = non_hidden_tempdir()?;
    let a = dir.path().join("_concat-a.xml");
    let b = dir.path().join("_concat-b.txt");
    fs::write(&a, "a")?;
    fs::write(&b, "b")?;

    let mut cmd = cargo_bin_cmd!("concat");
    cmd.current_dir(dir.path())
        .args(["clean", "-n", "-x", "txt"])
        .assert()
        .success();

    assert!(a.exists());
    assert!(!b.exists());
    Ok(())
}
