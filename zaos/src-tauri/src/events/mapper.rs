//! Maps runtime-specific CliEvent to provider-neutral Vec<ZaosEvent>.
//!
//! This is the Rust equivalent of the former TypeScript `claudeMapper.ts`.
//! One CliEvent may produce zero or more ZaosEvents (e.g. an AssistantEvent
//! with 3 content blocks → 3 separate events).

use super::types::*;
use super::zaos_events::ZaosEvent;

/// Convert a single Claude CLI event into zero or more ZAOS events.
pub fn map_cli_event(event: &CliEvent) -> Vec<ZaosEvent> {
    match event {
        CliEvent::System(sys) => map_system(sys),
        CliEvent::StreamDelta(sd) => map_stream_delta(sd),
        CliEvent::Assistant(asst) => map_assistant(asst),
        CliEvent::User(user) => map_user(user),
        CliEvent::ControlRequest(req) => map_control_request(req),
        CliEvent::RateLimit(rl) => map_rate_limit(rl),
        CliEvent::Result(res) => map_result(res),
        CliEvent::Unknown => vec![ZaosEvent::Unknown {}],
    }
}

fn map_system(sys: &SystemEvent) -> Vec<ZaosEvent> {
    if sys.subtype != "init" {
        return vec![ZaosEvent::Unknown {}];
    }
    vec![ZaosEvent::SessionInit {
        session_id: sys.session_id.clone(),
        model: sys.model.clone(),
        tools: sys.tools.clone(),
        mcp_servers: sys.mcp_servers.clone(),
        cwd: sys.cwd.clone(),
        runtime_version: sys.claude_code_version.clone(),
        agents: sys.agents.clone(),
    }]
}

fn map_stream_delta(sd: &StreamDeltaEvent) -> Vec<ZaosEvent> {
    let mut events = Vec::new();

    // Agent activity heartbeat
    if let Some(ref parent_id) = sd.parent_tool_use_id {
        events.push(ZaosEvent::AgentActivity {
            delegation_id: parent_id.clone(),
        });
    }

    match &sd.event {
        ApiStreamEvent::ContentBlockStart { content_block, .. } => {
            events.push(ZaosEvent::StreamControl {
                action: "block_start".to_string(),
                block_type: Some(content_block.block_type.clone()),
            });
        }
        ApiStreamEvent::ContentBlockDelta { delta, .. } => match delta {
            DeltaContent::TextDelta { text } => {
                events.push(ZaosEvent::MessageDelta { text: text.clone() });
            }
            DeltaContent::ThinkingDelta { thinking } => {
                events.push(ZaosEvent::ThinkingDelta {
                    text: thinking.clone(),
                });
            }
            _ => {}
        },
        ApiStreamEvent::ContentBlockStop { .. } => {
            events.push(ZaosEvent::StreamControl {
                action: "block_stop".to_string(),
                block_type: None,
            });
        }
        ApiStreamEvent::MessageStop => {
            events.push(ZaosEvent::StreamControl {
                action: "message_end".to_string(),
                block_type: None,
            });
        }
        _ => {}
    }

    if events.is_empty() {
        vec![ZaosEvent::Unknown {}]
    } else {
        events
    }
}

fn map_assistant(asst: &AssistantEvent) -> Vec<ZaosEvent> {
    let mut events = Vec::new();
    let msg = &asst.message;

    if let Some(ref parent_id) = asst.parent_tool_use_id {
        events.push(ZaosEvent::AgentActivity {
            delegation_id: parent_id.clone(),
        });
    }

    for block in &msg.content {
        match block {
            ContentBlock::Text { text } => {
                if !text.is_empty() {
                    events.push(ZaosEvent::MessageComplete {
                        message_id: msg.id.clone(),
                        text: text.clone(),
                    });
                }
            }
            ContentBlock::ToolUse { id, name, input, .. } => {
                events.push(ZaosEvent::ToolCallStarted {
                    id: id.clone(),
                    name: name.clone(),
                    input: input.clone(),
                    parent_tool_use_id: asst.parent_tool_use_id.clone(),
                    message_id: msg.id.clone(),
                });
            }
            ContentBlock::Thinking { thinking, .. } => {
                if !thinking.is_empty() {
                    events.push(ZaosEvent::ThinkingComplete {
                        message_id: msg.id.clone(),
                        text: thinking.clone(),
                    });
                }
            }
            ContentBlock::Unknown => {}
        }
    }

    if events.is_empty() {
        vec![ZaosEvent::Unknown {}]
    } else {
        events
    }
}

fn map_user(user: &UserEvent) -> Vec<ZaosEvent> {
    let mut events = Vec::new();

    if let Some(ref parent_id) = user.parent_tool_use_id {
        events.push(ZaosEvent::AgentActivity {
            delegation_id: parent_id.clone(),
        });
    }

    for block in &user.message.content {
        if let UserContentBlock::ToolResult {
            tool_use_id,
            content,
            is_error,
        } = block
        {
            let content_str = match content {
                serde_json::Value::String(s) => s.clone(),
                other => other.to_string(),
            };
            let has_error = is_error.unwrap_or(false)
                || content_str.starts_with("Error")
                || content_str.starts_with("error")
                || content_str.starts_with("ERROR");

            events.push(ZaosEvent::ToolCallFinished {
                tool_use_id: tool_use_id.clone(),
                output: content_str,
                is_error: has_error,
            });
        }
    }

    if events.is_empty() {
        vec![ZaosEvent::Unknown {}]
    } else {
        events
    }
}

fn map_control_request(req: &ControlRequest) -> Vec<ZaosEvent> {
    let description = req.message.clone().or_else(|| {
        req.extra
            .get("request")?
            .get("description")?
            .as_str()
            .map(|s| s.to_string())
    });

    vec![ZaosEvent::ApprovalRequested {
        request_id: req.request_id.clone(),
        tool_name: req.tool_name().map(|s| s.to_string()),
        tool_input: req.tool_input(),
        description,
    }]
}

fn map_rate_limit(rl: &RateLimitEvent) -> Vec<ZaosEvent> {
    vec![ZaosEvent::RateLimited {
        status: rl.rate_limit_info.status.clone(),
        resets_at: rl.rate_limit_info.resets_at,
    }]
}

fn map_result(res: &ResultEvent) -> Vec<ZaosEvent> {
    let mut events = vec![ZaosEvent::RunCompleted {
        is_error: res.is_error,
        duration_ms: res.duration_ms,
        duration_api_ms: res.duration_api_ms,
        num_turns: res.num_turns,
        result: res.result.clone(),
        stop_reason: res.stop_reason.clone(),
        session_id: res.session_id.clone(),
        cost_usd: res.total_cost_usd,
    }];

    if let Some(input_tokens) = res.usage.get("input_tokens").and_then(|v| v.as_u64()) {
        let output_tokens = res
            .usage
            .get("output_tokens")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        let cache_read = res
            .usage
            .get("cache_read_input_tokens")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        events.push(ZaosEvent::TokenUsage {
            input_tokens,
            output_tokens,
            cache_read_tokens: cache_read,
            duration_ms: res.duration_ms,
            cost_usd: res.total_cost_usd,
        });
    }

    events
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unknown_event() {
        let events = map_cli_event(&CliEvent::Unknown);
        assert_eq!(events.len(), 1);
        assert!(matches!(events[0], ZaosEvent::Unknown {}));
    }

    #[test]
    fn test_system_init_maps_to_session_init() {
        let sys = SystemEvent {
            subtype: "init".to_string(),
            session_id: "test-session".to_string(),
            uuid: "test-uuid".to_string(),
            model: Some("claude-sonnet-4-6".to_string()),
            cwd: Some("/tmp".to_string()),
            tools: Some(vec!["Read".to_string()]),
            mcp_servers: None,
            claude_code_version: Some("1.0.0".to_string()),
            agents: None,
            slash_commands: None,
            skills: None,
            attempt: None,
            max_retries: None,
            error: None,
            error_status: None,
        };
        let events = map_cli_event(&CliEvent::System(sys));
        assert_eq!(events.len(), 1);
        assert!(matches!(events[0], ZaosEvent::SessionInit { .. }));
    }

    #[test]
    fn test_system_api_retry_maps_to_unknown() {
        let sys = SystemEvent {
            subtype: "api_retry".to_string(),
            session_id: "test".to_string(),
            uuid: "test".to_string(),
            model: None,
            cwd: None,
            tools: None,
            mcp_servers: None,
            claude_code_version: None,
            agents: None,
            slash_commands: None,
            skills: None,
            attempt: Some(1),
            max_retries: Some(3),
            error: None,
            error_status: None,
        };
        let events = map_cli_event(&CliEvent::System(sys));
        assert!(matches!(events[0], ZaosEvent::Unknown {}));
    }
}
