# wincd

> One-step Windows path navigation for WSL — paste a Windows path, cd instantly

[![CI](https://github.com/ccc007ccc/wincd/actions/workflows/ci.yml/badge.svg)](https://github.com/ccc007ccc/wincd/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/wincd)](https://crates.io/crates/wincd)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE-MIT)

[中文文档](README.zh-CN.md)

## Why wincd?

Navigating Windows paths in WSL is painful:

```bash
# Traditional: type it all out
cd /mnt/c/Users/foo/Documents/Projects

# With wslpath: still need to compose the command
cd $(wslpath 'C:\Users\foo\Documents\Projects')

# With wincd: one step
wcd 'C:\Users\foo\Documents\Projects'
```

You can even read paths straight from the clipboard — copy a path in Windows Explorer with Ctrl+C, then just run `wcd` in WSL.

## Features

- **Multiple path formats**: `C:\`, `C:/`, `\\wsl$\`, `\\server\share`, mixed separators
- **Clipboard integration**: reads Windows clipboard when no args given
- **Direct cd**: shell integration for real directory switching
- **Path validation**: checks existence, suggests fuzzy matches
- **Reverse conversion**: WSL path → Windows path
- **One-command setup/uninstall**: `--setup` to configure, `--uninstall` to clean up
- **Pure Rust**: zero external dependencies, just compile and go

## Installation

### One-line install (recommended)

```bash
curl -fsSL https://raw.githubusercontent.com/ccc007ccc/wincd/main/install.sh | sh
```

The script downloads the binary and runs `--setup` automatically. Run `source ~/.bashrc` afterwards to use `wcd`.

### Download from GitHub Releases

Head to the [Releases](https://github.com/ccc007ccc/wincd/releases) page, then run `wincd --setup`.

### Build from source

```bash
git clone https://github.com/ccc007ccc/wincd.git
cd wincd
cargo build --release
cp target/release/wincd ~/.local/bin/
wincd --setup
```

### Via cargo

```bash
cargo install wincd
wincd --setup
```

## Quick Start

### Basic usage

```bash
# Convert a Windows path
wincd 'C:\Users\foo\Documents'
# Output: /mnt/c/Users/foo/Documents

# Forward slashes work too
wincd 'C:/Users/foo/Documents'
# Output: /mnt/c/Users/foo/Documents

# UNC paths
wincd '\\wsl$\Ubuntu\home\user'
# Output: /home/user
```

### Clipboard mode

```bash
# Copy a path in Windows Explorer, then:
wincd
# Reads clipboard and converts automatically
```

### Shell integration (recommended)

One-command setup — auto-detects your shell, writes integration code and completions:

```bash
wincd --setup
source ~/.bashrc  # or source ~/.zshrc
```

Then use `wcd` directly:

```bash
wcd 'C:\code\Rust'
# Switches to /mnt/c/code/Rust

wcd  # no args = read from clipboard
```

### Uninstall

```bash
wincd --uninstall
```

Removes shell integration, completions, and optionally the binary.

### Reverse conversion

```bash
# WSL → Windows
wincd -w /home/user/projects
# Output: C:\Users\...\home\user\projects

# Windows path with forward slashes
wincd -m /home/user/projects
# Output: C:/Users/.../home/user/projects
```

### Path not found

```bash
wincd 'C:\Users\foo\NonExistent'
# Warning: path does not exist: /mnt/c/Users/foo/NonExistent
# Possible directories:
#   /mnt/c/Users/foo/Documents
#   /mnt/c/Users/foo/Desktop
#   /mnt/c/Users/foo/Downloads

# Auto-find nearest existing parent
wincd -p 'C:\Users\foo\NonExistent\deep\path'
# Output: /mnt/c/Users/foo
```

## Full Usage

```
wincd [OPTIONS] [PATH]

Arguments:
  [PATH]  Windows path (reads from clipboard if omitted)

Options:
  -w, --to-windows    Reverse conversion: WSL → Windows
  -m, --mixed         Windows path with / separator
  -p, --parent        Find nearest existing parent directory
  -f, --force         Skip path existence check
  -v, --verbose       Show conversion details
  --init <SHELL>      Print shell integration code [bash, zsh, fish]
  --setup             One-command shell integration and completion setup
  --uninstall         Remove shell integration, completions, and binary
  --no-color          Disable colored output
  -h, --help          Show help
  -V, --version       Show version
```

## Custom mount points

If your WSL uses a custom mount prefix (configured in `/etc/wsl.conf`), wincd detects it automatically:

```ini
# /etc/wsl.conf
[automount]
root = /drv
```

wincd will use `/drv/c/...` instead of `/mnt/c/...`.

## License

[MIT](LICENSE-MIT)
