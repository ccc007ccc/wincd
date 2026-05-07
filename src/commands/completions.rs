//! `wincd completions <SHELL>` — 输出补全脚本到 stdout
//!
//! 用 clap_complete 在运行时根据 CLI 定义生成,自动覆盖所有选项/子命令,
//! 并同时为 `wcd` 函数注册补全(POSIX shell)。

use crate::cli::Cli;
use crate::shell::Shell;
use anyhow::Result;
use clap::CommandFactory;
use clap_complete::Shell as ClapShell;
use std::io::Write;

pub fn run(shell: Shell) -> Result<()> {
    let mut cmd = Cli::command();
    let stdout = std::io::stdout();
    let mut handle = stdout.lock();

    let alias = match shell {
        Shell::Bash => {
            clap_complete::generate(ClapShell::Bash, &mut cmd, "wincd", &mut handle);
            BASH_WCD_ALIAS
        }
        Shell::Zsh => {
            clap_complete::generate(ClapShell::Zsh, &mut cmd, "wincd", &mut handle);
            ZSH_WCD_ALIAS
        }
        Shell::Fish => {
            clap_complete::generate(ClapShell::Fish, &mut cmd, "wincd", &mut handle);
            FISH_WCD_ALIAS
        }
        Shell::PowerShell => {
            clap_complete::generate(ClapShell::PowerShell, &mut cmd, "wincd", &mut handle);
            // PowerShell 中 wcd 是函数,自然继承补全;无需额外注册
            ""
        }
    };

    if !alias.is_empty() {
        writeln!(handle)?;
        writeln!(handle, "{}", alias)?;
    }
    Ok(())
}

const BASH_WCD_ALIAS: &str = "# 让 wcd 共用 wincd 的补全\ncomplete -F _wincd -o filenames wcd";
const ZSH_WCD_ALIAS: &str = "# 让 wcd 共用 wincd 的补全\ncompdef wcd=wincd";
const FISH_WCD_ALIAS: &str =
    "# 让 wcd 共用 wincd 的补全(fish 用 wraps)\ncomplete -c wcd -w 'wincd convert'";
