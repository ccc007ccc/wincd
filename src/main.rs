use anyhow::Result;
use clap::Parser;
use owo_colors::OwoColorize;
use std::io::Write;
use std::path::PathBuf;

use wincd::clipboard;
use wincd::converter::{Converter, Direction};
use wincd::resolver;
use wincd::shell;

/// WSL 下一步到位的 Windows 路径导航工具
///
/// 将 Windows 路径（如 C:\Users\foo）转换为 WSL 路径（/mnt/c/Users/foo）。
/// 不传路径参数时自动从剪贴板读取。
#[derive(Parser, Debug)]
#[command(name = "wincd", version, about, long_about = None)]
struct Cli {
    /// Windows 路径，省略则从剪贴板读取
    path: Option<String>,

    /// 反向转换：WSL → Windows
    #[arg(short = 'w', long = "to-windows")]
    to_windows: bool,

    /// 输出 Windows 路径但用 / 分隔
    #[arg(short = 'm', long = "mixed")]
    mixed: bool,

    /// 自动向上查找存在的父目录
    #[arg(short = 'p', long = "parent")]
    parent: bool,

    /// 跳过路径存在性检查
    #[arg(short = 'f', long = "force")]
    force: bool,

    /// 显示转换详情
    #[arg(short = 'v', long = "verbose")]
    verbose: bool,

    /// 输出 shell 集成代码（bash / zsh / fish）
    #[arg(long = "init", value_name = "SHELL")]
    init: Option<String>,

    /// 一键配置 shell 集成和补全（自动检测当前 shell）
    #[arg(long = "setup")]
    setup: bool,

    /// 卸载：移除 shell 集成、补全脚本和二进制
    #[arg(long = "uninstall")]
    uninstall: bool,

    /// 禁用彩色输出
    #[arg(long = "no-color")]
    no_color: bool,
}

fn main() {
    if let Err(e) = run() {
        eprintln!("{} {}", "错误:".red().bold(), e);
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();

    // 处理 --init
    if let Some(shell_name) = &cli.init {
        let sh: shell::Shell = shell_name.parse().map_err(|e: String| anyhow::anyhow!(e))?;
        print!("{}", shell::init_script(sh));
        return Ok(());
    }

    // 处理 --setup
    if cli.setup {
        return do_setup();
    }

    // 处理 --uninstall
    if cli.uninstall {
        return do_uninstall();
    }

    // 获取输入路径：参数 或 剪贴板
    let input = match &cli.path {
        Some(p) => p.clone(),
        None => clipboard::read_clipboard_path()?,
    };

    let converter = Converter::new();

    if cli.to_windows {
        // WSL → Windows
        let result = converter.to_windows(&input, cli.mixed)?;
        if cli.verbose {
            print_verbose(&result);
        }
        println!("{}", result.converted);
    } else {
        // Windows → WSL
        let result = converter.to_wsl(&input)?;
        if cli.verbose {
            print_verbose(&result);
        }

        // 路径解析
        let resolved = resolver::resolve_path(&result.converted, cli.parent, cli.force);

        if !resolved.exact && !cli.force {
            // 路径不存在，给出提示
            eprintln!(
                "{} 路径不存在: {}",
                "警告:".yellow().bold(),
                resolved.path.display()
            );
            if !resolved.suggestions.is_empty() {
                eprintln!("{}", "可能的目录:".cyan());
                for s in &resolved.suggestions {
                    eprintln!("  {}", s.display());
                }
            }
        }

        // 输出最终路径（供 shell wrapper 的 cd 使用）
        println!("{}", resolved.path.display());
    }

    Ok(())
}

/// 一键配置 shell 集成和补全
fn do_setup() -> Result<()> {
    // 检测当前 shell
    let shell_path = std::env::var("SHELL").unwrap_or_default();
    let sh = if shell_path.contains("zsh") {
        shell::Shell::Zsh
    } else if shell_path.contains("fish") {
        shell::Shell::Fish
    } else {
        shell::Shell::Bash
    };

    let home = dirs::home_dir().ok_or_else(|| anyhow::anyhow!("无法获取 home 目录"))?;

    eprintln!("{} 检测到 shell: {:?}", "信息:".green().bold(), sh);

    // 写入 shell 集成到 rc 文件
    let (rc_file, marker) = match sh {
        shell::Shell::Bash => (home.join(".bashrc"), "# wincd integration"),
        shell::Shell::Zsh => (home.join(".zshrc"), "# wincd integration"),
        shell::Shell::Fish => (
            home.join(".config/fish/conf.d/wincd.fish"),
            "# wincd integration",
        ),
    };

    let init_code = shell::init_script(sh);

    // 读取现有内容，检查是否已配置
    let existing = std::fs::read_to_string(&rc_file).unwrap_or_default();
    if existing.contains(marker) {
        eprintln!(
            "{} {} 已配置，跳过",
            "信息:".green().bold(),
            rc_file.display()
        );
    } else {
        // 确保父目录存在
        if let Some(parent) = rc_file.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&rc_file)?;

        writeln!(file, "\n{}", marker)?;
        writeln!(file, "{}", init_code)?;
        eprintln!(
            "{} 已写入 shell 集成到 {}",
            "成功:".green().bold(),
            rc_file.display()
        );
    }

    // 安装补全脚本
    let (completion_src, completion_dest) = match sh {
        shell::Shell::Bash => {
            let d = home.join(".local/share/bash-completion/completions/wincd");
            ("completions/wincd.bash", d)
        }
        shell::Shell::Zsh => {
            let d = home.join(".zfunc/_wincd");
            ("completions/wincd.zsh", d)
        }
        shell::Shell::Fish => {
            let d = home.join(".config/fish/completions/wincd.fish");
            ("completions/wincd.fish", d)
        }
    };

    // 尝试从二进制同目录查找补全文件
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()))
        .unwrap_or_else(|| PathBuf::from("."));

    let completion_path = exe_dir.join(completion_src);

    if completion_path.exists() {
        if let Some(parent) = completion_dest.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::copy(&completion_path, &completion_dest)?;
        eprintln!(
            "{} 已安装补全脚本到 {}",
            "成功:".green().bold(),
            completion_dest.display()
        );
    } else {
        eprintln!(
            "{} 补全脚本 {} 未找到（不影响使用）",
            "提示:".yellow().bold(),
            completion_src
        );
    }

    eprintln!();
    eprintln!("{} 请运行以下命令使配置生效:", "提示:".cyan().bold());
    match sh {
        shell::Shell::Fish => eprintln!("  source {}", rc_file.display()),
        _ => eprintln!("  source {}", rc_file.display()),
    }
    eprintln!();
    eprintln!("之后就可以直接使用 wcd 命令了:");
    eprintln!("  wcd 'C:\\code\\Rust'");

    Ok(())
}

/// 卸载：移除 shell 集成、补全脚本、二进制
fn do_uninstall() -> Result<()> {
    let shell_path = std::env::var("SHELL").unwrap_or_default();
    let sh = if shell_path.contains("zsh") {
        shell::Shell::Zsh
    } else if shell_path.contains("fish") {
        shell::Shell::Fish
    } else {
        shell::Shell::Bash
    };

    let home = dirs::home_dir().ok_or_else(|| anyhow::anyhow!("无法获取 home 目录"))?;

    eprintln!("{} 检测到 shell: {:?}", "信息:".green().bold(), sh);

    // 移除 rc 文件中的集成代码
    let rc_file = match sh {
        shell::Shell::Bash => home.join(".bashrc"),
        shell::Shell::Zsh => home.join(".zshrc"),
        shell::Shell::Fish => home.join(".config/fish/conf.d/wincd.fish"),
    };

    if rc_file.exists() {
        let content = std::fs::read_to_string(&rc_file)?;
        let marker = "# wincd integration";
        if let Some(start) = content.find(marker) {
            // 找到 marker 位置，向前找到段落开头（空行）
            let before = &content[..start];
            let trim_end = before.trim_end().len();
            // 从 marker 开始向下找到段落结尾（下一个空行或文件结尾）
            let after_marker = &content[start..];
            let end_offset = after_marker
                .find("\n\n")
                .map(|i| i + 2)
                .unwrap_or(after_marker.len());

            let new_content = format!("{}{}", &content[..trim_end], &after_marker[end_offset..]);
            std::fs::write(&rc_file, new_content)?;
            eprintln!(
                "{} 已从 {} 移除 shell 集成",
                "成功:".green().bold(),
                rc_file.display()
            );
        } else {
            eprintln!(
                "{} {} 中未找到 wincd 集成代码",
                "提示:".yellow().bold(),
                rc_file.display()
            );
        }
    }

    // 移除补全脚本
    let completion_file = match sh {
        shell::Shell::Bash => home.join(".local/share/bash-completion/completions/wincd"),
        shell::Shell::Zsh => home.join(".zfunc/_wincd"),
        shell::Shell::Fish => home.join(".config/fish/completions/wincd.fish"),
    };

    if completion_file.exists() {
        std::fs::remove_file(&completion_file)?;
        eprintln!(
            "{} 已移除补全脚本 {}",
            "成功:".green().bold(),
            completion_file.display()
        );
    }

    // 移除二进制
    let exe = std::env::current_exe().ok();
    if let Some(exe_path) = &exe {
        // 确认二进制在 ~/.local/bin 下，避免误删
        if exe_path.starts_with(home.join(".local/bin")) {
            eprintln!(
                "{} 发现二进制: {}",
                "信息:".green().bold(),
                exe_path.display()
            );
            eprint!("是否删除二进制文件? [y/N] ");
            std::io::stderr().flush()?;

            let mut input = String::new();
            std::io::stdin().read_line(&mut input)?;
            if input.trim().to_lowercase() == "y" {
                // 不能删除正在运行的二进制（Linux 上可以 unlink），先尝试
                std::fs::remove_file(exe_path)?;
                eprintln!(
                    "{} 已删除二进制 {}",
                    "成功:".green().bold(),
                    exe_path.display()
                );
            } else {
                eprintln!("{} 跳过删除二进制", "提示:".yellow().bold());
            }
        }
    }

    eprintln!();
    eprintln!("{} 卸载完成", "成功:".green().bold());
    eprintln!("请运行以下命令使变更生效:");
    match sh {
        shell::Shell::Fish => eprintln!("  source {}", rc_file.display()),
        _ => eprintln!("  source {}", rc_file.display()),
    }

    Ok(())
}

/// 打印详细转换信息
fn print_verbose(result: &wincd::converter::ConvertResult) {
    let direction = match result.direction {
        Direction::ToWsl => "Windows → WSL",
        Direction::ToWindows => "WSL → Windows",
    };
    eprintln!("{} {}", "方向:".dimmed(), direction.dimmed());
    eprintln!("{} {}", "原始:".dimmed(), result.original.dimmed());
    eprintln!("{} {}", "转换:".dimmed(), result.converted.green());
    eprintln!();
}
