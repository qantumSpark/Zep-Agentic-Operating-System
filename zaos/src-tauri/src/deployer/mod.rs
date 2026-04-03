pub mod config;
pub mod embedded;
pub mod sync;

use std::path::{Path, PathBuf};
use config::WorkflowKitConfig;
use sync::SyncReport;

/// Find the zaos-hooks binary path.
/// In dev mode: look in target/debug/
/// In production: look in .zaos/bin/ or next to the main binary.
#[allow(dead_code)]
pub fn find_hooks_binary(project_dir: &Path) -> Option<PathBuf> {
    // Check .zaos/bin/ first (deployed)
    let deployed = project_dir.join(".zaos").join("bin").join(hooks_binary_name());
    if deployed.exists() {
        return Some(deployed);
    }

    // Check target/debug/ (dev mode)
    let dev_candidate = project_dir
        .join("src-tauri")
        .join("target")
        .join("debug")
        .join(hooks_binary_name());
    if dev_candidate.exists() {
        return Some(dev_candidate);
    }

    // Check next to current executable
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            let candidate = dir.join(hooks_binary_name());
            if candidate.exists() {
                return Some(candidate);
            }
        }
    }

    None
}

fn hooks_binary_name() -> &'static str {
    if cfg!(windows) {
        "zaos-hooks.exe"
    } else {
        "zaos-hooks"
    }
}

/// Deploy the zaos-hooks binary to .zaos/bin/
/// Copies from the source location (next to exe) if not already deployed.
/// Returns the deployed path, or the source path if copy fails.
pub fn deploy_hooks_binary(
    project_dir: &Path,
) -> Result<Option<PathBuf>, Box<dyn std::error::Error + Send + Sync>> {
    let target = project_dir
        .join(".zaos")
        .join("bin")
        .join(hooks_binary_name());

    // Find source binary (next to current exe — works in both dev and prod)
    let source = std::env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(|d| d.join(hooks_binary_name())))
        .filter(|p| p.exists());

    // If already deployed, check if source is newer
    if target.exists() {
        if let Some(ref src) = source {
            let src_modified = std::fs::metadata(src).and_then(|m| m.modified()).ok();
            let tgt_modified = std::fs::metadata(&target).and_then(|m| m.modified()).ok();
            if let (Some(s), Some(t)) = (src_modified, tgt_modified) {
                if s > t {
                    std::fs::copy(src, &target)?;
                    tracing::info!("Updated zaos-hooks binary at {:?}", target);
                }
            }
        }
        return Ok(Some(target));
    }

    // Not deployed yet — copy from source
    if let Some(ref src) = source {
        if let Some(parent) = target.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::copy(src, &target)?;
        tracing::info!("Deployed zaos-hooks binary to {:?}", target);
        return Ok(Some(target));
    }

    tracing::warn!("zaos-hooks binary not found — hooks will not be active");
    Ok(None)
}

/// Full deploy: config + binary + sync agents/rules/settings.
/// Agent and rule content comes from embedded:: (compiled into the binary).
pub fn deploy(project_dir: &Path) -> Result<SyncReport, Box<dyn std::error::Error + Send + Sync>> {
    let config = WorkflowKitConfig::load(project_dir);

    // Deploy hooks binary
    let hooks_path = deploy_hooks_binary(project_dir)?
        .unwrap_or_else(|| PathBuf::from("zaos-hooks")); // fallback to PATH

    // Sync everything (agents/rules from embedded::, settings generated)
    sync::sync_all(project_dir, &hooks_path, &config)
}
