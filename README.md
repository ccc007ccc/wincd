# wincd

> One-step Windows path navigation for WSL — paste a Windows path, cd instantly

[![CI](https://github.com/ccc007ccc/wincd/actions/workflows/ci.yml/badge.svg)](https://github.com/ccc007ccc/wincd/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/wincd)](https://crates.io/crates/wincd)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE-MIT)

[中文文档](README.zh-CN.md)

## What it does

Convert Windows paths into WSL paths (or vice versa), and `cd` to them in one step.

```bash
wcd 'C:\Users\foo\Documents\Projects'    # → cd /mnt/c/Users/foo/Documents/Projects
wcd                                       # ← read from clipboard, fallback to interactive prompt
wincd -w /mnt/c/Users/foo                 # → C:\Users\foo (reverse)
```

## Features

- **Multiple path formats**: `C:\`, `C:/`, `\wsl$\…`, `\wsl.localhost\…`, `\server\share\…`, mixed separators, `~/…`
- **Clipboard integration**: reads Windows clipboard when no args given; falls back to `powershell.exe Get-Clipboard` when `arboard` can't initialize (e.g. no WSLg/X11)
- **Interactive prompt**: when the clipboard isn't a path, `wcd` reads a line from stdin via `read -r` — bypasses shell escape entirely
- **Direct `cd`**: shell wrappers for bash / zsh / fish / PowerShell
- **Suggestions on miss**: when the path doesn't exist, lists similar siblings ranked by Jaro-Winkler
- **One-command setup/uninstall**: `wincd install` / `wincd uninstall`
- **Distro-aware**: detects `WSL_DISTRO_NAME` and `[automount] root` from `/etc/wsl.conf`

## Installation

### One-line install

```bash
curl -fsSL https://raw.githubusercontent.com/ccc007ccc/wincd/main/install.sh | sh
```

The script:
1. Detects architecture (`amd64` / `arm64`)
2. Downloads the binary from GitHub Releases
3. **Verifies SHA256** against the published checksum (set `WINCD_VERIFY=0` to skip)
4. Runs `wincd install` automatically (set `WINCD_NO_SETUP=1` to skip)

Pin a specific version:

```bash
curl -fsSL https://raw.githubusercontent.com/ccc007ccc/wincd/main/install.sh | WINCD_VERSION=v2.0.0 sh
```

### From source / cargo

```bash
cargo install wincd          # or: git clone … && cargo install --path .
wincd install
```

## CLI

`wincd` works in two equivalent ways — top-level args (compat), and explicit subcommands (preferred for scripts).

```
# Default behavior — same as `wincd convert`
wincd [OPTIONS] [PATH]

# Explicit subcommands
wincd convert      [OPTIONS] [PATH]      # convert path
wincd init         <SHELL>               # print integration code (for `eval "$(wincd init bash)"`)
wincd install      [--shell SHELL] [--force] [-y]
wincd uninstall    [--shell SHELL] [--all-shells] [--keep-binary] [-y]
wincd completions  <SHELL>               # print completion script
```

`<SHELL>`: `bash` | `zsh` | `fish` | `powershell` (aliases: `pwsh`, `ps1`).

### convert flags

| Flag                 | Meaning                                                |
|----------------------|--------------------------------------------------------|
| `-w, --to-windows`   | Reverse: WSL → Windows                                 |
| `-m, --mixed`        | Use `/` separator in Windows output                    |
| `-p, --parent`       | Walk up to nearest existing parent if path is missing  |
| `-f, --force`        | Skip existence check (always treat path as valid)      |
| `-v, --verbose`      | Show conversion details on stderr                      |
| `--no-color`         | Disable color output (also: `NO_COLOR` env var)        |

### Legacy flags (still work)

`--init <SHELL>` ↔ `wincd init <SHELL>`
`--setup`        ↔ `wincd install`
`--uninstall`    ↔ `wincd uninstall`

## Why `wcd 'C:\foo'` works but `wcd C:\foo` doesn't

bash and zsh interpret `\c`, `\R`, `\s` etc. **as escape sequences before the program ever sees them.** This is a shell limitation no tool can directly bypass. The `wcd` shell function provides three workarounds:

1. **Quote it explicitly**: `wcd 'C:\foo'` (single quotes prevent escaping)
2. **Use the clipboard**: `wcd` (no args) — reads Windows clipboard
3. **Interactive prompt**: `wcd` (when clipboard is empty/non-path) — reads a line from stdin via `read -r`, so the shell doesn't touch your backslashes

## Safety: `install` / `uninstall` confirmations

The subcommands `install` and `uninstall` have side effects (writing/removing files). To guard against typos and command-line mishaps, they default to **safe** behavior:

| Action                                        | Default behavior                                                |
|-----------------------------------------------|-----------------------------------------------------------------|
| `wincd install` (fresh)                       | Writes rc + completion. Idempotent — re-running is safe.        |
| `wincd install` (already configured)          | Skips, prints a hint to use `--force`.                          |
| `wincd install --force`                       | **Asks for `y/N` confirmation** in interactive terminal.        |
| `wincd install --force --yes`                 | Skips confirmation (use in CI / scripts).                       |
| `wincd uninstall`                             | **Lists every action it will take, asks for `y/N` confirmation.** |
| `wincd uninstall --yes`                       | Skips confirmation.                                             |
| `wincd uninstall` in non-TTY without `--yes`  | **Refuses to run.** Must pass `--yes` explicitly in pipelines.  |

Why so careful? Because `uninstall` will (by default) delete the binary at `~/.local/bin/wincd` or `~/.cargo/bin/wincd`. If a path you pasted ever happened to start with the literal token `uninstall` (very unlikely but possible), you'd want a chance to abort.

## Smart path lookup

```bash
# Path doesn't exist — wincd shows the nearest similar siblings
wincd 'C:\Users\foo\NonExistnt'
# warning: path does not exist
# possible directories:
#   /mnt/c/Users/foo/NonExistent      ← nearest match
#   /mnt/c/Users/foo/Documents
#   …

# -p walks up to the first existing parent automatically
wincd 'C:\Users\foo\NonExistent\deep\path' -p
# → /mnt/c/Users/foo
```

## Custom mount point

`/etc/wsl.conf`:
```ini
[automount]
root = /drv
```
→ wincd outputs `/drv/c/...` instead of `/mnt/c/...`.

## Distro detection (for reverse UNC paths)

For output like `\wsl$\<distro>\…`, wincd resolves `<distro>` in priority:

1. `WSL_DISTRO_NAME` environment variable (WSL injects this — most reliable, e.g. `Ubuntu-22.04`)
2. `/etc/os-release` `ID` + `VERSION_ID` (e.g. `ubuntu` + `22.04` → `Ubuntu-22.04`)
3. Default: `Ubuntu`

To override: `WSL_DISTRO_NAME=Debian wincd -w /home/user`.

## Troubleshooting

**`wcd: command not found` after install**
Re-source your rc: `source ~/.bashrc` (or open a new shell).

**Tab completion doesn't work**
- Bash: ensure `bash-completion` is installed and `~/.local/share/bash-completion/completions` is on the search path (most distros do this by default).
- Zsh: add to `~/.zshrc` (if not already):
  ```zsh
  fpath=(~/.zfunc $fpath)
  autoload -Uz compinit && compinit
  ```
- Fish: completions go into `~/.config/fish/completions/`, fish auto-loads them.

**Clipboard reading fails**
On WSL without WSLg/X11, `arboard` may fail to init. wincd then falls back to `powershell.exe Get-Clipboard`. If that also fails, ensure `powershell.exe` is on PATH (default in WSL).

**Wrong distro name in reverse UNC paths**
Set `WSL_DISTRO_NAME` explicitly.

**Want to dry-run uninstall**
Just run it without `--yes` — wincd lists every action it will take and asks for confirmation. Saying `n` is a clean no-op.

## License

[MIT](LICENSE-MIT)
