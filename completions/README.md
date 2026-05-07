# Completions

Completions are now **generated dynamically** via `clap_complete` from the live CLI definition.

## Get a completion script

```bash
wincd completions bash       # → stdout
wincd completions zsh
wincd completions fish
wincd completions powershell
```

## Install (handled automatically by `wincd install`)

`wincd install` writes the completion to:

| Shell      | Path                                                       |
|------------|------------------------------------------------------------|
| bash       | `~/.local/share/bash-completion/completions/wincd`         |
| zsh        | `~/.zfunc/_wincd`                                          |
| fish       | `~/.config/fish/completions/wincd.fish`                    |
| powershell | `~/.config/powershell/wincd-completion.ps1`                |

For `wcd` (the shell function added by `wincd install`), the same completion is reused via `compdef wcd=wincd` (zsh), `complete -F _wincd wcd` (bash), or `complete -c wcd -w 'wincd convert'` (fish).
