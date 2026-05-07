//! `wincd convert` — 默认行为:Windows ↔ WSL 路径转换
//!
//! 同时也是顶层 args(无子命令)的实际执行体。

use anyhow::Result;
use owo_colors::OwoColorize;

use crate::clipboard;
use crate::converter::{Converter, Direction};
use crate::resolver;
use crate::ui;

#[derive(Debug, Default)]
pub struct ConvertArgs {
    /// 输入路径(None 时从剪贴板读)
    pub path: Option<String>,
    /// 反向转换(WSL → Windows)
    pub to_windows: bool,
    /// 输出 Windows 路径但用 / 分隔
    pub mixed: bool,
    /// 自动向上找父目录
    pub parent: bool,
    /// 跳过路径存在性检查
    pub force: bool,
    /// 显示转换详情(到 stderr)
    pub verbose: bool,
}

/// 退出码
pub mod exit {
    pub const OK: i32 = 0;
    pub const CONVERT_ERR: i32 = 2;
    pub const PATH_NOT_EXIST: i32 = 3;
    pub const CLIPBOARD_ERR: i32 = 4;
}

/// 执行转换。返回退出码:0 = 成功;2 = 转换失败;3 = 路径不存在(非强制);4 = 剪贴板错误。
pub fn run(args: ConvertArgs) -> Result<i32> {
    let input = match args.path {
        Some(ref p) => p.clone(),
        None => match clipboard::read_clipboard_path() {
            Ok(s) => s,
            Err(e) => {
                ui::err(format!("{}", e));
                return Ok(exit::CLIPBOARD_ERR);
            }
        },
    };

    let converter = Converter::new();

    if args.to_windows {
        let result = match converter.to_windows(&input, args.mixed) {
            Ok(r) => r,
            Err(e) => {
                ui::err(format!("{}", e));
                return Ok(exit::CONVERT_ERR);
            }
        };
        if args.verbose {
            print_verbose(&result.original, &result.converted, result.direction);
        }
        println!("{}", result.converted);
        return Ok(exit::OK);
    }

    let result = match converter.to_wsl(&input) {
        Ok(r) => r,
        Err(e) => {
            ui::err(format!("{}", e));
            return Ok(exit::CONVERT_ERR);
        }
    };

    if args.verbose {
        print_verbose(&result.original, &result.converted, result.direction);
    }

    let resolved = resolver::resolve_path(&result.converted, args.parent, args.force);
    let mut exit_code = exit::OK;

    if !resolved.exact && !args.force {
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
        // 路径不存在不影响 wcd shell 集成 cd 的尝试,但 exit code 反映状态
        exit_code = exit::PATH_NOT_EXIST;
    }

    println!("{}", resolved.path.display());
    Ok(exit_code)
}

fn print_verbose(original: &str, converted: &str, direction: Direction) {
    let dir = match direction {
        Direction::ToWsl => "Windows → WSL",
        Direction::ToWindows => "WSL → Windows",
    };
    eprintln!("{} {}", "方向:".dimmed(), dir.dimmed());
    eprintln!("{} {}", "原始:".dimmed(), original.dimmed());
    eprintln!("{} {}", "转换:".dimmed(), converted.green());
    eprintln!();
}
