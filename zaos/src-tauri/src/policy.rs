use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fmt;

use crate::workflow::product_phase::ProductPhase;

// ── Enums ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PolicyProfile {
    Observe,
    GuidedBuild,
    AutopilotSafe,
    ReleaseGuarded,
}

impl Default for PolicyProfile {
    fn default() -> Self {
        PolicyProfile::GuidedBuild
    }
}

impl fmt::Display for PolicyProfile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let label = match self {
            PolicyProfile::Observe => "observe",
            PolicyProfile::GuidedBuild => "guided-build",
            PolicyProfile::AutopilotSafe => "autopilot-safe",
            PolicyProfile::ReleaseGuarded => "release-guarded",
        };
        write!(f, "{}", label)
    }
}

impl PolicyProfile {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "observe" => Some(PolicyProfile::Observe),
            "guided-build" => Some(PolicyProfile::GuidedBuild),
            "autopilot-safe" => Some(PolicyProfile::AutopilotSafe),
            "release-guarded" => Some(PolicyProfile::ReleaseGuarded),
            _ => None,
        }
    }
}

#[cfg(test)]
pub const ALL_PROFILES: &[PolicyProfile] = &[
    PolicyProfile::Observe,
    PolicyProfile::GuidedBuild,
    PolicyProfile::AutopilotSafe,
    PolicyProfile::ReleaseGuarded,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActionType {
    FileWrite,
    FileDelete,
    BashCommand,
    WebFetch,
    ToolCall,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Verdict {
    Allow,
    Ask,
    Deny,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

// ── Structs ────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct ActionContext {
    pub action_type: ActionType,
    pub file_paths: Vec<String>,
    pub tool_name: Option<String>,
    /// The raw command string extracted from Bash tool input (e.g. "cargo test").
    /// Used by `is_test_command` to inspect the actual command, not just tool_name.
    pub command_text: Option<String>,
    pub product_phase: ProductPhase,
    pub is_destructive: bool,
    #[allow(dead_code)] // Populated but not yet used in evaluate(); reserved for future policy rules
    pub is_reversible: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyDecision {
    pub verdict: Verdict,
    pub risk_level: RiskLevel,
    pub reason: String,
    pub matched_rules: Vec<String>,
}

// ── Helpers ────────────────────────────────────────────────────────────

/// Returns true if ALL paths are .md files or start with .memory/
fn is_memory_or_doc(paths: &[String]) -> bool {
    if paths.is_empty() {
        return false;
    }
    paths.iter().all(|p| {
        p.ends_with(".md")
            || p.starts_with(".memory/")
            || p.starts_with(".memory\\")
    })
}

/// Returns true if the command text contains "test" (simple heuristic).
/// Inspects `command_text` (the actual bash command) rather than `tool_name`
/// (which is always "Bash" for bash commands and would never match).
fn is_test_command(ctx: &ActionContext) -> bool {
    match &ctx.command_text {
        Some(cmd) => cmd.to_lowercase().contains("test"),
        None => false,
    }
}

/// Base risk level for a given action type
fn base_risk(action_type: &ActionType) -> RiskLevel {
    match action_type {
        ActionType::FileDelete => RiskLevel::High,
        ActionType::BashCommand => RiskLevel::Medium,
        ActionType::FileWrite => RiskLevel::Medium,
        ActionType::WebFetch => RiskLevel::Low,
        ActionType::ToolCall => RiskLevel::Medium,
    }
}

/// Elevate verdict: Deny > Ask > Allow
pub fn max_verdict(a: Verdict, b: Verdict) -> Verdict {
    match (a, b) {
        (Verdict::Deny, _) | (_, Verdict::Deny) => Verdict::Deny,
        (Verdict::Ask, _) | (_, Verdict::Ask) => Verdict::Ask,
        _ => Verdict::Allow,
    }
}

/// Derive an `ActionType` from the tool name sent by the AI runtime.
pub fn derive_action_type(tool_name: &str) -> ActionType {
    match tool_name {
        "Write" | "Edit" | "MultiEdit" => ActionType::FileWrite,
        "Bash" => ActionType::BashCommand,
        "WebFetch" | "WebSearch" => ActionType::WebFetch,
        _ => ActionType::ToolCall,
    }
}

/// Heuristic: returns `true` when the JSON input contains a bash command
/// that looks destructive (rm, git reset --hard, etc.).
pub fn is_destructive_command(input: &Value) -> bool {
    let command = match input.get("command").and_then(|v| v.as_str()) {
        Some(c) => c.to_lowercase(),
        None => return false,
    };

    let patterns = [
        "rm ",
        "rm -",
        "rmdir",
        "del ",
        "drop ",
        "truncate ",
        "git reset --hard",
        "git clean",
    ];

    patterns.iter().any(|p| command.contains(p))
}

/// Deterministic string representation of a Verdict (matches serde output).
pub fn verdict_to_str(v: Verdict) -> &'static str {
    match v {
        Verdict::Allow => "allow",
        Verdict::Ask => "ask",
        Verdict::Deny => "deny",
    }
}

/// Deterministic string representation of a RiskLevel (matches serde output).
pub fn risk_level_to_str(r: RiskLevel) -> &'static str {
    match r {
        RiskLevel::Low => "low",
        RiskLevel::Medium => "medium",
        RiskLevel::High => "high",
        RiskLevel::Critical => "critical",
    }
}

// ── Evaluate ───────────────────────────────────────────────────────────

pub fn evaluate(profile: &PolicyProfile, ctx: &ActionContext) -> PolicyDecision {
    let mut rules: Vec<String> = Vec::new();
    let mut risk = base_risk(&ctx.action_type);

    rules.push(format!("profile:{}", profile));
    rules.push(format!("action:{:?}", ctx.action_type).to_lowercase());

    // Base verdict from the profile x action matrix
    let mut verdict = match profile {
        // ── Observe ────────────────────────────────────────────
        PolicyProfile::Observe => {
            match ctx.action_type {
                ActionType::WebFetch => {
                    rules.push("verdict:ask".to_string());
                    Verdict::Ask
                }
                _ => {
                    rules.push("verdict:deny".to_string());
                    Verdict::Deny
                }
            }
        }

        // ── GuidedBuild ────────────────────────────────────────
        PolicyProfile::GuidedBuild => {
            match ctx.action_type {
                ActionType::FileWrite => {
                    if is_memory_or_doc(&ctx.file_paths) {
                        rules.push("memory_doc_auto_allow".to_string());
                        rules.push("verdict:allow".to_string());
                        Verdict::Allow
                    } else {
                        rules.push("verdict:ask".to_string());
                        Verdict::Ask
                    }
                }
                ActionType::FileDelete => {
                    rules.push("verdict:deny".to_string());
                    Verdict::Deny
                }
                ActionType::BashCommand => {
                    rules.push("verdict:ask".to_string());
                    Verdict::Ask
                }
                ActionType::WebFetch => {
                    rules.push("verdict:allow".to_string());
                    Verdict::Allow
                }
                ActionType::ToolCall => {
                    rules.push("verdict:ask".to_string());
                    Verdict::Ask
                }
            }
        }

        // ── AutopilotSafe ──────────────────────────────────────
        PolicyProfile::AutopilotSafe => {
            match ctx.action_type {
                ActionType::FileWrite => {
                    rules.push("verdict:allow".to_string());
                    Verdict::Allow
                }
                ActionType::FileDelete => {
                    rules.push("verdict:ask".to_string());
                    Verdict::Ask
                }
                ActionType::BashCommand => {
                    if ctx.is_destructive {
                        rules.push("destructive_bash_elevated".to_string());
                        rules.push("verdict:ask".to_string());
                        Verdict::Ask
                    } else {
                        rules.push("verdict:allow".to_string());
                        Verdict::Allow
                    }
                }
                ActionType::WebFetch => {
                    rules.push("verdict:allow".to_string());
                    Verdict::Allow
                }
                ActionType::ToolCall => {
                    rules.push("verdict:allow".to_string());
                    Verdict::Allow
                }
            }
        }

        // ── ReleaseGuarded ─────────────────────────────────────
        PolicyProfile::ReleaseGuarded => {
            match ctx.action_type {
                ActionType::FileWrite => {
                    if is_memory_or_doc(&ctx.file_paths) {
                        rules.push("memory_doc_exception".to_string());
                        rules.push("verdict:allow".to_string());
                        Verdict::Allow
                    } else {
                        rules.push("verdict:deny".to_string());
                        Verdict::Deny
                    }
                }
                ActionType::FileDelete => {
                    rules.push("verdict:deny".to_string());
                    Verdict::Deny
                }
                ActionType::BashCommand => {
                    if is_test_command(ctx) {
                        rules.push("test_command_exception".to_string());
                        rules.push("verdict:allow".to_string());
                        Verdict::Allow
                    } else {
                        rules.push("verdict:deny".to_string());
                        Verdict::Deny
                    }
                }
                ActionType::WebFetch => {
                    rules.push("verdict:ask".to_string());
                    Verdict::Ask
                }
                ActionType::ToolCall => {
                    rules.push("verdict:deny".to_string());
                    Verdict::Deny
                }
            }
        }
    };

    // Build base reason
    let reason = match profile {
        PolicyProfile::Observe => {
            match ctx.action_type {
                ActionType::WebFetch => "observe: web fetch requires approval".to_string(),
                _ => "observe: all writes denied".to_string(),
            }
        }
        PolicyProfile::GuidedBuild => {
            match ctx.action_type {
                ActionType::FileWrite if is_memory_or_doc(&ctx.file_paths) => {
                    "guided-build: memory/doc files auto-allowed".to_string()
                }
                ActionType::FileWrite => "guided-build: file write requires approval".to_string(),
                ActionType::FileDelete => "guided-build: file deletion denied".to_string(),
                ActionType::BashCommand => "guided-build: bash command requires approval".to_string(),
                ActionType::WebFetch => "guided-build: web fetch auto-allowed".to_string(),
                ActionType::ToolCall => "guided-build: tool call requires approval".to_string(),
            }
        }
        PolicyProfile::AutopilotSafe => {
            match ctx.action_type {
                ActionType::FileDelete => "autopilot-safe: file deletion requires approval".to_string(),
                ActionType::BashCommand if ctx.is_destructive => {
                    "autopilot-safe: destructive bash command requires approval".to_string()
                }
                _ => "autopilot-safe: action auto-allowed".to_string(),
            }
        }
        PolicyProfile::ReleaseGuarded => {
            match ctx.action_type {
                ActionType::FileWrite if is_memory_or_doc(&ctx.file_paths) => {
                    "release-guarded: memory/doc files allowed".to_string()
                }
                ActionType::BashCommand if is_test_command(ctx) => {
                    "release-guarded: test command allowed".to_string()
                }
                ActionType::WebFetch => "release-guarded: web fetch requires approval".to_string(),
                _ => "release-guarded: action denied in release mode".to_string(),
            }
        }
    };

    // ── Post-base modulations ──────────────────────────────────────────

    // Destructive flag: elevate risk and verdict
    if ctx.is_destructive {
        if risk < RiskLevel::High {
            risk = RiskLevel::High;
        }
        verdict = max_verdict(verdict, Verdict::Ask);
        if !rules.contains(&"destructive_elevated".to_string()) {
            rules.push("destructive_elevated".to_string());
        }
    }

    // Verify phase: elevate code FileWrite to Ask for guided-build and autopilot-safe
    if ctx.product_phase == ProductPhase::Verify
        && ctx.action_type == ActionType::FileWrite
        && !is_memory_or_doc(&ctx.file_paths)
        && (*profile == PolicyProfile::GuidedBuild || *profile == PolicyProfile::AutopilotSafe)
    {
        verdict = max_verdict(verdict, Verdict::Ask);
        rules.push("verify_phase_write_elevated".to_string());
    }

    // Build final reason (append modulation info if elevated)
    let final_reason = if ctx.is_destructive && verdict == Verdict::Ask {
        format!("{} (destructive action: elevated to ask)", reason)
    } else if rules.contains(&"verify_phase_write_elevated".to_string()) {
        format!("{} (verify phase: code writes elevated to ask)", reason)
    } else {
        reason
    };

    PolicyDecision {
        verdict,
        risk_level: risk,
        reason: final_reason,
        matched_rules: rules,
    }
}

// ── Tests ──────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn make_ctx(action_type: ActionType) -> ActionContext {
        ActionContext {
            action_type,
            file_paths: vec!["src/main.rs".to_string()],
            tool_name: None,
            command_text: None,
            product_phase: ProductPhase::Build,
            is_destructive: false,
            is_reversible: true,
        }
    }

    // ── Per-profile representative tests ───────────────────────────

    #[test]
    fn test_observe_denies_file_write() {
        let ctx = make_ctx(ActionType::FileWrite);
        let decision = evaluate(&PolicyProfile::Observe, &ctx);
        assert_eq!(decision.verdict, Verdict::Deny);
        assert!(!decision.reason.is_empty());
        assert!(!decision.matched_rules.is_empty());
        assert!(decision.matched_rules.contains(&"profile:observe".to_string()));
    }

    #[test]
    fn test_observe_asks_web_fetch() {
        let ctx = make_ctx(ActionType::WebFetch);
        let decision = evaluate(&PolicyProfile::Observe, &ctx);
        assert_eq!(decision.verdict, Verdict::Ask);
    }

    #[test]
    fn test_guided_build_asks_file_write() {
        let ctx = make_ctx(ActionType::FileWrite);
        let decision = evaluate(&PolicyProfile::GuidedBuild, &ctx);
        assert_eq!(decision.verdict, Verdict::Ask);
        assert!(!decision.reason.is_empty());
        assert!(!decision.matched_rules.is_empty());
    }

    #[test]
    fn test_autopilot_safe_allows_file_write() {
        let ctx = make_ctx(ActionType::FileWrite);
        let decision = evaluate(&PolicyProfile::AutopilotSafe, &ctx);
        assert_eq!(decision.verdict, Verdict::Allow);
        assert!(!decision.reason.is_empty());
        assert!(!decision.matched_rules.is_empty());
    }

    #[test]
    fn test_release_guarded_denies_file_write() {
        let ctx = make_ctx(ActionType::FileWrite);
        let decision = evaluate(&PolicyProfile::ReleaseGuarded, &ctx);
        assert_eq!(decision.verdict, Verdict::Deny);
        assert!(!decision.reason.is_empty());
        assert!(!decision.matched_rules.is_empty());
    }

    // ── is_destructive elevates verdict ────────────────────────────

    #[test]
    fn test_destructive_elevates_to_ask() {
        let mut ctx = make_ctx(ActionType::BashCommand);
        ctx.is_destructive = true;

        // autopilot-safe normally allows bash, but destructive → Ask
        let decision = evaluate(&PolicyProfile::AutopilotSafe, &ctx);
        assert_eq!(decision.verdict, Verdict::Ask);
        assert!(decision.risk_level >= RiskLevel::High);
        assert!(decision.matched_rules.contains(&"destructive_elevated".to_string()));
    }

    #[test]
    fn test_destructive_does_not_lower_deny() {
        let mut ctx = make_ctx(ActionType::FileDelete);
        ctx.is_destructive = true;

        // observe denies file_delete; destructive shouldn't lower it to Ask
        let decision = evaluate(&PolicyProfile::Observe, &ctx);
        assert_eq!(decision.verdict, Verdict::Deny);
    }

    // ── Verify phase elevates FileWrite for guided-build ───────────

    #[test]
    fn test_verify_phase_elevates_guided_build_file_write() {
        let mut ctx = make_ctx(ActionType::FileWrite);
        ctx.product_phase = ProductPhase::Verify;

        let decision = evaluate(&PolicyProfile::GuidedBuild, &ctx);
        assert_eq!(decision.verdict, Verdict::Ask);
        assert!(decision.matched_rules.contains(&"verify_phase_write_elevated".to_string()));
    }

    #[test]
    fn test_verify_phase_elevates_autopilot_safe_file_write() {
        let mut ctx = make_ctx(ActionType::FileWrite);
        ctx.product_phase = ProductPhase::Verify;

        let decision = evaluate(&PolicyProfile::AutopilotSafe, &ctx);
        assert_eq!(decision.verdict, Verdict::Ask);
        assert!(decision.matched_rules.contains(&"verify_phase_write_elevated".to_string()));
    }

    // ── Memory/doc path → Allow for guided-build ───────────────────

    #[test]
    fn test_guided_build_allows_memory_files() {
        let ctx = ActionContext {
            action_type: ActionType::FileWrite,
            file_paths: vec![".memory/state.md".to_string()],
            tool_name: None,
            command_text: None,
            product_phase: ProductPhase::Build,
            is_destructive: false,
            is_reversible: true,
        };
        let decision = evaluate(&PolicyProfile::GuidedBuild, &ctx);
        assert_eq!(decision.verdict, Verdict::Allow);
        assert!(decision.matched_rules.contains(&"memory_doc_auto_allow".to_string()));
    }

    #[test]
    fn test_guided_build_allows_md_files() {
        let ctx = ActionContext {
            action_type: ActionType::FileWrite,
            file_paths: vec!["README.md".to_string()],
            tool_name: None,
            command_text: None,
            product_phase: ProductPhase::Build,
            is_destructive: false,
            is_reversible: true,
        };
        let decision = evaluate(&PolicyProfile::GuidedBuild, &ctx);
        assert_eq!(decision.verdict, Verdict::Allow);
    }

    #[test]
    fn test_guided_build_mixed_paths_not_auto_allowed() {
        let ctx = ActionContext {
            action_type: ActionType::FileWrite,
            file_paths: vec![
                ".memory/state.md".to_string(),
                "src/main.rs".to_string(),
            ],
            tool_name: None,
            command_text: None,
            product_phase: ProductPhase::Build,
            is_destructive: false,
            is_reversible: true,
        };
        let decision = evaluate(&PolicyProfile::GuidedBuild, &ctx);
        assert_eq!(decision.verdict, Verdict::Ask);
    }

    // ── ToolCall as fallback catch-all ──────────────────────────────

    #[test]
    fn test_tool_call_guided_build_asks() {
        let ctx = ActionContext {
            action_type: ActionType::ToolCall,
            file_paths: vec![],
            tool_name: Some("custom_tool".to_string()),
            command_text: None,
            product_phase: ProductPhase::Build,
            is_destructive: false,
            is_reversible: true,
        };
        let decision = evaluate(&PolicyProfile::GuidedBuild, &ctx);
        assert_eq!(decision.verdict, Verdict::Ask);
        assert_eq!(decision.risk_level, RiskLevel::Medium);
    }

    #[test]
    fn test_tool_call_autopilot_safe_allows() {
        let ctx = ActionContext {
            action_type: ActionType::ToolCall,
            file_paths: vec![],
            tool_name: Some("any_tool".to_string()),
            command_text: None,
            product_phase: ProductPhase::Build,
            is_destructive: false,
            is_reversible: true,
        };
        let decision = evaluate(&PolicyProfile::AutopilotSafe, &ctx);
        assert_eq!(decision.verdict, Verdict::Allow);
    }

    // ── Release-guarded test command exception ──────────────────────

    #[test]
    fn test_release_guarded_allows_test_commands() {
        let ctx = ActionContext {
            action_type: ActionType::BashCommand,
            file_paths: vec![],
            tool_name: Some("Bash".to_string()),
            command_text: Some("cargo test".to_string()),
            product_phase: ProductPhase::Release,
            is_destructive: false,
            is_reversible: true,
        };
        let decision = evaluate(&PolicyProfile::ReleaseGuarded, &ctx);
        assert_eq!(decision.verdict, Verdict::Allow);
        assert!(decision.matched_rules.contains(&"test_command_exception".to_string()));
    }

    #[test]
    fn test_release_guarded_denies_non_test_bash() {
        let ctx = ActionContext {
            action_type: ActionType::BashCommand,
            file_paths: vec![],
            tool_name: Some("Bash".to_string()),
            command_text: Some("cargo build".to_string()),
            product_phase: ProductPhase::Release,
            is_destructive: false,
            is_reversible: true,
        };
        let decision = evaluate(&PolicyProfile::ReleaseGuarded, &ctx);
        assert_eq!(decision.verdict, Verdict::Deny);
    }

    // ── Helpers ────────────────────────────────────────────────────

    #[test]
    fn test_is_memory_or_doc_empty_paths() {
        assert!(!is_memory_or_doc(&[]));
    }

    #[test]
    fn test_is_memory_or_doc_all_md() {
        assert!(is_memory_or_doc(&["foo.md".to_string(), "bar.md".to_string()]));
    }

    #[test]
    fn test_is_memory_or_doc_mixed() {
        assert!(!is_memory_or_doc(&["foo.md".to_string(), "bar.rs".to_string()]));
    }

    #[test]
    fn test_is_test_command() {
        let mut ctx = make_ctx(ActionType::BashCommand);

        ctx.command_text = Some("cargo test".to_string());
        assert!(is_test_command(&ctx));

        ctx.command_text = Some("npm test".to_string());
        assert!(is_test_command(&ctx));

        ctx.command_text = Some("cargo build".to_string());
        assert!(!is_test_command(&ctx));

        ctx.command_text = None;
        assert!(!is_test_command(&ctx));
    }

    // ── Profile utility tests ──────────────────────────────────────

    #[test]
    fn test_default_profile() {
        assert_eq!(PolicyProfile::default(), PolicyProfile::GuidedBuild);
    }

    #[test]
    fn test_display_profiles() {
        assert_eq!(format!("{}", PolicyProfile::Observe), "observe");
        assert_eq!(format!("{}", PolicyProfile::GuidedBuild), "guided-build");
        assert_eq!(format!("{}", PolicyProfile::AutopilotSafe), "autopilot-safe");
        assert_eq!(format!("{}", PolicyProfile::ReleaseGuarded), "release-guarded");
    }

    #[test]
    fn test_from_str_valid() {
        assert_eq!(PolicyProfile::from_str("observe"), Some(PolicyProfile::Observe));
        assert_eq!(PolicyProfile::from_str("guided-build"), Some(PolicyProfile::GuidedBuild));
        assert_eq!(PolicyProfile::from_str("autopilot-safe"), Some(PolicyProfile::AutopilotSafe));
        assert_eq!(PolicyProfile::from_str("release-guarded"), Some(PolicyProfile::ReleaseGuarded));
    }

    #[test]
    fn test_from_str_invalid() {
        assert_eq!(PolicyProfile::from_str("invalid"), None);
        assert_eq!(PolicyProfile::from_str(""), None);
    }

    #[test]
    fn test_all_profiles_constant() {
        assert_eq!(ALL_PROFILES.len(), 4);
        assert!(ALL_PROFILES.contains(&PolicyProfile::Observe));
        assert!(ALL_PROFILES.contains(&PolicyProfile::GuidedBuild));
        assert!(ALL_PROFILES.contains(&PolicyProfile::AutopilotSafe));
        assert!(ALL_PROFILES.contains(&PolicyProfile::ReleaseGuarded));
    }

    // ── Reason and matched_rules never empty ───────────────────────

    #[test]
    fn test_all_profiles_produce_nonempty_reason_and_rules() {
        for profile in ALL_PROFILES {
            for action_type in &[
                ActionType::FileWrite,
                ActionType::FileDelete,
                ActionType::BashCommand,
                ActionType::WebFetch,
                ActionType::ToolCall,
            ] {
                let ctx = make_ctx(*action_type);
                let decision = evaluate(profile, &ctx);
                assert!(
                    !decision.reason.is_empty(),
                    "Empty reason for {:?} + {:?}",
                    profile, action_type
                );
                assert!(
                    !decision.matched_rules.is_empty(),
                    "Empty matched_rules for {:?} + {:?}",
                    profile, action_type
                );
            }
        }
    }

    // ── derive_action_type ────────────────────────────────────────

    #[test]
    fn test_derive_action_type_write() {
        assert_eq!(derive_action_type("Write"), ActionType::FileWrite);
    }

    #[test]
    fn test_derive_action_type_edit() {
        assert_eq!(derive_action_type("Edit"), ActionType::FileWrite);
    }

    #[test]
    fn test_derive_action_type_multi_edit() {
        assert_eq!(derive_action_type("MultiEdit"), ActionType::FileWrite);
    }

    #[test]
    fn test_derive_action_type_bash() {
        assert_eq!(derive_action_type("Bash"), ActionType::BashCommand);
    }

    #[test]
    fn test_derive_action_type_web_fetch() {
        assert_eq!(derive_action_type("WebFetch"), ActionType::WebFetch);
    }

    #[test]
    fn test_derive_action_type_web_search() {
        assert_eq!(derive_action_type("WebSearch"), ActionType::WebFetch);
    }

    #[test]
    fn test_derive_action_type_unknown() {
        assert_eq!(derive_action_type("SomeOtherTool"), ActionType::ToolCall);
    }

    // ── is_destructive_command ────────────────────────────────────

    #[test]
    fn test_is_destructive_rm() {
        let input = serde_json::json!({"command": "rm -rf /tmp/foo"});
        assert!(is_destructive_command(&input));
    }

    #[test]
    fn test_is_destructive_git_reset() {
        let input = serde_json::json!({"command": "git reset --hard HEAD"});
        assert!(is_destructive_command(&input));
    }

    #[test]
    fn test_is_destructive_safe_command() {
        let input = serde_json::json!({"command": "cargo test"});
        assert!(!is_destructive_command(&input));
    }

    #[test]
    fn test_is_destructive_no_command() {
        let input = serde_json::json!({});
        assert!(!is_destructive_command(&input));
    }

    #[test]
    fn test_is_destructive_git_clean() {
        let input = serde_json::json!({"command": "git clean -fd"});
        assert!(is_destructive_command(&input));
    }
}
