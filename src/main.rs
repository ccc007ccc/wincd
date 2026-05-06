use anyhow::Result;
use clap::Parser;
use owo_colors::OwoColorize;

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
        let sh = shell::Shell::from_str(shell_name)
            .ok_or_else(|| anyhow::anyhow!("不支持的 shell: {}（支持 bash/zsh/fish）", shell_name))?;
        print!("{}", shell::init_script(sh));
        return Ok(());
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
