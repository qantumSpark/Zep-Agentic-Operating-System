use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Clone, Serialize)]
pub struct SyncReport {
    pub created: Vec<String>,
    pub updated: Vec<String>,
    pub skipped: Vec<String>,
}

/// Manifest tracking what was last deployed (stored at .zaos/deploy-manifest.json)
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct DeployManifest {
    pub files: HashMap<String, String>, // relative path -> hash
    pub last_deployed: Option<String>,  // ISO timestamp
}

impl DeployManifest {
    pub fn load(project_dir: &Path) -> Self {
        let path = project_dir.join(".zaos").join("deploy-manifest.json");
        std::fs::read_to_string(&path)
            .ok()
            .and_then(|c| serde_json::from_str(&c).ok())
            .unwrap_or_default()
    }

    pub fn save(&self, project_dir: &Path) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let path = project_dir.join(".zaos").join("deploy-manifest.json");
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&path, serde_json::to_string_pretty(self)?)?;
        Ok(())
    }
}

/// FNV-1a 64-bit hash producing a hex fingerprint for content comparison.
fn fnv1a_hex(data: &[u8]) -> String {
    let mut hash: u64 = 0xcbf29ce484222325; // FNV offset basis
    for byte in data {
        hash ^= *byte as u64;
        hash = hash.wrapping_mul(0x100000001b3); // FNV prime
    }
    format!("{:016x}", hash)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SyncOutcome {
    Created,
    Updated,
    Skipped,
}

impl SyncReport {
    fn record(&mut self, outcome: SyncOutcome, path: String) {
        match outcome {
            SyncOutcome::Created => self.created.push(path),
            SyncOutcome::Updated => self.updated.push(path),
            SyncOutcome::Skipped => self.skipped.push(path),
        }
    }
}

/// Sync a single file from source content to target path.
fn sync_file(
    target: &Path,
    content: &[u8],
    manifest: &mut DeployManifest,
    rel_path: &str,
) -> Result<SyncOutcome, Box<dyn std::error::Error + Send + Sync>> {
    let new_hash = fnv1a_hex(content);

    // Fast path: manifest says we deployed this exact content and target exists
    if let Some(manifest_hash) = manifest.files.get(rel_path) {
        if *manifest_hash == new_hash && target.exists() {
            return Ok(SyncOutcome::Skipped);
        }
    }

    let outcome = if target.exists() {
        let existing = std::fs::read(target)?;
        let existing_hash = fnv1a_hex(&existing);

        if existing_hash == new_hash {
            manifest.files.insert(rel_path.to_string(), new_hash);
            return Ok(SyncOutcome::Skipped);
        }

        // User modified the file — don't overwrite
        if let Some(manifest_hash) = manifest.files.get(rel_path) {
            if *manifest_hash != existing_hash {
                tracing::info!("Skipping user-modified file: {}", rel_path);
                return Ok(SyncOutcome::Skipped);
            }
        }

        SyncOutcome::Updated
    } else {
        SyncOutcome::Created
    };

    if let Some(parent) = target.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(target, content)?;
    manifest.files.insert(rel_path.to_string(), new_hash);
    tracing::info!("{:?}: {}", outcome, rel_path);
    Ok(outcome)
}

/// Sync a set of embedded files to project's .claude/{subdir}/
fn sync_embedded(
    project_dir: &Path,
    subdir: &str,
    files: &[super::embedded::EmbeddedFile],
    skip: &[String],
    manifest: &mut DeployManifest,
    report: &mut SyncReport,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let target_dir = project_dir.join(".claude").join(subdir);

    for file in files {
        if skip.iter().any(|s| s == file.filename) {
            continue;
        }

        let rel_path = format!("{}/{}", subdir, file.filename);
        let target = target_dir.join(file.filename);

        report.record(sync_file(&target, file.content.as_bytes(), manifest, &rel_path)?, rel_path);
    }
    Ok(())
}

/// Generate and sync .claude/settings.json with hook configuration
pub fn sync_settings(
    hooks_binary_path: &Path,
    project_dir: &Path,
    hooks_enabled: bool,
    manifest: &mut DeployManifest,
    report: &mut SyncReport,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    if !hooks_enabled {
        tracing::info!("Hooks disabled — skipping settings.json generation");
        return Ok(());
    }

    let binary = hooks_binary_path.to_string_lossy().replace('\\', "/");

    let settings = serde_json::json!({
        "hooks": {
            "UserPromptSubmit": [{
                "hooks": [{
                    "type": "command",
                    "command": format!("\"{}\" inject-context", binary),
                    "timeout": 5
                }]
            }],
            "PreToolUse": [{
                "matcher": "Write|Edit",
                "hooks": [{
                    "type": "command",
                    "command": format!("\"{}\" block-code", binary),
                    "timeout": 5
                }]
            }],
            "SessionStart": [{
                "matcher": "compact",
                "hooks": [{
                    "type": "command",
                    "command": format!("\"{}\" on-compact", binary),
                    "timeout": 10
                }]
            }]
        }
    });

    let content = serde_json::to_string_pretty(&settings)?;
    let target = project_dir.join(".claude").join("settings.json");
    let rel_path = "settings.json";

    report.record(sync_file(&target, content.as_bytes(), manifest, rel_path)?, rel_path.to_string());

    Ok(())
}

/// Sync CLAUDE.md to project root
pub fn sync_claude_md(
    project_dir: &Path,
    manifest: &mut DeployManifest,
    report: &mut SyncReport,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let target = project_dir.join("CLAUDE.md");
    let rel_path = "CLAUDE.md";

    report.record(sync_file(&target, super::embedded::CLAUDE_MD.as_bytes(), manifest, rel_path)?, rel_path.to_string());
    Ok(())
}

/// Full sync: agents + rules + settings + CLAUDE.md (all content from embedded::)
pub fn sync_all(
    project_dir: &Path,
    hooks_binary_path: &Path,
    config: &super::config::WorkflowKitConfig,
) -> Result<SyncReport, Box<dyn std::error::Error + Send + Sync>> {
    let mut manifest = DeployManifest::load(project_dir);
    let mut report = SyncReport {
        created: vec![],
        updated: vec![],
        skipped: vec![],
    };

    sync_embedded(project_dir, "agents", super::embedded::AGENTS, &[], &mut manifest, &mut report)?;
    sync_embedded(project_dir, "rules", super::embedded::RULES, &config.disabled_rules, &mut manifest, &mut report)?;
    sync_settings(
        hooks_binary_path,
        project_dir,
        config.hooks_enabled,
        &mut manifest,
        &mut report,
    )?;
    sync_claude_md(project_dir, &mut manifest, &mut report)?;

    // Only persist manifest when something actually changed
    if !report.created.is_empty() || !report.updated.is_empty() {
        manifest.last_deployed = Some(chrono::Utc::now().to_rfc3339());
        manifest.save(project_dir)?;
    }

    tracing::info!(
        "Sync complete: {} created, {} updated, {} skipped",
        report.created.len(),
        report.updated.len(),
        report.skipped.len()
    );

    Ok(report)
}
