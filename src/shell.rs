//! Shell 类型与集成代码生成

use std::path::{Path, PathBuf};

/// 写入 rc 文件时使用的标记。开始/结束 marker 必须互不为前缀。
pub const BEGIN_MARKER: &str = "# >>> wincd initialize >>>";
pub const END_MARKER: &str = "# <<< wincd initialize <<<";

#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum Shell {
    Bash,
    Zsh,
    Fish,
    #[value(name = "powershell", aliases = ["pwsh", "ps1"])]
    PowerShell,
}

impl Shell {
    pub fn name(self) -> &'static str {
        match self {
            Shell::Bash => "bash",
            Shell::Zsh => "zsh",
            Shell::Fish => "fish",
            Shell::PowerShell => "powershell",
        }
    }

    pub fn rc_relpath(self) -> &'static str {
        match self {
            Shell::Bash => ".bashrc",
            Shell::Zsh => ".zshrc",
            Shell::Fish => ".config/fish/conf.d/wincd.fish",
            Shell::PowerShell => ".config/powershell/Microsoft.PowerShell_profile.ps1",
        }
    }

    pub fn completion_relpath(self) -> &'static str {
        match self {
            Shell::Bash => ".local/share/bash-completion/completions/wincd",
            Shell::Zsh => ".zfunc/_wincd",
            Shell::Fish => ".config/fish/completions/wincd.fish",
            Shell::PowerShell => ".config/powershell/wincd-completion.ps1",
        }
    }
}

impl std::str::FromStr for Shell {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "bash" => Ok(Shell::Bash),
            "zsh" => Ok(Shell::Zsh),
            "fish" => Ok(Shell::Fish),
            "powershell" | "pwsh" | "ps1" => Ok(Shell::PowerShell),
            _ => Err(format!(
                "不支持的 shell: {}(支持 bash/zsh/fish/powershell)",
                s
            )),
        }
    }
}

pub fn detect_current_shell() -> Shell {
    if let Ok(shell_path) = std::env::var("SHELL") {
        let s = shell_path.to_lowercase();
        if s.contains("zsh") {
            return Shell::Zsh;
        }
        if s.contains("fish") {
            return Shell::Fish;
        }
        if s.contains("pwsh") || s.contains("powershell") {
            return Shell::PowerShell;
        }
        return Shell::Bash;
    }
    Shell::Bash
}

pub fn init_script(shell: Shell) -> String {
    let body = match shell {
        Shell::Bash => BASH_BODY,
        Shell::Zsh => ZSH_BODY,
        Shell::Fish => FISH_BODY,
        Shell::PowerShell => POWERSHELL_BODY,
    };
    format!("{}\n{}\n{}\n", BEGIN_MARKER, body.trim(), END_MARKER)
}

pub fn cd_command(path: &str) -> String {
    format!("cd {}", posix_quote(path))
}

/// POSIX shell 单引号转义
pub fn posix_quote(s: &str) -> String {
    if s.is_empty() {
        return "''".to_string();
    }
    let safe = s
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '/' | '_' | '-' | '.' | '@' | ':' | ','));
    if safe {
        return s.to_string();
    }
    let escaped = s.replace('\'', "'\\''");
    format!("'{}'", escaped)
}

pub fn find_existing_block(content: &str) -> Option<(usize, usize)> {
    let begin = content.find(BEGIN_MARKER)?;
    let after_begin = begin + BEGIN_MARKER.len();
    let end_rel = content[after_begin..].find(END_MARKER)?;
    let end = after_begin + end_rel + END_MARKER.len();
    Some((begin, end))
}

pub fn strip_block(content: &str) -> Option<String> {
    let (start, end) = find_existing_block(content)?;
    let before = content[..start].trim_end();
    let after = content[end..].trim_start();
    let mut result = String::with_capacity(content.len());
    result.push_str(before);
    if !before.is_empty() && !after.is_empty() {
        result.push_str("\n\n");
    }
    result.push_str(after);
    if !result.is_empty() && !result.ends_with('\n') {
        result.push('\n');
    }
    Some(result)
}

pub fn rc_file_path(home: &Path, shell: Shell) -> PathBuf {
    home.join(shell.rc_relpath())
}

pub fn completion_file_path(home: &Path, shell: Shell) -> PathBuf {
    home.join(shell.completion_relpath())
}

const BASH_BODY: &str = r#"
# wincd shell 集成 — 在 bash 中使用 wcd 命令直接 cd 到 Windows 路径
wcd() {
    local _wincd_target
    if [ "$#" -eq 0 ]; then
        _wincd_target="$(command wincd convert 2>/dev/null)"
        if [ -z "$_wincd_target" ] && [ -t 0 ] && [ -t 2 ]; then
            local _wincd_input
            read -r -e -p "wincd> 输入 Windows 路径: " _wincd_input || return 1
            [ -z "$_wincd_input" ] && return 1
            _wincd_target="$(command wincd convert -- "$_wincd_input")" || return 1
        elif [ -z "$_wincd_target" ]; then
            command wincd convert 2>&1
            return 1
        fi
    else
        _wincd_target="$(command wincd convert -- "$@")" || return 1
    fi
    builtin cd -- "$_wincd_target"
}
"#;

const ZSH_BODY: &str = r#"
# wincd shell 集成 — 在 zsh 中使用 wcd 命令直接 cd 到 Windows 路径
wcd() {
    emulate -L zsh
    local _wincd_target
    if (( $# == 0 )); then
        _wincd_target="$(command wincd convert 2>/dev/null)"
        if [[ -z "$_wincd_target" && -t 0 && -t 2 ]]; then
            local _wincd_input
            read -r "_wincd_input?wincd> 输入 Windows 路径: " || return 1
            [[ -z "$_wincd_input" ]] && return 1
            _wincd_target="$(command wincd convert -- "$_wincd_input")" || return 1
        elif [[ -z "$_wincd_target" ]]; then
            command wincd convert 2>&1
            return 1
        fi
    else
        _wincd_target="$(command wincd convert -- "$@")" || return 1
    fi
    builtin cd -- "$_wincd_target"
}
"#;

const FISH_BODY: &str = r#"
# wincd shell 集成 — 在 fish 中使用 wcd 命令直接 cd 到 Windows 路径
function wcd --description 'cd to a Windows path via wincd'
    set -l _wincd_target
    if test (count $argv) -eq 0
        set _wincd_target (command wincd convert 2>/dev/null)
        if test -z "$_wincd_target"; and isatty stdin; and isatty stderr
            read -P "wincd> 输入 Windows 路径: " _wincd_input
            or return 1
            test -z "$_wincd_input"; and return 1
            set _wincd_target (command wincd convert -- $_wincd_input)
            or return 1
        else if test -z "$_wincd_target"
            command wincd convert 2>&1
            return 1
        end
    else
        set _wincd_target (command wincd convert -- $argv)
        or return 1
    end
    builtin cd -- $_wincd_target
end
"#;

const POWERSHELL_BODY: &str = r#"
# wincd shell 集成 — 在 PowerShell 中使用 wcd 函数直接 Set-Location
function wcd {
    [CmdletBinding()]
    param(
        [Parameter(ValueFromRemainingArguments=$true)]
        [string[]]$WincdArgs
    )
    $target = $null
    if (-not $WincdArgs -or $WincdArgs.Count -eq 0) {
        $target = & wincd convert 2>$null
        if ([string]::IsNullOrEmpty($target)) {
            $userInput = Read-Host "wincd> 输入 Windows 路径"
            if ([string]::IsNullOrEmpty($userInput)) { return }
            $target = & wincd convert -- $userInput
            if ($LASTEXITCODE -ne 0) { return }
        }
    } else {
        $target = & wincd convert -- @WincdArgs
        if ($LASTEXITCODE -ne 0) { return }
    }
    Set-Location -LiteralPath $target
}
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shell_from_str() {
        assert_eq!("bash".parse::<Shell>().unwrap(), Shell::Bash);
        assert_eq!("ZSH".parse::<Shell>().unwrap(), Shell::Zsh);
        assert_eq!("Fish".parse::<Shell>().unwrap(), Shell::Fish);
        assert_eq!("powershell".parse::<Shell>().unwrap(), Shell::PowerShell);
        assert_eq!("pwsh".parse::<Shell>().unwrap(), Shell::PowerShell);
        assert!("nushell".parse::<Shell>().is_err());
    }

    #[test]
    fn test_posix_quote() {
        assert_eq!(posix_quote("/foo/bar"), "/foo/bar");
        assert_eq!(posix_quote("/has space"), "'/has space'");
        assert_eq!(posix_quote("a'b"), "'a'\\''b'");
        assert_eq!(posix_quote(""), "''");
        assert_eq!(posix_quote("foo$bar"), "'foo$bar'");
        assert_eq!(posix_quote("foo`bar"), "'foo`bar'");
    }

    #[test]
    fn test_init_script_has_markers() {
        for sh in [Shell::Bash, Shell::Zsh, Shell::Fish, Shell::PowerShell] {
            let s = init_script(sh);
            assert!(s.contains(BEGIN_MARKER), "shell={:?}", sh);
            assert!(s.contains(END_MARKER), "shell={:?}", sh);
            assert!(s.find(BEGIN_MARKER).unwrap() < s.find(END_MARKER).unwrap());
        }
    }

    #[test]
    fn test_markers_not_prefix_of_each_other() {
        assert!(!BEGIN_MARKER.contains(END_MARKER));
        assert!(!END_MARKER.contains(BEGIN_MARKER));
    }

    #[test]
    fn test_strip_block_round_trip() {
        let original = "export PATH=/foo";
        let with_block = format!(
            "{}\n{}\necho done\n",
            original,
            init_script(Shell::Bash).trim_end()
        );
        let stripped = strip_block(&with_block).unwrap();
        assert!(!stripped.contains(BEGIN_MARKER));
        assert!(!stripped.contains(END_MARKER));
        assert!(stripped.contains("export PATH=/foo"));
        assert!(stripped.contains("echo done"));
    }

    #[test]
    fn test_strip_block_returns_none_if_absent() {
        assert!(strip_block("just plain content\n").is_none());
    }

    #[test]
    fn test_cd_command_simple() {
        assert_eq!(cd_command("/mnt/c/Users"), "cd /mnt/c/Users");
    }

    #[test]
    fn test_cd_command_with_spaces() {
        assert_eq!(
            cd_command("/mnt/c/Program Files"),
            "cd '/mnt/c/Program Files'"
        );
    }

    #[test]
    fn test_rc_paths() {
        let home = PathBuf::from("/home/user");
        assert_eq!(rc_file_path(&home, Shell::Bash), home.join(".bashrc"));
        assert_eq!(
            rc_file_path(&home, Shell::Fish),
            home.join(".config/fish/conf.d/wincd.fish")
        );
    }
}
