use std::path::{Path, PathBuf};
use anyhow::{Result, Context, anyhow};

pub struct PathGuard {
    allowed_roots: Vec<PathBuf>,
}

impl PathGuard {
    pub fn new(roots: Vec<PathBuf>) -> Self {
        let allowed_roots = roots.into_iter()
            .filter_map(|p| p.canonicalize().ok())
            .collect();
        Self { allowed_roots }
    }

    pub fn canonicalize_path(&self, path: &str, is_writable: bool) -> Result<PathBuf> {
        let path = Path::new(path);
        let absolute = if path.is_absolute() {
            path.to_path_buf()
        } else {
            std::env::current_dir()?.join(path)
        };

        let canonical = absolute.canonicalize()
            .or_else(|_| if is_writable { Ok(absolute) } else { Err(anyhow!("File not found")) })?;

        for root in &self.allowed_roots {
            if canonical.starts_with(root) {
                return Ok(canonical);
            }
        }

        Err(anyhow!("Path {:?} is outside of allowed roots", canonical))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use tempfile::tempdir;

    #[test]
    fn test_path_within_roots() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.txt");
        File::create(&file_path).unwrap();
        
        let guard = PathGuard::new(vec![dir.path().to_path_buf()]);
        let res = guard.canonicalize_path(file_path.to_str().unwrap(), false);
        assert!(res.is_ok());
    }

    #[test]
    fn test_path_outside_roots() {
        let dir1 = tempdir().unwrap();
        let dir2 = tempdir().unwrap();
        let file_path = dir1.path().join("test.txt");
        File::create(&file_path).unwrap();
        
        let guard = PathGuard::new(vec![dir2.path().to_path_buf()]);
        let res = guard.canonicalize_path(file_path.to_str().unwrap(), false);
        assert!(res.is_err());
    }
}
