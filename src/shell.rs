/// Shell 类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Shell {
    Bash,
    Zsh,
    Fish,
}

impl std::str::FromStr for Shell {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "bash" => Ok(Shell::Bash),
            "zsh" => Ok(Shell::Zsh),
            "fish" => Ok(Shell::Fish),
            _ => Err(format!("不支持的 shell: {}（支持 bash/zsh/fish）", s)),
        }
    }
}

/// 输出 shell 集成代码
pub fn init_script(shell: Shell) -> &'static str {
    match shell {
        Shell::Bash => BASH_INIT,
        Shell::Zsh => ZSH_INIT,
        Shell::Fish => FISH_INIT,
    }
}

/// 生成 cd 命令字符串
pub fn cd_command(path: &str) -> String {
    format!("cd {}", shell_escape(path))
}

/// 简单的 shell 转义：如果路径含空格或特殊字符则加引号
fn shell_escape(path: &str) -> String {
    if path.contains(' ')
        || path.contains('(')
        || path.contains(')')
        || path.contains('\'')
        || path.contains('"')
    {
        // 用单引号包裹，内部的单引号用 '\'' 转义
        let escaped = path.replace('\'', "'\\''");
        format!("'{}'", escaped)
    } else {
        path.to_string()
    }
}

const BASH_INIT: &str = r#"
# wincd shell 集成 — 在 bash 中使用 wcd 命令直接 cd
wcd() {
    local path
    if [ $# -eq 0 ]; then
        path="$(command wincd)" || return 1
    else
        path="$(command wincd "$@")" || return 1
    fi
    cd "$path"
}
# wincd integration end
"#;

const ZSH_INIT: &str = r#"
# wincd shell 集成 — 在 zsh 中使用 wcd 命令直接 cd
wcd() {
    local path
    if [ $# -eq 0 ]; then
        path="$(command wincd)" || return 1
    else
        path="$(command wincd "$@")" || return 1
    fi
    cd "$path"
}
# wincd integration end
"#;

const FISH_INIT: &str = r#"
# wincd shell 集成 — 在 fish 中使用 wcd 命令直接 cd
function wcd
    if test (count $argv) -eq 0
        set path (command wincd); or return 1
    else
        set path (command wincd $argv); or return 1
    end
    cd $path
end
# wincd integration end
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shell_from_str() {
        assert_eq!("bash".parse::<Shell>().unwrap(), Shell::Bash);
        assert_eq!("ZSH".parse::<Shell>().unwrap(), Shell::Zsh);
        assert_eq!("Fish".parse::<Shell>().unwrap(), Shell::Fish);
        assert!("powershell".parse::<Shell>().is_err());
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
    fn test_cd_command_with_quote() {
        assert_eq!(
            cd_command("/mnt/c/user's dir"),
            "cd '/mnt/c/user'\\''s dir'"
        );
    }

    #[test]
    fn test_init_script_not_empty() {
        assert!(!init_script(Shell::Bash).is_empty());
        assert!(!init_script(Shell::Zsh).is_empty());
        assert!(!init_script(Shell::Fish).is_empty());
    }
}
