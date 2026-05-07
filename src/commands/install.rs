//! `wincd install` — 一键配置 shell 集成 + 补全脚本
//!
//! 安全策略:
//! - 默认幂等 — 已配置时跳过(显示提示)
//! - `--force` 覆盖现有集成块,在交互式终端会先要求确认(防误触)
//! - `--yes` 在 `--force` 时跳过确认,适配 CI/脚本

use anyhow::{Context, Result};
use std::fs;
use std::io::{IsTerminal, Write};

use crate::cli::Cli;
use crate::shell::{self, Shell};
use crate::ui;
use clap::CommandFactory;
use clap_complete::Shell as ClapShell;

#[derive(Debug, Default)]
pub struct InstallArgs {
    /// 显式指定 shell;None 时自动检测
    pub shell: Option<Shell>,
    /// 强制覆盖已存在的集成块
    pub force: bool,
    /// 跳过 --force 时的确认提示
    pub yes: bool,
}

pub fn run(args: InstallArgs) -> Result<()> {
    let sh = args.shell.unwrap_or_else(shell::detect_current_shell);
    let home = dirs::home_dir().context("无法获取 home 目录")?;

    ui::info(format!("检测到 shell: {}", sh.name()));

    // 1. 写入 shell 集成
    let rc_file = shell::rc_file_path(&home, sh);
    let init_code = shell::init_script(sh);
    let existing = fs::read_to_string(&rc_file).unwrap_or_default();

    let already_installed = shell::find_existing_block(&existing).is_some();

    if already_installed && !args.force {
        ui::info(format!(
            "{} 已包含集成代码,跳过(--force 可覆盖)",
            rc_file.display()
        ));
    } else {
        if already_installed && args.force && !args.yes && std::io::stdin().is_terminal() {
            ui::warn(format!(
                "将覆盖 {} 中已有的 wincd 集成块",
                rc_file.display()
            ));
            if !confirm("确认覆盖?")? {
                ui::info("已取消,未做任何修改");
                return Ok(());
            }
        }

        if let Some(parent) = rc_file.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("创建目录失败: {}", parent.display()))?;
        }

        let new_content = if already_installed {
            let stripped = shell::strip_block(&existing).unwrap_or(existing.clone());
            let mut s = stripped.trim_end().to_string();
            if !s.is_empty() {
                s.push_str("\n\n");
            }
            s.push_str(&init_code);
            s
        } else {
            let mut s = existing.clone();
            if !s.is_empty() && !s.ends_with('\n') {
                s.push('\n');
            }
            if !s.is_empty() {
                s.push('\n');
            }
            s.push_str(&init_code);
            s
        };

        fs::write(&rc_file, new_content)
            .with_context(|| format!("写入失败: {}", rc_file.display()))?;
        ui::ok(format!("已写入 shell 集成 → {}", rc_file.display()));
    }

    // 2. 写入补全脚本(直接生成,不依赖外部文件)
    let completion_path = shell::completion_file_path(&home, sh);
    if let Some(parent) = completion_path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("创建补全目录失败: {}", parent.display()))?;
    }

    let mut file = fs::File::create(&completion_path)
        .with_context(|| format!("创建补全文件失败: {}", completion_path.display()))?;
    let mut cmd = Cli::command();
    let clap_shell = match sh {
        Shell::Bash => ClapShell::Bash,
        Shell::Zsh => ClapShell::Zsh,
        Shell::Fish => ClapShell::Fish,
        Shell::PowerShell => ClapShell::PowerShell,
    };
    clap_complete::generate(clap_shell, &mut cmd, "wincd", &mut file);
    let alias = match sh {
        Shell::Bash => "\n# wcd 复用 wincd 补全\ncomplete -F _wincd -o filenames wcd\n",
        Shell::Zsh => "\n# wcd 复用 wincd 补全\ncompdef wcd=wincd\n",
        Shell::Fish => "\n# wcd 复用 wincd 补全\ncomplete -c wcd -w 'wincd convert'\n",
        Shell::PowerShell => "",
    };
    if !alias.is_empty() {
        file.write_all(alias.as_bytes())?;
    }
    file.flush()?;
    ui::ok(format!("已安装补全 → {}", completion_path.display()));

    eprintln!();
    ui::hint("请运行以下命令使配置立即生效:");
    match sh {
        Shell::Fish => eprintln!("  source {}", rc_file.display()),
        Shell::Zsh => {
            eprintln!("  source {}", rc_file.display());
            eprintln!("  # 若 zsh 未加载 .zfunc,请确保 .zshrc 中包含:");
            eprintln!("  #   fpath=(~/.zfunc $fpath); autoload -Uz compinit && compinit");
        }
        Shell::PowerShell => {
            eprintln!("  . $PROFILE");
            eprintln!("  # 若需补全立即生效,执行:");
            eprintln!("  #   . {}", completion_path.display());
        }
        Shell::Bash => eprintln!("  source {}", rc_file.display()),
    }
    eprintln!();
    eprintln!("之后可直接使用 wcd 命令(无参数从剪贴板/交互输入,有参数转发):");
    eprintln!(r"  wcd 'C:\code\Rust'");
    eprintln!("  wcd                # 从剪贴板读,失败则提示输入");

    Ok(())
}

fn confirm(prompt: &str) -> Result<bool> {
    eprint!("{} [y/N] ", prompt);
    std::io::stderr().flush()?;
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    Ok(input.trim().eq_ignore_ascii_case("y"))
}
