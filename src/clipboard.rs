//! 剪贴板读取
//!
//! 优先使用 `arboard` 跨平台读取;在 WSL 无 X11/WSLg 等环境下,自动 fallback 到调用
//! `powershell.exe Get-Clipboard` 获取 Windows 剪贴板内容。

use crate::converter::clean_path_input;
use anyhow::{Context, Result};
use std::process::Command;

/// 从系统剪贴板读取文本
pub fn read_clipboard() -> Result<String> {
    match read_via_arboard() {
        Ok(s) => Ok(s),
        Err(e_arboard) => match read_via_powershell() {
            Ok(s) => Ok(s),
            Err(e_pwsh) => Err(anyhow::anyhow!(
                "无法访问剪贴板:\n  arboard: {}\n  powershell.exe: {}",
                e_arboard,
                e_pwsh
            )),
        },
    }
}

fn read_via_arboard() -> Result<String> {
    let mut ctx = arboard::Clipboard::new().context("初始化 arboard 失败")?;
    let text = ctx.get_text().context("arboard 读取剪贴板失败")?;
    Ok(text)
}

/// 调用 powershell.exe Get-Clipboard 读取(WSL interop)
fn read_via_powershell() -> Result<String> {
    let candidates = [
        "powershell.exe",
        "/mnt/c/Windows/System32/WindowsPowerShell/v1.0/powershell.exe",
    ];
    let mut last_err: Option<String> = None;
    for cmd in candidates {
        match Command::new(cmd)
            .args([
                "-NoProfile",
                "-NonInteractive",
                "-Command",
                "Get-Clipboard -Raw",
            ])
            .output()
        {
            Ok(out) if out.status.success() => {
                let s = String::from_utf8_lossy(&out.stdout).to_string();
                return Ok(s);
            }
            Ok(out) => {
                last_err = Some(format!(
                    "{} 退出码 {:?}: {}",
                    cmd,
                    out.status.code(),
                    String::from_utf8_lossy(&out.stderr)
                ));
            }
            Err(e) => {
                last_err = Some(format!("{} 启动失败: {}", cmd, e));
            }
        }
    }
    Err(anyhow::anyhow!(
        "powershell.exe 不可用: {}",
        last_err.unwrap_or_default()
    ))
}

/// 读取剪贴板并清理为可用路径(失败时 anyhow::bail!)
pub fn read_clipboard_path() -> Result<String> {
    let raw = read_clipboard()?;
    let first_line = raw.lines().next().unwrap_or("").to_string();
    let cleaned = clean_path_input(&first_line);

    if cleaned.is_empty() {
        anyhow::bail!("剪贴板内容为空");
    }

    if !looks_like_path(&cleaned) {
        anyhow::bail!("剪贴板内容不像路径: {}", truncate(&cleaned, 60));
    }

    Ok(cleaned)
}

/// 严格判断是否像路径
fn looks_like_path(s: &str) -> bool {
    let bytes = s.as_bytes();
    // 盘符 + 分隔符:C:\ 或 C:/
    if bytes.len() >= 3
        && bytes[0].is_ascii_alphabetic()
        && bytes[1] == b':'
        && (bytes[2] == b'/' || bytes[2] == b'\\')
    {
        return true;
    }
    // UNC
    if s.starts_with("\\\\") || s.starts_with("//") {
        return true;
    }
    // POSIX 绝对路径
    if s.starts_with('/') {
        return true;
    }
    // ~ 展开
    if s.starts_with('~') {
        return true;
    }
    // 含分隔符且不太短
    if (s.contains('\\') || s.contains('/')) && s.len() >= 3 {
        return true;
    }
    false
}

fn truncate(s: &str, max_len: usize) -> String {
    if s.chars().count() <= max_len {
        s.to_string()
    } else {
        let mut out: String = s.chars().take(max_len).collect();
        out.push_str("...");
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_truncate_short() {
        assert_eq!(truncate("abc", 10), "abc");
    }

    #[test]
    fn test_truncate_long() {
        let s = "a".repeat(100);
        assert_eq!(truncate(&s, 10), "aaaaaaaaaa...");
    }

    #[test]
    fn test_looks_like_path_drive() {
        assert!(looks_like_path(r"C:\foo"));
        assert!(looks_like_path("D:/bar"));
    }

    #[test]
    fn test_looks_like_path_unc() {
        assert!(looks_like_path(r"\\server\share"));
        assert!(looks_like_path("//server/share"));
    }

    #[test]
    fn test_looks_like_path_posix() {
        assert!(looks_like_path("/home/user"));
        assert!(looks_like_path("~/foo"));
    }

    #[test]
    fn test_looks_like_path_negative() {
        assert!(!looks_like_path("hello world"));
        assert!(!looks_like_path("just text"));
        assert!(!looks_like_path("a:"));
        assert!(!looks_like_path("ab"));
    }

    #[test]
    fn test_looks_like_path_relative_with_separator() {
        assert!(looks_like_path("foo/bar"));
        assert!(looks_like_path(r"a\b"));
    }
}
