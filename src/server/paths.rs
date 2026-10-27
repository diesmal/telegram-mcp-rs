//! Where the daemon keeps its socket, locks and log, and how a build is identified.

use anyhow::{Context, Result};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

/// Repo / install root: holds `.env`, `telegram.session` and `run/`.
///
/// `TELEGRAM_MCP_HOME` wins; otherwise it is derived from the binary location
/// (`<root>/target/{release,debug}/telegram-mcp-rs` → `<root>`), so it no longer
/// matters which directory a client starts the server from.
pub fn base_dir(exe: &Path) -> PathBuf {
    if let Ok(home) = std::env::var("TELEGRAM_MCP_HOME") {
        return PathBuf::from(home);
    }
    let dir = exe.parent().unwrap_or(Path::new(".")).to_path_buf();
    let is_profile_dir = matches!(dir.file_name().and_then(|n| n.to_str()), Some("release" | "debug"));
    let in_target = dir.parent().and_then(|p| p.file_name()).and_then(|n| n.to_str()) == Some("target");
    if is_profile_dir && in_target {
        if let Some(root) = dir.parent().and_then(|p| p.parent()) {
            return root.to_path_buf();
        }
    }
    dir
}

/// Identifies the binary currently on disk (mtime + size). A rebuild changes it,
/// so new connections go to a daemon running the new code while the old daemon
/// idles out on its own.
pub fn build_id(exe: &Path) -> Result<String> {
    let meta = std::fs::metadata(exe).with_context(|| format!("stat {}", exe.display()))?;
    let mtime = meta.modified()?.duration_since(UNIX_EPOCH)?.as_nanos();
    Ok(format!("{:x}", mtime ^ ((meta.len() as u128) << 64)).chars().rev().take(12).collect())
}

pub struct RunPaths {
    pub dir: PathBuf,
}

impl RunPaths {
    pub fn new(base: &Path) -> Result<Self> {
        let dir = base.join("run");
        std::fs::create_dir_all(&dir).with_context(|| format!("create {}", dir.display()))?;
        std::fs::set_permissions(&dir, std::fs::Permissions::from_mode(0o700))?;
        Ok(Self { dir })
    }

    pub fn socket(&self, build: &str) -> PathBuf {
        self.dir.join(format!("daemon-{build}.sock"))
    }

    /// Held for the daemon's whole life: guarantees one daemon per build.
    pub fn daemon_lock(&self, build: &str) -> PathBuf {
        self.dir.join(format!("daemon-{build}.lock"))
    }

    /// Held by a proxy while it starts a daemon, so concurrent proxies don't
    /// all spawn one.
    pub fn spawn_lock(&self) -> PathBuf {
        self.dir.join("spawn.lock")
    }

    pub fn log(&self) -> PathBuf {
        self.dir.join("daemon.log")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base_dir_from_cargo_target() {
        unsafe { std::env::remove_var("TELEGRAM_MCP_HOME") };
        assert_eq!(base_dir(Path::new("/x/repo/target/release/bin")), PathBuf::from("/x/repo"));
        assert_eq!(base_dir(Path::new("/opt/tg/bin")), PathBuf::from("/opt/tg"));
    }
}
