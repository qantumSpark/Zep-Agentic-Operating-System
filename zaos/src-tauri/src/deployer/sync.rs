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

    if target.exists() {
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

        if let Some(parent) = target.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(target, content)?;
        manifest.files.insert(rel_path.to_string(), new_hash);
        tracing::info!("Updated: {}", rel_path);
        Ok(SyncOutcome::Updated)
    } else {
        if let Some(parent) = target.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(target, content)?;
        manifest.files.insert(rel_path.to_string(), new_hash);
        tracing::info!("Created: {}", rel_path);
        Ok(SyncOutcome::Created)
    }
}

/// Sync all agents from source_dir to project's .claude/agents/
pub fn sync_agents(
    source_dir: &Path,
    project_dir: &Path,
    manifest: &mut DeployManifest,
    report: &mut SyncReport,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let agents_source = source_dir.join("agents");
    let agents_target = project_dir.join(".claude").join("agents");

    if !agents_source.exists() {
        tracing::warn!("Agents source directory not found: {:?}", agents_source);
        return Ok(());
    }

    for entry in std::fs::read_dir(&agents_source)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().map_or(false, |e| e == "md") {
            let filename = path.file_name().unwrap().to_string_lossy().to_string();
            let rel_path = format!("agents/{}", filename);
            let content = std::fs::read(&path)?;
            let target = agents_target.join(&filename);

            match sync_file(&target, &content, manifest, &rel_path)? {
                SyncOutcome::Created => report.created.push(rel_path),
                SyncOutcome::Updated => report.updated.push(rel_path),
                SyncOutcome::Skipped => report.skipped.push(rel_path),
            }
        }
    }
    Ok(())
}

/// Sync all rules from source_dir to project's .claude/rules/
pub fn sync_rules(
    source_dir: &Path,
    project_dir: &Path,
    manifest: &mut DeployManifest,
    report: &mut SyncReport,
    disabled_rules: &[String],
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let rules_source = source_dir.join("rules");
    let rules_target = project_dir.join(".claude").join("rules");

    if !rules_source.exists() {
        return Ok(());
    }

    for entry in std::fs::read_dir(&rules_source)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().map_or(false, |e| e == "mdc") {
            let filename = path.file_name().unwrap().to_string_lossy().to_string();

            // Skip disabled rules
            if disabled_rules.contains(&filename) {
                continue;
            }

            let rel_path = format!("rules/{}", filename);
            let content = std::fs::read(&path)?;
            let target = rules_target.join(&filename);

            match sync_file(&target, &content, manifest, &rel_path)? {
                SyncOutcome::Created => report.created.push(rel_path),
                SyncOutcome::Updated => report.updated.push(rel_path),
                SyncOutcome::Skipped => report.skipped.push(rel_path),
            }
        }
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

    match sync_file(&target, content.as_bytes(), manifest, rel_path)? {
        SyncOutcome::Created => report.created.push(rel_path.to_string()),
        SyncOutcome::Updated => report.updated.push(rel_path.to_string()),
        SyncOutcome::Skipped => report.skipped.push(rel_path.to_string()),
    }

    Ok(())
}

/// Full sync: agents + rules + settings
pub fn sync_all(
    source_dir: &Path,
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

    sync_agents(source_dir, project_dir, &mut manifest, &mut report)?;
    sync_rules(
        source_dir,
        project_dir,
        &mut manifest,
        &mut report,
        &config.disabled_rules,
    )?;
    sync_settings(
        hooks_binary_path,
        project_dir,
        config.hooks_enabled,
        &mut manifest,
        &mut report,
    )?;

    // Update manifest timestamp
    manifest.last_deployed = Some(chrono::Utc::now().to_rfc3339());
    manifest.save(project_dir)?;

    tracing::info!(
        "Sync complete: {} created, {} updated, {} skipped",
        report.created.len(),
        report.updated.len(),
        report.skipped.len()
    );

    Ok(report)
}
