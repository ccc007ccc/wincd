//! 路径解析与建议

use std::path::{Path, PathBuf};
use strsim::jaro_winkler;

#[derive(Debug)]
pub struct ResolveResult {
    pub path: PathBuf,
    pub exact: bool,
    pub suggestions: Vec<PathBuf>,
}

const MAX_SUGGESTIONS: usize = 10;

pub fn resolve_path(path: &str, use_parent: bool, force: bool) -> ResolveResult {
    let p = PathBuf::from(path);

    if force {
        return ResolveResult {
            path: p,
            exact: true,
            suggestions: vec![],
        };
    }
    if p.exists() {
        return ResolveResult {
            path: p,
            exact: true,
            suggestions: vec![],
        };
    }

    if use_parent {
        if let Some(parent) = find_existing_parent(&p) {
            let suggestions = list_suggestions(&parent, p.file_name().and_then(|s| s.to_str()));
            return ResolveResult {
                path: parent,
                exact: false,
                suggestions,
            };
        }
    }

    if let Some(parent) = p.parent() {
        if parent.exists() {
            let suggestions = list_suggestions(parent, p.file_name().and_then(|s| s.to_str()));
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

fn list_suggestions(dir: &Path, hint: Option<&str>) -> Vec<PathBuf> {
    let mut entries: Vec<PathBuf> = Vec::new();
    if let Ok(rd) = std::fs::read_dir(dir) {
        for entry in rd.flatten() {
            if entry.path().is_dir() {
                entries.push(entry.path());
            }
        }
    }

    if let Some(target) = hint {
        let target_lower = target.to_lowercase();
        entries.sort_by(|a, b| {
            let sa = score(a, &target_lower);
            let sb = score(b, &target_lower);
            sb.partial_cmp(&sa).unwrap_or(std::cmp::Ordering::Equal)
        });
    } else {
        entries.sort();
    }

    entries.truncate(MAX_SUGGESTIONS);
    entries
}

fn score(path: &Path, target_lower: &str) -> f64 {
    let name = match path.file_name().and_then(|s| s.to_str()) {
        Some(n) => n.to_lowercase(),
        None => return 0.0,
    };
    jaro_winkler(&name, target_lower)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_resolve_existing_path() {
        let tmp = std::env::temp_dir();
        let result = resolve_path(tmp.to_str().unwrap(), false, false);
        assert!(result.exact);
    }

    #[test]
    fn test_resolve_nonexistent_force() {
        let result = resolve_path("/nonexistent/path/12345", false, true);
        assert!(result.exact);
    }

    #[test]
    fn test_resolve_nonexistent_with_parent() {
        let tmp = std::env::temp_dir();
        let fake = tmp.join("wincd_test_nonexistent_12345_zzz");
        let result = resolve_path(fake.to_str().unwrap(), true, false);
        assert!(!result.exact);
        assert!(result.path.exists());
    }

    #[test]
    fn test_list_suggestions_similarity_sort() {
        let tmp = std::env::temp_dir().join(format!("wincd_test_resolver_{}", std::process::id()));
        let _ = fs::remove_dir_all(&tmp);
        fs::create_dir_all(&tmp).unwrap();
        for d in ["alpha_match", "zzzzz", "beta", "alphabet"] {
            fs::create_dir(tmp.join(d)).unwrap();
        }
        let s = list_suggestions(&tmp, Some("alpha"));
        assert!(s.iter().take(2).all(|p| p
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .starts_with("alpha")));
        let _ = fs::remove_dir_all(&tmp);
    }
}
