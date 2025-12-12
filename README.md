# concat-rs

Rust CLI implementation of `concat` from `concat-zsh`.

`concat` merges the contents of multiple files (and/or directories) into a single output file, for use as context in LLM prompts or general file aggregation.

## Install

Preferred (uses the repo `install` script with conflict warnings):

```sh
./install
```

Bypass prompts (for CI/non-interactive use):

```sh
./install --force
```

## Notes

- If you still have `concat-zsh` sourced as a Zsh function named `concat`, it will shadow the Rust binary. Use `command concat ...` or remove the function from your shell config.

## Usage

```sh
concat [OPTIONS] [FILE|DIR|GLOB...]
concat clean [OPTIONS] [DIR...]
```

If no inputs are provided, `concat` defaults to `.`.

## Output

- Default output format is **XML**.
- Use `-t, --text` for plain text output.
- Output filenames default to `_concat-*` unless `-o, --output` is provided.
- A metadata header (line and character counts per file) is included by default; disable with `-M, --no-metadata`.

### Output filename logic (when `--output` is not set)

- `-x <ext>` once: `_concat-<ext>.xml` (or `.txt` with `--text`)
- `-x` multiple times: `_concat-output.xml`
- No args at all: `_concat-<cwd>.xml`
- One directory input: `_concat-<dir>.xml`
- Otherwise: `_concat-output.xml`

## Options (main command)

- `-o, --output <file>`: output file name
- `-r, --recursive`: search directories recursively (default)
- `-n, --no-recursive`: do not recurse
- `-t, --text`: plain text output (default XML)
- `-x, --ext <ext>` (repeatable): include only these extensions
- `-g, --ignore-ext <ext>` (repeatable): exclude these extensions
- `-I, --include <glob>` (repeatable): include only paths matching these globs
- `-e, -E, --exclude <glob>` (repeatable): exclude paths matching these globs
- `-T, --tree`: include a directory tree of the current directory in the output
- `-H, --hidden`: include hidden files/directories
- `-P, --no-purge-pycache`: do not remove `__pycache__` and `.pyc` in the current directory
- `-C, --no-clean-concat`: do not delete existing `_concat-*` files in the current directory before writing
- `-b, --include-binary`: include non-text files (encoded as base64)
- `-M, --no-metadata`: omit the per-file metadata header (line/character counts)
- `-l, --no-dir-list`: omit the matched directory list section (XML only)
- `-v, --verbose`: verbose logging
- `-d, --debug`: extra debug logging

### Hidden files + include globs

By default, hidden files are skipped. You can either:

- Pass `-H, --hidden`, or
- Use an `--include` glob that explicitly targets hidden paths (for example `**/.env`), without enabling full hidden traversal globally.

## `clean` subcommand

Deletes previously generated `_concat-*` files from the given directories (default: `.`). Searches recursively by default; use `-n` to disable recursion. Supports `-x/-g/-I/-e/-H` similarly to the main command.

## Contributing

Build locally:

```sh
cargo build
```

Build an optimized binary:

```sh
cargo build --release
```

Run checks:

```sh
cargo fmt
cargo clippy --all-targets --all-features -- -D warnings
cargo test
```
