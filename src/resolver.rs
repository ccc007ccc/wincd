use std::path::{Path, PathBuf};

/// 路径解析结果
#[derive(Debug)]
pub struct ResolveResult {
    /// 最终使用的路径
    pub path: PathBuf,
    /// 是否为原始路径（存在）
    pub exact: bool,
    /// 如果做了模糊匹配，匹配到的建议列表
    pub suggestions: Vec<PathBuf>,
}

/// 检查路径是否存在，不存在时尝试向上查找
pub fn resolve_path(path: &str, use_parent: bool, force: bool) -> ResolveResult {
    let p = PathBuf::from(path);

    // 强制模式，跳过检查
    if force {
        return ResolveResult {
            path: p,
            exact: true,
            suggestions: vec![],
        };
    }

    // 路径存在，直接返回
    if p.exists() {
        return ResolveResult {
            path: p,
            exact: true,
            suggestions: vec![],
        };
    }

    // 启用了向上查找
    if use_parent {
        if let Some(parent) = find_existing_parent(&p) {
            let suggestions = list_suggestions(&parent);
            return ResolveResult {
                path: parent,
                exact: false,
                suggestions,
            };
        }
    }

    // 尝试在父目录中模糊匹配
    if let Some(parent) = p.parent() {
        if parent.exists() {
            let suggestions = list_suggestions(parent);
            return ResolveResult {
                path: p,
                exact: false,
                suggestions,
            };
        }
    }

    ResolveResult {
        path: p,
        exact: false,
        suggestions: vec![],
    }
}

/// 向上查找最近存在的父目录
fn find_existing_parent(path: &Path) -> Option<PathBuf> {
    let mut current = path;
    loop {
        if current.exists() {
            return Some(current.to_path_buf());
        }
        match current.parent() {
            Some(parent) => current = parent,
            None => return None,
        }
    }
}

/// 列出目录下的子目录作为建议
fn list_suggestions(dir: &Path) -> Vec<PathBuf> {
    let mut suggestions = Vec::new();
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            if entry.path().is_dir() {
                suggestions.push(entry.path());
            }
        }
    }
    suggestions.sort();
    // 最多返回 10 个建议
    suggestions.truncate(10);
    suggestions
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_existing_path() {
        let tmp = std::env::temp_dir();
        let result = resolve_path(tmp.to_str().unwrap(), false, false);
        assert!(result.exact);
    }

    #[test]
    fn test_resolve_nonexistent_force() {
        let result = resolve_path("/nonexistent/path/12345", false, true);
        assert!(result.exact); // force 模式下视为 exact
    }

    #[test]
    fn test_resolve_nonexistent_with_parent() {
        let tmp = std::env::temp_dir();
        let fake = tmp.join("wincd_test_nonexistent_12345");
        let result = resolve_path(fake.to_str().unwrap(), true, false);
        // 应该向上找到 tmp 目录
        assert!(!result.exact);
        assert!(result.path.exists());
    }

    #[test]
    fn test_list_suggestions() {
        let tmp = std::env::temp_dir();
        let suggestions = list_suggestions(&tmp);
        // tmp 目录应该有内容
        assert!(!suggestions.is_empty());
    }
}
