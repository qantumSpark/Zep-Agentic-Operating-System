# ZAOS - Ultimate Runtime and Control Plane Vision

## 1. Purpose

This document defines the target runtime and control plane architecture for the final version of ZAOS.

It answers a core systems question:

**What kind of runtime is required if ZAOS is to orchestrate agents, tools, policies, approvals, traces, and side effects reliably enough for professional software delivery?**

The answer is not a single giant agent, not a provider-bound chat loop, and not a UI that merely displays tool output.

The answer is a **governed control plane** with:

- a central supervisor
- bounded subagents
- provider adapters
- a durable session manager
- an append-only event log
- executable policies and approvals
- safe-output application
- isolated workspaces
- replayable traces
- read-only observers for the cockpit

---

## 2. Why ZAOS Needs an Ultimate Runtime

Agentic systems do not fail only because models forget context or reason badly.
They also fail because the runtime itself is too weak.

The most common runtime failures include:

- a single agent mixing orchestration, execution, judgment, and mutation
- direct writes without an approval boundary
- provider-specific session handling leaking into product logic
- subagents sharing identity or contaminating each other's state
- local session storage breaking continuity in ephemeral environments
- hooks that cannot identify the correct task, role, or subagent
- traces that are too raw to drive supervision
- retries that happen without checkpoints or stuck detection
- UI monitoring that observes failure but cannot govern runtime behavior

For ZAOS to become a professional delivery cockpit, runtime cannot remain an implementation detail.
It must be treated as a first-class control system.

---

## 3. Core Design Principles

The final ZAOS runtime must follow these principles:

- orchestration must be separate from execution
- runtime truth must not live inside the model prompt
- provider integrations must sit behind adapters
- every important action must emit a structured event
- side effects must be isolated and governable
- approvals and policies must be executable, not advisory
- live state, snapshot, trace, and handoff must remain distinct
- subagents must have stable identities and bounded privileges
- replay and resume must be designed in from the start
- observers must remain read-only

---

## 4. External Patterns This Builds On

The ZAOS runtime vision synthesizes strong recurring patterns from:

- [OpenAI Agents SDK - Sessions](https://openai.github.io/openai-agents-js/guides/sessions/)
- [OpenAI Agents SDK - Running agents](https://openai.github.io/openai-agents-python/running_agents/)
- [OpenAI Agents SDK - Handoffs](https://openai.github.io/openai-agents-python/handoffs/)
- [OpenAI Agents SDK - Multi-agent orchestration](https://openai.github.io/openai-agents-python/multi_agent/)
- [OpenAI Agents SDK - Tools](https://openai.github.io/openai-agents-python/tools/)
- [OpenAI Agents SDK - Tracing](https://openai.github.io/openai-agents-js/guides/tracing/)
- [LangGraph - Durable execution](https://docs.langchain.com/oss/javascript/langgraph/durable-execution)
- [LangGraph - Interrupts](https://docs.langchain.com/oss/javascript/langgraph/interrupts)
- [Claude Code hooks](https://docs.anthropic.com/en/docs/claude-code/hooks)
- [Claude Code settings](https://code.claude.com/docs/en/settings)
- [Claude Code sub-agents](https://docs.anthropic.com/en/docs/claude-code/sub-agents)
- [Claude Code security](https://docs.anthropic.com/en/docs/claude-code/security)
- [GitHub Agentic Workflows - Architecture](https://github.github.com/gh-aw/introduction/architecture/)
- [GitHub Agentic Workflows - Safe Outputs](https://github.github.com/gh-aw/reference/safe-outputs/)
- [GitHub Agentic Workflows - Threat Detection](https://github.github.com/gh-aw/reference/threat-detection/)
- [OpenHands - Agent Server](https://docs.openhands.dev/sdk/arch/agent-server)
- [OpenHands - Agent architecture](https://docs.openhands.dev/sdk/arch/agent)
- [OpenHands - Events](https://docs.openhands.dev/sdk/arch/events)
- [OpenHands - Conversation persistence](https://docs.openhands.dev/sdk/guides/convo-persistence)
- [OpenHands - Observability](https://docs.openhands.dev/sdk/guides/observability)
- [OpenHands - MCP](https://docs.openhands.dev/sdk/guides/mcp)
- [AgentOps decorators and hierarchy](https://docs.agentops.ai/v2/concepts/decorators)
- [StreamNative Agent Engine](https://streamnative.io/blog/introducing-the-streamnative-agent-engine)
- [From Functions to Agents](https://streamnative.io/blog/from-functions-to-agents-what-changes-in-the-runtime)
- [Faramesh - control plane paper](https://arxiv.org/abs/2601.17744)
- [Forrester - agent control planes](https://www.forrester.com/blogs/agent-control-planes-still-need-a-robust-standards-stack/)

It also takes seriously real-world failure reports such as:

- [Claude Code issue #7881 - shared session IDs for subagents](https://github.com/anthropics/claude-code/issues/7881)
- [Claude Agent SDK issue #47 - session management in ephemeral environments](https://github.com/anthropics/claude-agent-sdk-typescript/issues/47)
- [OpenHands issue #7175 - context resets](https://github.com/All-Hands-AI/OpenHands/issues/7175)
- [Claude Code issue #11950 - hide subagent internals](https://github.com/anthropics/claude-code/issues/11950)

---

## 5. Runtime Philosophy

The final ZAOS runtime should be understood as a **control plane for agentic work**, not as a chat wrapper.

Its job is to:

- route work to the right runtime and role
- maintain durable identity and continuity
- govern tool access and mutation
- emit and retain structured traces
- turn raw executions into supervised operations
- feed the cockpit with reliable read models
- keep provider-specific mechanics away from product truth

The runtime should therefore be:

- `Claude-first` in initial implementation
- `provider-agnostic-ready` by architecture
- durable by default
- event-driven
- policy-aware
- replayable
- observable
- safe by execution design, not by prompting alone

---

## 6. The Final ZAOS Runtime Architecture

The strongest design for ZAOS is a control plane composed of seven major blocks.

### 6.1 Supervisor and Orchestrator

This is the command layer of the runtime.

Responsibilities:

- receive workflow intent from ZAOS
- choose the next persona or subagent
- decide between direct execution, delegation, approval, or escalation
- manage retries and stuck handling
- bind work to the current workflow phase and gate
- convert product-level tasks into runtime-level runs

This layer is the owner of orchestration logic.
It must not directly own provider-specific implementation details.

### 6.2 Runtime Adapter Layer

Each provider or execution backend must be hidden behind a runtime adapter.

Examples:

- Claude adapter
- OpenAI adapter
- local scripted worker adapter
- deterministic checker adapter
- future remote runtime adapter

Responsibilities:

- normalize message exchange
- normalize tool invocation results
- normalize tracing events
- normalize subagent handoff contracts
- surface provider-specific limits without leaking them into core ZAOS state

This is what allows ZAOS to remain `Claude-first` without becoming Claude-bound.

### 6.3 Session Manager and Checkpoint Layer

The runtime must treat the session as a durable operational object.

Responsibilities:

- create and track session state
- track active run, task, gate, and persona
- persist checkpoints between important runtime transitions
- support pause, resume, replay, and fork
- bind approvals and handoffs to stable continuity objects

The session manager must never rely only on local transient provider state.
ZAOS must own its own continuity model.

### 6.4 Event Bus and Append-Only Runtime Log

Every significant runtime action should emit a structured event.

Examples:

- session started
- task bound
- subagent spawned
- tool call requested
- tool call approved or denied
- file change proposed
- file change applied
- test run completed
- gate passed or blocked
- stuck condition detected
- run escalated
- session closed

This event stream should be append-only and immutable at the log level.

Its purpose is to drive:

- replay
- audit
- debugging
- supervision
- metrics
- cockpit read models
- memory ingestion

### 6.5 Policy, Permission, and Approval Layer

The runtime must contain an executable governance layer.

Responsibilities:

- enforce permissions by phase, persona, tool, and risk level
- evaluate hooks such as `PreToolUse`, `PostToolUse`, and task completion checks
- require approval for sensitive mutations
- deny prohibited paths, tools, or actions
- attach reasons and policy decisions to events
- record overrides and escalation outcomes

This layer should combine:

- hook execution
- policy-as-code
- human approval checkpoints
- deterministic runtime checks

### 6.6 Safe-Output Writer and Mutation Boundary

The runtime should not let agents write directly into protected systems without a separate application boundary.

The safe-output writer exists to:

- stage proposed changes
- vet outputs before application
- run mutation checks
- apply approved writes in controlled order
- attach provenance and validation metadata to mutations

This pattern is especially important for:

- code edits
- infrastructure config changes
- secrets handling
- deployment actions
- external API mutations

The guiding rule is:

**agents propose, the runtime applies.**

### 6.7 Observers and Read Models

The control plane should expose read-only observer services that consume runtime events without mutating runtime truth.

Examples:

- cockpit view model generator
- risk monitor
- stuck detector
- replay builder
- metrics and analytics service
- session summarizer
- memory ingestion service
- audit and forensic views

Observers must remain downstream of the event log.
They may derive views, but they do not own the runtime state machine.

---

## 7. Identity Model

One of the clearest lessons from real-world failures is that identity must be explicit.

The final ZAOS runtime should at minimum track:

- `project_id`
- `workflow_id`
- `epic_id`
- `task_id`
- `session_id`
- `run_id`
- `agent_id`
- `subagent_id`
- `checkpoint_id`
- `approval_id`
- `trace_id`
- `artifact_id`

Important rules:

- a subagent must not be identified only through the parent session
- a task must remain distinguishable from a run
- a checkpoint must identify a resumable point, not just a timestamp
- approvals and denials must bind to the exact action instance they govern

Without stable identity, the runtime becomes hard to supervise, replay, or secure.

---

## 8. Session, Resume, Replay, and Fork

The runtime must support four distinct continuity operations.

### 8.1 Live Execution

The active runtime state used during current execution.

### 8.2 Resume

Continue an interrupted or paused session from the last valid checkpoint.

### 8.3 Replay

Reconstruct what happened for audit, debugging, learning, or validation.

### 8.4 Fork

Branch from a prior checkpoint or session state to explore an alternative path without corrupting the main line.

These operations must remain separate in the runtime design.
Replay is not resume.
Fork is not retry.
Snapshot is not live state.

---

## 9. Event Model

The event model is one of the most important parts of the runtime.

Each event should include:

- event type
- event time
- actor identity
- target identity
- workflow phase
- task and gate context
- policy context
- input and output references
- trace linkage
- approval linkage if applicable
- artifact linkage if created

Examples of event families:

- lifecycle events
- delegation events
- tool events
- mutation events
- approval events
- validation events
- security events
- checkpoint events
- escalation events
- closure events

The runtime should prefer many small structured events over a few opaque transcripts.

---

## 10. Subagents and Bounded Execution

Subagents are useful only when they create a meaningful boundary.

Valid reasons to spawn a subagent include:

- context isolation
- narrower permissions
- specialized role behavior
- independent sidecar work
- bounded research or verification
- disjoint implementation ownership

Subagents should not be used merely to create the appearance of sophistication.

Every subagent should have:

- a declared role
- an explicit scope
- stable identity
- a bounded permission profile
- a result contract
- a completion summary or artifact

Subagent internals should not flood the cockpit by default.
The system should show the outcome first and allow drill-down when needed.

---

## 11. Workspace, Sandbox, and Safe Side Effects

The runtime must control side effects deliberately.

Recommended principles:

- read-only by default
- staged writes before application
- protected paths for sensitive files
- separate mutation channels for code, config, and deployment
- sandboxed execution where possible
- explicit approval for risky external actions
- deterministic checks before apply

For parallel work, the runtime should support isolated workspaces or worktrees when needed so that multiple agents do not collide by default.

---

## 12. Provider-Agnostic-Ready Architecture

The control plane should not bind workflow truth to one provider's conversation format or session behavior.

Provider differences should be absorbed by adapters:

- message formatting
- tool schemas
- session mechanics
- trace shapes
- approval semantics
- subagent mechanics
- token and cost metadata

This is not a demand for immediate multi-provider parity.
It is a design constraint that prevents future architectural lock-in.

---

## 13. Runtime Hooks and Approval Flow

The runtime should expose a small number of execution-critical hook phases.

Recommended hook surfaces:

- `BeforeRun`
- `BeforeDelegation`
- `PreToolUse`
- `PostToolUse`
- `BeforeMutationApply`
- `BeforeGateEvaluation`
- `OnStuckDetection`
- `OnEscalation`
- `OnTaskClosure`
- `OnSessionClosure`

The runtime should also support approval states such as:

- requested
- approved
- denied
- expired
- superseded
- auto-approved by policy

These approval records must be durable and replayable.

---

## 14. Runtime to Cockpit Contract

The cockpit must not scrape the runtime ad hoc.

Instead, the runtime should publish clean read models for:

- current workflow state
- active session state
- active task and gate
- pending approvals
- recent evidence
- current risks
- subagent status
- stuck conditions
- replay availability
- policy denials and violations

This keeps the cockpit decision-first and prevents UI logic from becoming a hidden orchestration layer.

---

## 15. Anti-Patterns to Avoid

ZAOS should explicitly reject these runtime anti-patterns:

- one agent doing orchestration, coding, review, validation, and closure alone
- runtime state hidden inside prompts or provider-local sessions
- direct writes from the model into sensitive systems
- hooks that observe but cannot enforce
- shared ambiguous identities for subagents
- raw transcript dumping as a substitute for structured events
- retries without checkpoints
- replay that mutates live state
- UI polling used as the primary source of truth
- provider-specific logic leaking into workflow truth

---

## 16. Definition of Done

The ZAOS runtime/control plane can be considered complete only when all of the following are true in practice:

- the orchestrator can manage workflow execution without becoming a god agent
- provider-specific runtimes are hidden behind adapters
- sessions, runs, checkpoints, and subagents have stable identities
- all important runtime actions emit structured events
- the system can resume, replay, and fork without corrupting live state
- policies and approvals are executable and durable
- risky mutations pass through a safe-output application boundary
- observers can feed the cockpit without mutating runtime truth
- the runtime remains usable for both normal task execution and sensitive operations
- the architecture is Claude-first but not permanently provider-locked

---

## 17. Final Statement

The ultimate ZAOS runtime should be:

**a provider-agnostic-ready agentic control plane built around a central supervisor, bounded subagents, runtime adapters, durable sessions, append-only events, executable policy, safe-output application, and read-only observers, so that agentic work becomes governable, replayable, and safe enough for professional software delivery.**

