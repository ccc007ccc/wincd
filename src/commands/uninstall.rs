//! `wincd uninstall` — 移除 shell 集成、补全脚本,可选删除二进制
//!
//! 安全策略:默认列出将要执行的所有操作并要求 y/N 确认;`--yes` 跳过确认。
//! 这是为了避免子命令撞名/误触导致的意外破坏(例如剪贴板里恰好是 "uninstall")。

use anyhow::{Context, Result};
use std::fs;
use std::io::{IsTerminal, Write};

use crate::shell::{self, Shell};
use crate::ui;

#[derive(Debug, Default)]
pub struct UninstallArgs {
    /// 显式指定 shell;None 时自动检测
    pub shell: Option<Shell>,
    /// 跳过所有确认提示
    pub yes: bool,
    /// 移除所有支持的 shell(而非仅当前)
    pub all_shells: bool,
    /// 不删除二进制
    pub keep_binary: bool,
}

#[derive(Debug)]
enum Action {
    StripRc { rc: std::path::PathBuf },
    RemoveCompletion { path: std::path::PathBuf },
    RemoveBinary { path: std::path::PathBuf },
}

impl Action {
    fn describe(&self) -> String {
        match self {
            Action::StripRc { rc } => format!("从 {} 移除 wincd 集成块", rc.display()),
            Action::RemoveCompletion { path } => format!("删除补全脚本 {}", path.display()),
            Action::RemoveBinary { path } => format!("删除二进制 {}", path.display()),
        }
    }
}

pub fn run(args: UninstallArgs) -> Result<()> {
    let home = dirs::home_dir().context("无法获取 home 目录")?;

    let shells: Vec<Shell> = if args.all_shells {
        vec![Shell::Bash, Shell::Zsh, Shell::Fish, Shell::PowerShell]
    } else {
        vec![args.shell.unwrap_or_else(shell::detect_current_shell)]
    };

    // 1. 收集将要执行的所有操作(干跑)
    let mut actions: Vec<Action> = Vec::new();
    for sh in &shells {
        let rc_file = shell::rc_file_path(&home, *sh);
        if rc_file.exists() {
            let content = fs::read_to_string(&rc_file).unwrap_or_default();
            if shell::find_existing_block(&content).is_some() {
                actions.push(Action::StripRc { rc: rc_file });
            }
        }
        let completion_path = shell::completion_file_path(&home, *sh);
        if completion_path.exists() {
            actions.push(Action::RemoveCompletion {
                path: completion_path,
            });
        }
    }
    if !args.keep_binary {
        if let Ok(exe) = std::env::current_exe() {
            let safe_dirs = [home.join(".local/bin"), home.join(".cargo/bin")];
            if safe_dirs.iter().any(|d| exe.starts_with(d)) {
                actions.push(Action::RemoveBinary { path: exe });
            }
        }
    }

    if actions.is_empty() {
        ui::info("没有可卸载的内容(rc 集成块、补全脚本、二进制均未找到)");
        return Ok(());
    }

    // 2. 显示并确认
    ui::warn("即将执行以下操作:");
    for a in &actions {
        eprintln!("  • {}", a.describe());
    }
    eprintln!();

    if !args.yes {
        if !std::io::stdin().is_terminal() {
            ui::err("非交互式环境检测到,且未指定 --yes,已中止以避免误触");
            anyhow::bail!("需要 --yes 才能在非交互环境执行 uninstall");
        }
        if !confirm("确认继续卸载吗?")? {
            ui::info("已取消,未做任何修改");
            return Ok(());
        }
    }

    // 3. 真正执行
    for action in actions {
        match action {
            Action::StripRc { rc } => {
                let content = fs::read_to_string(&rc)
                    .with_context(|| format!("读取失败: {}", rc.display()))?;
                if let Some(new_content) = shell::strip_block(&content) {
                    fs::write(&rc, new_content)
                        .with_context(|| format!("写入失败: {}", rc.display()))?;
                    ui::ok(format!("已从 {} 移除集成代码", rc.display()));
                }
            }
            Action::RemoveCompletion { path } => {
                fs::remove_file(&path).with_context(|| format!("删除失败: {}", path.display()))?;
                ui::ok(format!("已删除补全 {}", path.display()));
            }
            Action::RemoveBinary { path } => match fs::remove_file(&path) {
                Ok(_) => ui::ok(format!("已删除二进制 {}", path.display())),
                Err(e) => ui::warn(format!("无法删除 {}: {}(可手动 rm)", path.display(), e)),
            },
        }
    }

    eprintln!();
    ui::ok("卸载完成");
    Ok(())
}

fn confirm(prompt: &str) -> Result<bool> {
    eprint!("{} [y/N] ", prompt);
    std::io::stderr().flush()?;
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    Ok(input.trim().eq_ignore_ascii_case("y"))
}
