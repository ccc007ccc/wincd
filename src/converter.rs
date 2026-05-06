use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConvertError {
    #[error("无法识别路径格式: {0}")]
    UnrecognizedFormat(String),
    #[error("无效的盘符: {0}")]
    InvalidDrive(char),
    #[error("UNC 路径格式不正确: {0}")]
    InvalidUnc(String),
}

/// 转换方向
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    /// Windows → WSL
    ToWsl,
    /// WSL → Windows
    ToWindows,
}

/// 转换结果
#[derive(Debug, Clone)]
pub struct ConvertResult {
    pub original: String,
    pub converted: String,
    pub direction: Direction,
}

/// 核心转换器
pub struct Converter {
    /// WSL 挂载前缀，默认 /mnt
    mount_prefix: String,
}

impl Converter {
    pub fn new() -> Self {
        Self {
            mount_prefix: detect_mount_prefix(),
        }
    }

    #[cfg(test)]
    pub fn with_mount_prefix(prefix: &str) -> Self {
        Self {
            mount_prefix: prefix.to_string(),
        }
    }

    /// 将 Windows 路径转换为 WSL 路径
    pub fn to_wsl(&self, input: &str) -> Result<ConvertResult, ConvertError> {
        let trimmed = clean_path_input(input);

        // 已经是 WSL 路径，直接返回
        if trimmed.starts_with('/') {
            return Ok(ConvertResult {
                original: input.to_string(),
                converted: trimmed.to_string(),
                direction: Direction::ToWsl,
            });
        }

        // ~ 展开
        if trimmed == "~" || trimmed.starts_with("~/") {
            let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/home"));
            let rest = trimmed.strip_prefix('~').unwrap_or("");
            return Ok(ConvertResult {
                original: input.to_string(),
                converted: format!("{}{}", home.display(), rest),
                direction: Direction::ToWsl,
            });
        }

        // UNC 路径 \\wsl$\... 或 \\wsl.localhost\...
        if trimmed.starts_with("\\\\") || trimmed.starts_with("//") {
            return self.convert_unc(&trimmed, input);
        }

        // Windows 盘符路径 X:\... 或 X:/...
        if let Some(drive) = extract_drive_letter(&trimmed) {
            return self.convert_drive_path(drive, &trimmed, input);
        }

        Err(ConvertError::UnrecognizedFormat(input.to_string()))
    }

    /// 将 WSL 路径转换为 Windows 路径
    pub fn to_windows(&self, input: &str, mixed: bool) -> Result<ConvertResult, ConvertError> {
        let trimmed = clean_path_input(input);

        if !trimmed.starts_with('/') {
            return Err(ConvertError::UnrecognizedFormat(input.to_string()));
        }

        // /mnt/x/... → X:\... 或 X:/...
        let prefix = format!("{}/", self.mount_prefix);
        if trimmed.starts_with(&prefix) || trimmed == self.mount_prefix {
            let rest = trimmed
                .strip_prefix(&prefix)
                .or_else(|| trimmed.strip_prefix(&self.mount_prefix))
                .unwrap_or("");
            if !rest.is_empty() {
                let drive_char = rest
                    .split('/')
                    .next()
                    .unwrap_or("")
                    .chars()
                    .next()
                    .ok_or_else(|| ConvertError::UnrecognizedFormat(input.to_string()))?;
                if drive_char.is_ascii_alphabetic() {
                    let path_part = rest
                        .strip_prefix(drive_char)
                        .unwrap_or("")
                        .trim_start_matches('/');
                    let sep = if mixed { "/" } else { "\\" };
                    let win_path = format!(
                        "{}:{}{}",
                        drive_char.to_uppercase(),
                        sep,
                        path_part.replace('/', sep)
                    );
                    return Ok(ConvertResult {
                        original: input.to_string(),
                        converted: win_path,
                        direction: Direction::ToWindows,
                    });
                }
            }
        }

        // /home/user/... → \\wsl$\DISTRO\home\user\...
        let distro = detect_distro_name();
        let win_path = if mixed {
            format!("//wsl$/{}/{}", distro, trimmed.trim_start_matches('/'))
        } else {
            format!(
                "\\\\wsl$\\{}\\{}",
                distro,
                trimmed.trim_start_matches('/').replace('/', "\\")
            )
        };

        Ok(ConvertResult {
            original: input.to_string(),
            converted: win_path,
            direction: Direction::ToWindows,
        })
    }

    /// 转换 UNC 路径
    fn convert_unc(&self, cleaned: &str, original: &str) -> Result<ConvertResult, ConvertError> {
        let normalized = cleaned.replace('\\', "/");
        let parts: Vec<&str> = normalized.trim_start_matches('/').split('/').collect();

        if parts.len() < 2 {
            return Err(ConvertError::InvalidUnc(original.to_string()));
        }

        let server = parts[0];

        // \\wsl$\DISTRO\path 或 \\wsl.localhost\DISTRO\path
        if server == "wsl$" || server == "wsl.localhost" {
            if parts.len() < 3 {
                return Err(ConvertError::InvalidUnc(original.to_string()));
            }
            // parts[1] = distro, parts[2..] = path
            let path = parts[2..].join("/");
            return Ok(ConvertResult {
                original: original.to_string(),
                converted: format!("/{}", path),
                direction: Direction::ToWsl,
            });
        }

        // 普通 UNC \\server\share\path → /mnt/unc/server/share/path
        let unc_path = parts.join("/");
        Ok(ConvertResult {
            original: original.to_string(),
            converted: format!("{}/unc/{}", self.mount_prefix, unc_path),
            direction: Direction::ToWsl,
        })
    }

    /// 转换盘符路径
    fn convert_drive_path(
        &self,
        drive: char,
        cleaned: &str,
        original: &str,
    ) -> Result<ConvertResult, ConvertError> {
        // 跳过盘符和冒号，如 "C:" → 跳过 2 字符
        let rest = &cleaned[2..];
        let rest = rest.replace('\\', "/");
        let rest = rest.trim_start_matches('/');

        let wsl_path = format!("{}/{}/{}", self.mount_prefix, drive.to_lowercase(), rest);
        Ok(ConvertResult {
            original: original.to_string(),
            converted: wsl_path,
            direction: Direction::ToWsl,
        })
    }
}

impl Default for Converter {
    fn default() -> Self {
        Self::new()
    }
}

/// 清理路径输入：去除首尾空白、引号
pub fn clean_path_input(input: &str) -> String {
    let mut s = input.trim();
    // 去除成对的引号
    while (s.starts_with('"') && s.ends_with('"')) || (s.starts_with('\'') && s.ends_with('\'')) {
        s = &s[1..s.len() - 1];
    }
    s.trim().to_string()
}

/// 提取盘符字母（如 C:\ 中的 'C'）
fn extract_drive_letter(path: &str) -> Option<char> {
    let bytes = path.as_bytes();
    if bytes.len() >= 2 && bytes[0].is_ascii_alphabetic() && bytes[1] == b':' {
        Some(bytes[0].to_ascii_lowercase() as char)
    } else {
        None
    }
}

/// 检测 WSL 挂载前缀
fn detect_mount_prefix() -> String {
    // 读取 /etc/wsl.conf 中的 [automount] root 配置
    if let Ok(content) = std::fs::read_to_string("/etc/wsl.conf") {
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("root") && trimmed.contains('=') {
                if let Some(val) = trimmed.split('=').nth(1) {
                    let val = val.trim().trim_matches('"').trim_matches('/');
                    if !val.is_empty() {
                        return format!("/{}", val);
                    }
                }
            }
        }
    }
    "/mnt".to_string()
}

/// 检测当前 WSL 发行版名称
fn detect_distro_name() -> String {
    // 方法1：读取 /etc/os-release
    if let Ok(content) = std::fs::read_to_string("/etc/os-release") {
        for line in content.lines() {
            if let Some(val) = line.strip_prefix("PRETTY_NAME=") {
                return val.trim_matches('"').to_string();
            }
        }
    }
    // 方法2：环境变量
    std::env::var("WSL_DISTRO_NAME").unwrap_or_else(|_| "Ubuntu".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_drive_path() {
        let c = Converter::with_mount_prefix("/mnt");
        let r = c.to_wsl("C:\\Users\\foo").unwrap();
        assert_eq!(r.converted, "/mnt/c/Users/foo");
    }

    #[test]
    fn test_forward_slash() {
        let c = Converter::with_mount_prefix("/mnt");
        let r = c.to_wsl("C:/Users/foo").unwrap();
        assert_eq!(r.converted, "/mnt/c/Users/foo");
    }

    #[test]
    fn test_uppercase_drive() {
        let c = Converter::with_mount_prefix("/mnt");
        let r = c.to_wsl("D:\\Projects").unwrap();
        assert_eq!(r.converted, "/mnt/d/Projects");
    }

    #[test]
    fn test_already_wsl_path() {
        let c = Converter::with_mount_prefix("/mnt");
        let r = c.to_wsl("/home/user").unwrap();
        assert_eq!(r.converted, "/home/user");
    }

    #[test]
    fn test_unc_wsl_path() {
        let c = Converter::with_mount_prefix("/mnt");
        let r = c.to_wsl("\\\\wsl$\\Ubuntu\\home\\user").unwrap();
        assert_eq!(r.converted, "/home/user");
    }

    #[test]
    fn test_unc_wsl_localhost() {
        let c = Converter::with_mount_prefix("/mnt");
        let r = c
            .to_wsl("\\\\wsl.localhost\\Ubuntu-22.04\\home\\user\\projects")
            .unwrap();
        assert_eq!(r.converted, "/home/user/projects");
    }

    #[test]
    fn test_unc_network_share() {
        let c = Converter::with_mount_prefix("/mnt");
        let r = c.to_wsl("\\\\server\\share\\file.txt").unwrap();
        assert_eq!(r.converted, "/mnt/unc/server/share/file.txt");
    }

    #[test]
    fn test_quoted_path() {
        let c = Converter::with_mount_prefix("/mnt");
        let r = c.to_wsl("\"C:\\Users\\foo\"").unwrap();
        assert_eq!(r.converted, "/mnt/c/Users/foo");
    }

    #[test]
    fn test_single_quoted_path() {
        let c = Converter::with_mount_prefix("/mnt");
        let r = c.to_wsl("'C:\\Users\\foo'").unwrap();
        assert_eq!(r.converted, "/mnt/c/Users/foo");
    }

    #[test]
    fn test_leading_trailing_spaces() {
        let c = Converter::with_mount_prefix("/mnt");
        let r = c.to_wsl("  C:\\Users\\foo  ").unwrap();
        assert_eq!(r.converted, "/mnt/c/Users/foo");
    }

    #[test]
    fn test_to_windows_basic() {
        let c = Converter::with_mount_prefix("/mnt");
        let r = c.to_windows("/mnt/c/Users/foo", false).unwrap();
        assert_eq!(r.converted, "C:\\Users\\foo");
    }

    #[test]
    fn test_to_windows_mixed() {
        let c = Converter::with_mount_prefix("/mnt");
        let r = c.to_windows("/mnt/c/Users/foo", true).unwrap();
        assert_eq!(r.converted, "C:/Users/foo");
    }

    #[test]
    fn test_custom_mount_prefix() {
        let c = Converter::with_mount_prefix("/drv");
        let r = c.to_wsl("C:\\Users\\foo").unwrap();
        assert_eq!(r.converted, "/drv/c/Users/foo");
    }

    #[test]
    fn test_drive_only() {
        let c = Converter::with_mount_prefix("/mnt");
        let r = c.to_wsl("C:\\").unwrap();
        assert_eq!(r.converted, "/mnt/c/");
    }

    #[test]
    fn test_clean_path_input() {
        assert_eq!(clean_path_input("  \"C:\\foo\"  "), "C:\\foo");
        assert_eq!(clean_path_input("'C:\\foo'"), "C:\\foo");
        assert_eq!(clean_path_input("  C:\\foo  "), "C:\\foo");
    }

    #[test]
    fn test_extract_drive_letter() {
        assert_eq!(extract_drive_letter("C:\\foo"), Some('c'));
        assert_eq!(extract_drive_letter("d:/bar"), Some('d'));
        assert_eq!(extract_drive_letter("/home"), None);
        assert_eq!(extract_drive_letter("foo"), None);
    }
}
