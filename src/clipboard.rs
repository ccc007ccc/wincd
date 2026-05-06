use crate::converter::clean_path_input;
use anyhow::{Context, Result};

/// 从系统剪贴板读取文本内容
pub fn read_clipboard() -> Result<String> {
    let mut ctx = arboard::Clipboard::new().context("无法访问系统剪贴板")?;
    let text = ctx
        .get_text()
        .context("无法读取剪贴板文本")?;
    Ok(text)
}

/// 读取剪贴板并清理为可用路径
pub fn read_clipboard_path() -> Result<String> {
    let raw = read_clipboard()?;
    let cleaned = clean_path_input(&raw);

    if cleaned.is_empty() {
        anyhow::bail!("剪贴板内容为空");
    }

    // 简单检查是否像路径：包含 \ 或 / 或 : 或以字母开头
    let looks_like_path = cleaned.contains('\\')
        || cleaned.contains('/')
        || (cleaned.len() >= 2 && cleaned.as_bytes()[1] == b':');

    if !looks_like_path {
        anyhow::bail!(
            "剪贴板内容不像路径: {}",
            truncate(&cleaned, 60)
        );
    }

    Ok(cleaned)
}

/// 截断字符串，超过长度加省略号
fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len])
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
        assert_eq!(truncate(&s, 10).len(), 13); // "aaaaaaaaaa..."
    }
}
