use std::io::Write;
use std::process::{Command, Stdio};

use anyhow::{Context, Result, bail};

pub fn copy_to_clipboard(content: &str) -> Result<()> {
    #[cfg(target_os = "macos")]
    {
        return copy_with_command("pbcopy", &[], content);
    }

    #[cfg(target_os = "linux")]
    {
        if copy_with_command("wl-copy", &[], content).is_ok() {
            return Ok(());
        }

        return copy_with_command("xclip", &["-selection", "clipboard"], content)
            .or_else(|_| copy_with_command("xsel", &["--clipboard", "--input"], content))
            .context("No clipboard utility found. Install wl-copy, xclip, or xsel.");
    }

    #[cfg(target_os = "windows")]
    {
        return copy_with_command("cmd", &["/C", "clip"], content);
    }

    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    {
        bail!("Clipboard copying is not supported on this platform.");
    }
}

fn copy_with_command(cmd: &str, args: &[&str], content: &str) -> Result<()> {
    let mut child = Command::new(cmd)
        .args(args)
        .stdin(Stdio::piped())
        .spawn()
        .with_context(|| format!("Failed to spawn clipboard command '{cmd}'"))?;

    if let Some(stdin) = child.stdin.as_mut() {
        stdin
            .write_all(content.as_bytes())
            .context("Failed to write data to clipboard process")?;
    } else {
        bail!("Clipboard command '{cmd}' does not accept stdin.");
    }

    let status = child.wait()?;

    if !status.success() {
        bail!("Clipboard command '{cmd}' exited with status {status}");
    }

    Ok(())
}
