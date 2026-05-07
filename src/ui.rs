//! 输出与颜色控制
//!
//! 统一处理着色输出，响应 `--no-color`、`NO_COLOR` 环境变量和非 TTY 自动关闭。

use owo_colors::OwoColorize;
use std::io::IsTerminal;

/// 初始化颜色输出策略。
///
/// 关闭着色的条件:
/// - 显式传入 `force_no_color = true`
/// - 设置了 `NO_COLOR` 环境变量(任意值)
/// - stdout 或 stderr 不是 TTY(被管道/重定向)
pub fn init_color(force_no_color: bool) {
    let no_color = force_no_color
        || std::env::var_os("NO_COLOR").is_some()
        || !std::io::stdout().is_terminal()
        || !std::io::stderr().is_terminal();
    if no_color {
        owo_colors::set_override(false);
    }
}

/// 信息提示(stderr)
pub fn info(msg: impl AsRef<str>) {
    eprintln!("{} {}", "[信息]".green().bold(), msg.as_ref());
}

/// 成功提示(stderr)
pub fn ok(msg: impl AsRef<str>) {
    eprintln!("{} {}", "[成功]".green().bold(), msg.as_ref());
}

/// 警告(stderr)
pub fn warn(msg: impl AsRef<str>) {
    eprintln!("{} {}", "[警告]".yellow().bold(), msg.as_ref());
}

/// 错误(stderr)
pub fn err(msg: impl AsRef<str>) {
    eprintln!("{} {}", "[错误]".red().bold(), msg.as_ref());
}

/// 提示(stderr)
pub fn hint(msg: impl AsRef<str>) {
    eprintln!("{} {}", "[提示]".cyan().bold(), msg.as_ref());
}
