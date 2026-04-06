# ZAOS - Ultimate Workflow Vision

## 1. Purpose

This document defines the target workflow architecture for the final version of ZAOS.

It answers a core product question:

**What kind of workflow system is required if ZAOS is to make agentic software development reliable enough for professional delivery?**

The answer is not a monolithic plan, not a free-form chat loop, and not an unconstrained autonomous swarm.

The answer is a **governed workflow system** with:

- a central orchestrator
- specialized personas
- a macro-pipeline for product framing
- a micro-loop for task execution
- runtime guardrails
- deterministic validation
- security gates
- traceability
- escalation rules
- resumable artifacts

---

## 2. Why ZAOS Needs an Ultimate Workflow

LLMs do not fail only because they forget facts.
They also fail because they operate poorly without structure.

Their workflow weaknesses include:

- starting to code before understanding
- collapsing research, planning, coding, and judgment into one role
- retrying blindly after failure
- over-trusting their own intermediate conclusions
- declaring success without evidence
- bypassing process under pressure
- drifting away from original intent during long sessions
- handling sensitive operations without reliable safety checks

For ZAOS to be a professional delivery cockpit, workflow must not be an aesthetic layer.
It must be one of the main control systems of the product.

---

## 3. Core Design Principles

The final ZAOS workflow must follow these principles:

- workflow beats improvisation
- planning and execution should be separable
- execution should happen in short bounded loops
- each role should have a clear responsibility
- validation must rely on proof, not persuasion
- hooks and policies must enforce what prompts alone cannot enforce
- traces and checkpoints are mandatory
- security-sensitive actions must be governed explicitly
- failure must trigger diagnosis, not blind retry
- resumption must be artifact-based

---

## 4. External Patterns This Builds On

The ZAOS workflow vision synthesizes strong recurring patterns from:

- [OpenAI practical guide to building agents](https://openai.com/business/guides-and-resources/a-practical-guide-to-building-ai-agents/)
- [OpenAI Agents SDK](https://openai.github.io/openai-agents-python/)
- [OpenAI guardrails](https://openai.github.io/openai-agents-python/guardrails/)
- [OpenAI trace grading](https://developers.openai.com/api/docs/guides/trace-grading)
- [Claude Code sub-agents](https://code.claude.com/docs/en/sub-agents)
- [Claude Code hooks](https://code.claude.com/docs/en/hooks)
- [Anthropic - Building agents with the Claude Agent SDK](https://claude.com/blog/building-agents-with-the-claude-agent-sdk/)
- [Anthropic - Demystifying evals for AI agents](https://www.anthropic.com/engineering/demystifying-evals-for-ai-agents)
- [GitHub Agentic Workflows](https://github.github.com/gh-aw)
- [GitHub threat detection](https://github.github.com/gh-aw/reference/threat-detection/)
- [Cline Plan & Act](https://docs.cline.bot/core-workflows/plan-and-act)
- [OpenHands custom agent workflow](https://docs.openhands.dev/sdk/guides/agent-custom)
- [aider linting and testing](https://aider.chat/docs/usage/lint-test.html)
- [OWASP AI Agent Security Cheat Sheet](https://cheatsheetseries.owasp.org/cheatsheets/AI_Agent_Security_Cheat_Sheet.html)
- [AWS idempotency guidance](https://docs.aws.amazon.com/wellarchitected/latest/framework/rel_prevent_interaction_failure_idempotent.html)

These sources converge toward a common model:

- central orchestration
- role specialization
- policy-enforced tool usage
- bounded loops
- explicit validation
- observability
- resumption

---

## 5. High-Level Workflow Model

The final ZAOS workflow should operate at two levels:

1. Macro-Pipeline
2. Task Micro-Loop

This dual structure is essential.

The macro-pipeline frames the project globally.
The micro-loop executes work safely and incrementally.

---

## 6. Macro-Pipeline

### 6.1 Role

The macro-pipeline creates the trusted frame in which execution can happen.

Its function is to prevent chaos before implementation starts.

### 6.2 Stages

The recommended final macro-pipeline is:

1. Intake
2. Research
3. Product Contract
4. Architecture
5. Security Framing / Threat Model when needed
6. Epic Planning
7. Task Planning
8. Release Strategy

### 6.3 Intake

Purpose:

- capture the request
- identify business goals
- identify users
- identify constraints
- determine risk level
- identify expected quality bar

### 6.4 Research

Purpose:

- clarify unknowns
- compare options
- verify external constraints
- identify risk areas
- reduce avoidable implementation improvisation

### 6.5 Product Contract

Purpose:

- define scope
- define outcomes
- define constraints
- define acceptance criteria
- define non-functional requirements
- define security expectations

### 6.6 Architecture

Purpose:

- define structure
- define data flows
- define dependencies
- define integration points
- expose major technical risks

### 6.7 Security Framing

Purpose:

- identify sensitive zones
- determine whether auth, payments, secrets, permissions, or compliance are in scope
- define required security depth for the project

### 6.8 Epic Planning

Purpose:

- divide the product into bounded delivery units
- define milestone-level progress

### 6.9 Task Planning

Purpose:

- divide epics into concrete executable tasks
- attach persona ownership
- attach expected evidence
- attach closure conditions

### 6.10 Release Strategy

Purpose:

- define readiness criteria
- define testing expectations
- define maintenance needs
- define supportability and rollback expectations

### 6.11 What the Macro-Pipeline Is Not

It is not meant to produce one giant implementation batch.

It exists to create a stable and auditable execution frame.

---

## 7. Task Micro-Loop

### 7.1 Role

The task micro-loop is where actual software delivery happens.

It must be:

- short
- bounded
- evidence-driven
- resumable

### 7.2 Target Task Loop

Each task should move through a loop like this:

1. task comprehension
2. memory recall and relevant context loading
3. local research and exploration when necessary
4. mini implementation plan
5. implementation
6. review
7. test
8. if failure: diagnosis
9. targeted return to implementation
10. re-review when needed
11. re-test
12. task closure with evidence
13. memory and workflow update
14. next task start

### 7.3 Why This Structure Matters

ZAOS explicitly rejects the following model:

- define the entire product
- implement the entire product
- review only at the end
- test only at the end

That monolithic structure increases:

- drift
- context loss
- regression accumulation
- weak traceability
- weak resumption
- false progress signals

The target is:

- global framing first
- then repeated short loops of verified delivery
- task by task
- until epic completion
- then until product completion

### 7.4 Local Research and Exploration

The local research step is intentionally narrow.

It exists for task-level needs such as:

- checking an API or library behavior
- exploring an unfamiliar code path
- confirming an integration constraint
- locating the correct implementation point

It is not meant to reopen broad product framing during task execution.

### 7.5 Failure Handling

Failure must not lead directly to retry.

Failure must lead to:

- diagnosis
- causal hypothesis
- targeted correction
- explicit re-validation

Blind retry is an anti-pattern.

---

## 8. Central Orchestration Model

### 8.1 Why Orchestration Must Be Central

The most robust pattern found across sources is a central orchestrator with specialized delegated work.

ZAOS should not rely on a fully decentralized swarm.

### 8.2 Orchestrator Responsibilities

The orchestrator should:

- maintain the overall workflow state
- choose the right persona for the current step
- manage transitions and gates
- preserve alignment with product intent
- request evidence before closure
- escalate when proof is weak or loops emerge
- avoid absorbing every role itself

### 8.3 Orchestrator Constraints

The orchestrator should not become a hidden super-agent.

It should avoid:

- coding by default
- reviewing its own implementation
- validating its own claims
- making security calls without dedicated security paths

---

## 9. Persona Model

### 9.1 Final Persona Set

The final ZAOS workflow should support at least:

- `Orchestrator`
- `Researcher`
- `Product Strategist`
- `Architect`
- `Planner / Tech Lead`
- `Coder`
- `Reviewer`
- `Deterministic Tester`
- `Debugger`
- `AppSec Reviewer`
- `Auth / Payments Specialist`
- `Release Manager`
- `Maintainer`

### 9.2 Why Personas Matter

Personas are not aesthetic labels.

They reduce workflow failure by:

- isolating concerns
- isolating context
- preventing one agent from acting as planner, implementer, reviewer, and judge at once
- making handoffs auditable

### 9.3 Persona Principles

- one task should have one active owner persona
- secondary personas may contribute through bounded delegation
- each persona should have clear permitted actions
- each persona should have explicit handoff conditions

---

## 10. Plan Mode and Act Mode

### 10.1 Rationale

Multiple ecosystems converge on a useful split between:

- read-only exploration and planning
- active execution and mutation

### 10.2 Plan Mode

Plan Mode should allow:

- reading
- exploring
- comparing options
- clarifying scope
- preparing implementation plans
- identifying risks

It should restrict:

- code mutation
- release actions
- high-impact tool use

### 10.3 Act Mode

Act Mode should allow:

- writing
- editing
- running controlled commands
- testing
- implementing task plans

It should remain bounded by:

- workflow phase
- persona permissions
- policy layer
- guardrails

### 10.4 Product Role

This split helps reduce:

- premature coding
- unexamined assumptions
- repair loops caused by shallow understanding

---

## 11. Hooks and Guardrails

### 11.1 Role

Hooks are the runtime guardrail layer of ZAOS.

Prompts can suggest behavior.
Hooks can enforce behavior.

### 11.2 Why Hooks Matter

The workflow becomes reliable only when the runtime can:

- block
- defer
- inject context
- trigger checks
- record evidence
- stop unsafe transitions

### 11.3 Recommended Hook Categories

#### `PreToolUse`

Checks:

- policy compliance
- active phase
- persona permissions
- path safety
- file sensitivity
- secret exposure risks
- network and MCP permission rules

#### `PostToolUse`

Handles:

- result parsing
- error extraction
- evidence capture
- state updates
- anomaly signals

#### `OnWrite / OnEdit`

Handles:

- restricted files
- out-of-phase writes
- drift against task scope
- structured mutation receipts

#### `OnTaskClosure`

Checks:

- required evidence
- tests run status
- review status
- unresolved blockers
- memory update completeness

#### `OnGateValidation`

Checks:

- gate criteria
- required artifacts
- open risks
- required human approvals

#### `OnSubagentStop`

Handles:

- collection of relevant outputs
- summary extraction
- transcript compaction
- handoff artifact creation

#### `AsyncTestHook`

Handles:

- background lint
- unit tests
- integration tests
- smoke checks
- targeted security checks

#### `SecurityHook`

Handles:

- auth paths
- payment paths
- webhook paths
- secret handling
- permission changes
- sensitive config changes

#### `DriftHook`

Handles:

- implementation drift against product contract
- workflow drift against expected phase
- memory drift signals

#### `LoopGuardHook`

Handles:

- repeated retries
- no-progress cycles
- failure clustering
- excessive token burn without advancement

### 11.4 Hook Philosophy

Hooks should be deterministic whenever possible.

They should prefer:

- explicit rules
- structured checks
- machine-observable conditions

over:

- free-form interpretation by the same model being constrained

---

## 12. Safety and Security Model

### 12.1 Defense in Depth

The final ZAOS workflow should use layered safety:

- least privilege
- explicit tool approvals
- structured inputs and outputs
- path and file protection
- prompt injection defenses
- sandboxing
- threat detection
- human review on high-impact actions

### 12.2 Sensitive Workflows

The workflow should treat these as sensitive by default:

- authentication
- session handling
- authorization
- payments
- webhook handling
- secret storage
- deployment
- destructive migrations
- CI or infrastructure mutations

### 12.3 Safety Requirements

For sensitive actions, ZAOS should require:

- correct persona path
- gated execution
- stronger approvals
- stronger evidence requirements
- explicit review
- clear rollback awareness

---

## 13. Deterministic Validation

### 13.1 Core Rule

An agent must not be able to certify its own work through narrative alone.

### 13.2 Validation Sources

ZAOS should rely on:

- test execution
- lint and static checks
- runtime checks
- CI results
- screenshots when relevant
- diffs
- logs
- review findings
- structured trace graders where appropriate

### 13.3 Validation Principles

- no proof, no closure
- no test claim without observable execution
- no review closure without explicit review signal
- no gate progression on unverified optimism

### 13.4 Trace-Based Validation

Where possible, ZAOS should validate not only final outputs but traces of behavior:

- tool usage
- command outputs
- mutation patterns
- handoff outcomes
- policy triggers

---

## 14. Observability and Traceability

### 14.1 Role

Tracing is not optional.
It is the substrate of trust in the workflow.

### 14.2 What Must Be Traceable

- prompts and task framing
- persona handoffs
- tool calls
- approvals
- hook decisions
- tests executed
- validation outcomes
- escalation events
- closure decisions

### 14.3 Product Function

The cockpit must be built on traces, not on optimistic summaries.

Tracing supports:

- debugging
- audits
- evidence retrieval
- session replay
- workflow improvement

---

## 15. Checkpoints and Resumption

### 15.1 Role

Every task should be resumable without replaying the full conversation.

### 15.2 Checkpoint Requirements

Each task should be able to persist:

- current state
- last known plan
- current blockers
- last evidence set
- current risks
- next recommended step
- responsible persona

### 15.3 Product Function

Checkpoints make the workflow:

- robust to interruption
- robust to context limits
- robust to runtime restarts
- easier to maintain over long projects

---

## 16. Circuit Breakers and Escalation

### 16.1 Why They Matter

An agentic workflow without stopping rules will eventually drift, loop, or fake progress.

### 16.2 Circuit Breaker Triggers

ZAOS should support escalation on:

- repeated failed tests
- repeated no-progress edits
- repeated retries on the same failure
- missing evidence after expected closure point
- unresolved contradiction between review and implementation
- abnormal token burn without progress
- sensitive action requiring human confirmation

### 16.3 Escalation Outcomes

Escalation may result in:

- pause
- required diagnosis
- human review
- persona switch
- return to planning
- gate denial

### 16.4 Idempotency Principle

Mutating actions should be designed with idempotency in mind wherever possible.

This is especially important for:

- deployments
- webhooks
- migrations
- external API effects
- retryable actions

---

## 17. Workflow Artifacts

The final workflow should leave behind durable, auditable artifacts.

These should include:

- product contract
- architecture notes
- epic and task definitions
- plan artifacts
- review findings
- test outputs
- threat model outputs
- closure receipts
- session snapshots
- maintenance lessons

Artifact-based workflow is more reliable than chat-only workflow.

---

## 18. Anti-Patterns to Explicitly Reject

ZAOS should explicitly reject:

- one giant implementation pass
- one agent doing all roles at once
- unconstrained retries after failure
- passing gates without evidence
- validation by persuasive prose
- silent mutation of sensitive files
- security postponed until the end
- context flooding instead of targeted routing
- closure without memory or artifact updates

---

## 19. Product Definition of Done for Workflow

The workflow layer of ZAOS can be considered complete when all of the following are true in practice:

- the product supports a full macro-pipeline from intake to release strategy
- execution happens through bounded task loops rather than monolithic implementation
- personas have clear responsibilities and enforceable boundaries
- hooks and policies reliably govern sensitive operations
- validation depends on deterministic evidence
- traces make workflow behavior auditable
- checkpoints make long-running work resumable
- circuit breakers prevent infinite loops and fake progress
- security-sensitive work is handled through dedicated workflow paths
- maintenance and bug fixing are first-class workflow citizens

---

## 20. Final Workflow Statement

The ultimate ZAOS workflow should be:

**a two-level governed workflow in which a central orchestrator frames the project through a macro-pipeline, then drives short task-level micro-loops executed by specialized personas, constrained by hooks and policies, observed through traces, validated by deterministic evidence, protected by security gates, and stopped by explicit escalation rules whenever drift, loops, or insufficient proof appear.**

---

## 21. Final Recommendation

If the final ZAOS workflow had to be summarized in one synthesis line:

- OpenAI for orchestration, guardrails, and trace thinking
- Anthropic for subagents and hook-driven runtime control
- GitHub Agentic Workflows for security and threat detection
- Cline and OpenHands for plan/act separation
- aider for practical validation loops
- OWASP and AWS for safety, abuse resistance, and idempotent operations

That combination is the strongest candidate for the ultimate workflow architecture of ZAOS.
