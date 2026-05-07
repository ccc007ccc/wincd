//! Windows ↔ WSL 路径转换核心

use std::path::PathBuf;
use std::sync::OnceLock;
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    ToWsl,
    ToWindows,
}

#[derive(Debug, Clone)]
pub struct ConvertResult {
    pub original: String,
    pub converted: String,
    pub direction: Direction,
}

pub struct Converter {
    mount_prefix: String,
}

impl Converter {
    pub fn new() -> Self {
        Self {
            mount_prefix: detect_mount_prefix().to_string(),
        }
    }

    pub fn with_mount_prefix(prefix: &str) -> Self {
        Self {
            mount_prefix: prefix.to_string(),
        }
    }

    pub fn mount_prefix(&self) -> &str {
        &self.mount_prefix
    }

    pub fn to_wsl(&self, input: &str) -> Result<ConvertResult, ConvertError> {
        let trimmed = clean_path_input(input);

        if trimmed.is_empty() {
            return Err(ConvertError::UnrecognizedFormat(input.to_string()));
        }

        if trimmed.starts_with('/') {
            return Ok(ConvertResult {
                original: input.to_string(),
                converted: trimmed,
                direction: Direction::ToWsl,
            });
        }

        if trimmed == "~" || trimmed.starts_with("~/") {
            let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/home"));
            let rest = trimmed.strip_prefix('~').unwrap_or("");
            return Ok(ConvertResult {
                original: input.to_string(),
                converted: format!("{}{}", home.display(), rest),
                direction: Direction::ToWsl,
            });
        }

        if trimmed.starts_with("\\\\") || trimmed.starts_with("//") {
            return self.convert_unc(&trimmed, input);
        }

        if let Some(drive) = extract_drive_letter(&trimmed) {
            return self.convert_drive_path(drive, &trimmed, input);
        }

        Err(ConvertError::UnrecognizedFormat(input.to_string()))
    }

    pub fn to_windows(&self, input: &str, mixed: bool) -> Result<ConvertResult, ConvertError> {
        let trimmed = clean_path_input(input);

        if !trimmed.starts_with('/') {
            return Err(ConvertError::UnrecognizedFormat(input.to_string()));
        }

        let prefix = format!("{}/", self.mount_prefix);
        if trimmed.starts_with(&prefix) || trimmed == self.mount_prefix {
            let rest = trimmed
                .strip_prefix(&prefix)
                .or_else(|| trimmed.strip_prefix(&self.mount_prefix))
                .unwrap_or("");
            if let Some(drive_char) = rest.chars().next() {
                if drive_char.is_ascii_alphabetic() {
                    let path_part = rest
                        .strip_prefix(drive_char)
                        .unwrap_or("")
                        .trim_start_matches('/');
                    let sep = if mixed { "/" } else { "\\" };
                    let win_path = format!(
                        "{}:{}{}",
                        drive_char.to_ascii_uppercase(),
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

        let distro = detect_distro_name();
        let body = trimmed.trim_start_matches('/');
        let win_path = if mixed {
            format!("//wsl$/{}/{}", distro, body)
        } else {
            format!("\\\\wsl$\\{}\\{}", distro, body.replace('/', "\\"))
        };

        Ok(ConvertResult {
            original: input.to_string(),
            converted: win_path,
            direction: Direction::ToWindows,
        })
    }

    fn convert_unc(&self, cleaned: &str, original: &str) -> Result<ConvertResult, ConvertError> {
        let normalized = cleaned.replace('\\', "/");
        let parts: Vec<&str> = normalized
            .trim_start_matches('/')
            .split('/')
            .filter(|p| !p.is_empty())
            .collect();

        if parts.len() < 2 {
            return Err(ConvertError::InvalidUnc(original.to_string()));
        }

        let server = parts[0];

        if server == "wsl$" || server.eq_ignore_ascii_case("wsl.localhost") {
            let path = if parts.len() >= 3 {
                parts[2..].join("/")
            } else {
                String::new()
            };
            return Ok(ConvertResult {
                original: original.to_string(),
                converted: format!("/{}", path),
                direction: Direction::ToWsl,
            });
        }

        let unc_path = parts.join("/");
        Ok(ConvertResult {
            original: original.to_string(),
            converted: format!("{}/unc/{}", self.mount_prefix, unc_path),
            direction: Direction::ToWsl,
        })
    }

    fn convert_drive_path(
        &self,
        drive: char,
        cleaned: &str,
        original: &str,
    ) -> Result<ConvertResult, ConvertError> {
        if cleaned.len() < 2 {
            return Err(ConvertError::InvalidDrive(drive));
        }
        let rest = &cleaned[2..];
        let rest = rest.replace('\\', "/");
        let rest = rest.trim_start_matches('/');

        let wsl_path = format!(
            "{}/{}/{}",
            self.mount_prefix,
            drive.to_ascii_lowercase(),
            rest
        );
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

pub fn clean_path_input(input: &str) -> String {
    let mut s = input.trim();
    while s.len() >= 2
        && ((s.starts_with('"') && s.ends_with('"')) || (s.starts_with('\'') && s.ends_with('\'')))
    {
        s = &s[1..s.len() - 1];
    }
    s.trim().to_string()
}

pub fn extract_drive_letter(path: &str) -> Option<char> {
    let bytes = path.as_bytes();
    if bytes.len() >= 2 && bytes[0].is_ascii_alphabetic() && bytes[1] == b':' {
        Some(bytes[0].to_ascii_lowercase() as char)
    } else {
        None
    }
}

static MOUNT_PREFIX: OnceLock<String> = OnceLock::new();
static DISTRO_NAME: OnceLock<String> = OnceLock::new();

fn detect_mount_prefix() -> &'static str {
    MOUNT_PREFIX
        .get_or_init(|| read_mount_prefix_from_conf().unwrap_or_else(|| "/mnt".to_string()))
        .as_str()
}

fn read_mount_prefix_from_conf() -> Option<String> {
    let content = std::fs::read_to_string("/etc/wsl.conf").ok()?;
    let mut in_automount = false;
    for raw in content.lines() {
        let line = raw.split('#').next().unwrap_or("").trim();
        if line.is_empty() {
            continue;
        }
        if let Some(stripped) = line.strip_prefix('[') {
            if let Some(name) = stripped.strip_suffix(']') {
                in_automount = name.trim().eq_ignore_ascii_case("automount");
                continue;
            }
        }
        if !in_automount {
            continue;
        }
        if let Some((k, v)) = line.split_once('=') {
            if k.trim().eq_ignore_ascii_case("root") {
                let val = v.trim().trim_matches('"').trim_matches('\'');
                let val = val.trim_end_matches('/');
                if !val.is_empty() {
                    let prefix = if val.starts_with('/') {
                        val.to_string()
                    } else {
                        format!("/{}", val)
                    };
                    return Some(prefix);
                }
            }
        }
    }
    None
}

pub fn detect_distro_name() -> &'static str {
    DISTRO_NAME
        .get_or_init(|| {
            std::env::var("WSL_DISTRO_NAME")
                .ok()
                .or_else(read_distro_from_os_release)
                .unwrap_or_else(|| "Ubuntu".to_string())
        })
        .as_str()
}

fn read_distro_from_os_release() -> Option<String> {
    let content = std::fs::read_to_string("/etc/os-release").ok()?;
    let mut id: Option<String> = None;
    let mut version_id: Option<String> = None;
    for line in content.lines() {
        let line = line.trim();
        if let Some(v) = line.strip_prefix("ID=") {
            id = Some(v.trim_matches('"').trim_matches('\'').to_string());
        } else if let Some(v) = line.strip_prefix("VERSION_ID=") {
            version_id = Some(v.trim_matches('"').trim_matches('\'').to_string());
        }
    }
    let id = id?;
    let mut name = capitalize(&id);
    if let Some(v) = version_id {
        if !v.is_empty() {
            name.push('-');
            name.push_str(&v);
        }
    }
    Some(name)
}

fn capitalize(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        Some(c) => c.to_ascii_uppercase().to_string() + chars.as_str(),
        None => String::new(),
    }
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
    fn test_clean_path_input_short_quote_does_not_panic() {
        assert_eq!(clean_path_input("\""), "\"");
        assert_eq!(clean_path_input("'"), "'");
        assert_eq!(clean_path_input(""), "");
    }

    #[test]
    fn test_extract_drive_letter() {
        assert_eq!(extract_drive_letter("C:\\foo"), Some('c'));
        assert_eq!(extract_drive_letter("d:/bar"), Some('d'));
        assert_eq!(extract_drive_letter("/home"), None);
        assert_eq!(extract_drive_letter("foo"), None);
    }

    #[test]
    fn test_capitalize() {
        assert_eq!(capitalize("ubuntu"), "Ubuntu");
        assert_eq!(capitalize("debian"), "Debian");
        assert_eq!(capitalize(""), "");
    }
}
