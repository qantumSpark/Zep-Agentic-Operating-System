# ZAOS - Ultimate Policy System Vision

## 1. Purpose

This document defines the target policy, rules, hooks, and approvals architecture for the final version of ZAOS.

It answers a central product question:

**How should ZAOS govern agent behavior so that important rules are not merely suggested, but actually enforced?**

The answer is not:

- a giant prompt file
- one global rules document
- informal instructions with no runtime effect
- UI warnings without execution control

The answer is a **hierarchical policy system** combining:

- prompt guidance
- scoped rules
- runtime hooks
- deterministic policy-as-code
- approvals and escalation
- subagent boundaries
- auditability and policy testing

---

## 2. Why ZAOS Needs an Ultimate Policy System

LLMs do not reliably obey complex instructions just because those instructions exist.

When policy relies only on prompts, systems tend to fail through:

- context overload
- contradictory rules
- ignored instructions
- unsafe shortcuts
- silent drift
- uncontrolled tool use
- overreach into sensitive files or actions

For a professional agentic cockpit, policy must not be implicit.
It must be executable and layered.

ZAOS therefore needs a policy system that can:

- guide the model
- restrict the runtime
- block unsafe actions
- trigger approvals
- test non-negotiable rules
- scale across personas, phases, files, and projects

---

## 3. Core Design Principles

The final ZAOS policy system must follow these principles:

- prompts guide, but do not enforce
- rules should be short, scoped, and composable
- runtime controls should block what prompts cannot
- non-negotiable rules should be deterministic and testable
- approvals should exist for high-impact actions
- persona boundaries should be permission boundaries
- policy should be phase-aware and risk-aware
- third-party tools should be untrusted by default
- sensitive paths should be explicitly protected
- policy behavior should be auditable

---

## 4. External Patterns This Builds On

The ZAOS policy vision synthesizes strong recurring patterns from:

- [Claude Code settings](https://code.claude.com/docs/en/settings)
- [Claude Code hooks](https://code.claude.com/docs/en/hooks)
- [Claude Code hooks guide](https://code.claude.com/docs/en/hooks-guide)
- [Claude Code sub-agents](https://code.claude.com/docs/en/sub-agents)
- [Claude Code security](https://code.claude.com/docs/en/security)
- [Claude Code memory](https://code.claude.com/docs/en/memory)
- [OpenAI Safety in building agents](https://developers.openai.com/api/docs/guides/agent-builder-safety)
- [OpenAI Guardrails](https://openai.github.io/openai-agents-js/guides/guardrails/)
- [OpenAI Guardrails JS](https://openai.github.io/openai-guardrails-js/)
- [OpenAI Guardrails evals](https://openai.github.io/openai-guardrails-js/evals/)
- [GitHub Agentic Workflows architecture](https://github.github.com/gh-aw/introduction/architecture/)
- [GitHub Safe Outputs](https://github.github.com/gh-aw/reference/safe-outputs/)
- [GitHub Threat Detection](https://github.github.com/gh-aw/reference/threat-detection/)
- [Cursor rules](https://docs.cursor.com/en/context)
- [Cline Rules](https://docs.cline.bot/customization/cline-rules)
- [Cline Conditional Rules](https://docs.cline.bot/features/conditional-rules)
- [Cline Hooks](https://docs.cline.bot/customization/hooks)
- [Cline Memory Bank](https://docs.cline.bot/features/memory-bank)
- [OPA Policy Language](https://www.openpolicyagent.org/docs/policy-language)
- [OPA policy testing](https://www.openpolicyagent.org/docs/latest/policy-testing/)
- [Conftest](https://github.com/open-policy-agent/conftest)
- [Conftest docs](https://www.conftest.dev/)
- [OWASP AI Agent Security Cheat Sheet](https://cheatsheetseries.owasp.org/cheatsheets/AI_Agent_Security_Cheat_Sheet.html)

These sources converge on a strong pattern:

- guidance in prompt-like memory
- runtime enforcement via hooks and permissions
- deterministic policy for critical rules
- explicit approvals for impact-heavy actions
- safe mutation boundaries

---

## 5. Policy Philosophy

The most important general rule for ZAOS is:

**Guidance tells the agent what should happen. Policy determines what may happen.**

This means:

- prompts are for intent and style
- hooks are for runtime control
- policy-as-code is for non-negotiable constraints
- approvals are for human arbitration
- subagents are for isolating responsibility and privilege

That hierarchy is the backbone of the final ZAOS policy system.

---

## 6. The Five-Layer Policy Stack

The final ZAOS policy architecture should be built as five main layers:

1. Prompt Guidance Layer
2. Scoped Rules Layer
3. Hook Enforcement Layer
4. Deterministic Policy Layer
5. Approval and Escalation Layer

Subagent boundaries act as an additional structural control across all layers.

---

## 7. Layer 1 - Prompt Guidance Layer

### 7.1 Role

Prompt guidance exists to shape behavior, not to guarantee it.

### 7.2 Typical Contents

- intent
- examples
- conventions
- architectural preferences
- writing style
- desired process behavior

### 7.3 Product Rule

Prompt guidance should never be the only control used for high-impact or safety-critical rules.

### 7.4 Why It Still Matters

Prompt guidance is useful for:

- alignment
- productivity
- consistency
- smoother handoffs

It just must not be confused with enforcement.

---

## 8. Layer 2 - Scoped Rules Layer

### 8.1 Role

Scoped rules provide targeted context without overloading the agent.

### 8.2 Good Rule Design

Rules should be:

- short
- versioned
- explicit
- scoped
- composable

### 8.3 Typical Scopes

- project-wide
- persona-specific
- phase-specific
- path-specific
- domain-specific
- user-level

### 8.4 Why Scoping Matters

Large monolithic rule files degrade over time through:

- conflict
- irrelevance
- dilution in context
- poor compliance

Scoped rules reduce cognitive and contextual noise.

### 8.5 Product Rule

Rules should be injected or activated only when relevant to the current work.

---

## 9. Layer 3 - Hook Enforcement Layer

### 9.1 Role

Hooks are the runtime enforcement layer.

They turn policy from suggestion into action.

### 9.2 What Hooks Must Be Able to Do

- allow
- deny
- ask for approval
- defer
- enrich context
- trigger checks
- stop closure

### 9.3 Recommended Hook Points

- `PreToolUse`
- `PermissionRequest`
- `PostToolUse`
- `SubagentStart`
- `SubagentStop`
- `TaskCompleted`
- `Stop`
- `SessionStart`
- `SessionEnd`
- `PreCompact`

### 9.4 Typical Hook Responsibilities

- block disallowed file access
- block disallowed tool use
- enforce phase constraints
- enforce persona constraints
- capture proofs
- trigger tests
- enforce closure evidence
- detect loop or drift conditions

### 9.5 Product Rule

If a rule must hold at execution time, it should be backed by hooks or deterministic policy, not only by prompt text.

---

## 10. Layer 4 - Deterministic Policy Layer

### 10.1 Role

The deterministic policy layer handles non-negotiable rules in a testable way.

### 10.2 Suitable Tools

Good fits include:

- OPA / Rego
- Conftest
- structured validators
- repo policy engines

### 10.3 What This Layer Should Govern

- permissions
- MCP allowlists
- protected files
- sandbox modes
- safe output requirements
- CI policy checks
- structured config validation
- workflow policy conformance

### 10.4 Why It Matters

Deterministic policy gives ZAOS:

- reproducibility
- CI testability
- explicit failure modes
- enforceable guardrails

### 10.5 Product Rule

If a rule can be expressed as deterministic logic on a structured artifact, it should live here.

---

## 11. Layer 5 - Approval and Escalation Layer

### 11.1 Role

Some decisions should not be automatic.

Approvals and escalation exist for:

- high-impact actions
- ambiguous situations
- security-sensitive changes
- policy conflicts
- suspicious behavior

### 11.2 Typical Approval Triggers

- secrets
- auth
- payments
- webhooks
- deployment
- destructive file operations
- high-risk MCP actions
- protected file edits

### 11.3 Typical Escalation Triggers

- repeated rule violations
- insufficient proof at closure
- suspicious mutation patterns
- unresolved contradiction
- repeated loop without progress

### 11.4 Product Rule

Approvals should not be a fallback for weak policy.
They should be the human boundary for impactful or ambiguous cases.

---

## 12. Subagents as Policy Boundaries

### 12.1 Role

Subagents are not only workflow tools.
They are policy boundaries.

### 12.2 Why They Matter

Subagents allow ZAOS to isolate:

- context
- responsibility
- tool access
- MCP scope
- privilege level

### 12.3 Example Restrictions

- reviewer: read-only tools only
- researcher: no write access
- coder: write allowed but under specific hooks
- maintainer: elevated write, stronger approvals
- AppSec reviewer: security-focused access, no arbitrary product mutation by default

### 12.4 Product Rule

Personas should not share the same unrestricted capability set.

---

## 13. Permissions and Modes

### 13.1 Role

Modes translate policy posture into runtime behavior.

### 13.2 Common Modes

- plan / read-only
- controlled write
- accept edits
- restricted execution
- elevated execution under approval

### 13.3 Why Modes Matter

Modes provide a simple operational summary of policy posture.

### 13.4 Product Rule

Dangerous or bypass-style modes should never be the normal operating default.

---

## 14. Protected Files and Sensitive Paths

### 14.1 Role

Some files and paths should be protected by policy, not merely by convention.

### 14.2 Examples

- `.env`
- secrets and credentials
- auth configuration
- deployment config
- CI workflows
- payment and billing config
- workflow and policy files
- memory files considered canonical

### 14.3 Typical Controls

- deny
- approval required
- reviewer-only access
- stage-and-vet write path

### 14.4 Product Rule

Sensitive files should be explicitly protected at the settings or policy layer.

---

## 15. MCP Governance

### 15.1 Role

Third-party MCP servers expand capability and attack surface.

### 15.2 Core Rule

MCP servers should be treated as untrusted until explicitly governed.

### 15.3 Required Controls

- allowlisting
- scope limitation
- approval for sensitive calls
- traceability
- persona-aware access
- sandbox-aware usage

### 15.4 Product Rule

An MCP integration should never be trusted merely because it exists in the environment.

---

## 16. Safe Outputs and Mutation Boundaries

### 16.1 Role

Policy must separate generation from mutation.

### 16.2 Preferred Model

The preferred pattern is:

- generate safely
- validate outputs
- apply writes through controlled paths

### 16.3 Why This Matters

This reduces:

- prompt injection damage
- direct mutation risk
- policy bypass
- unsafe propagation of untrusted data

### 16.4 Product Rule

High-impact writes should prefer staged and vetted output paths over direct application.

---

## 17. Policy by Phase

### 17.1 Role

Different workflow phases require different policy posture.

### 17.2 Example Phase Postures

#### Research

- broad read
- no write by default
- strong untrusted-input handling

#### Planning

- structured artifact production
- no arbitrary implementation writes

#### Implementation

- controlled write
- test triggering
- protected files guarded

#### Review

- mostly read-only
- critique allowed
- limited mutation unless explicitly authorized

#### Test

- execute checks
- reject closure without evidence

#### Security Review

- elevated inspection
- stronger gates
- specialist persona path

#### Release

- highest caution
- approvals
- deployment-specific controls

### 17.3 Product Rule

Policy should adapt to workflow phase rather than staying static across all tasks.

---

## 18. Policy by Persona

### 18.1 Role

Personas should have differentiated policy envelopes.

### 18.2 Why It Matters

This helps prevent:

- privilege creep
- role confusion
- drift in responsibility

### 18.3 Product Rule

Persona identity should affect:

- tools allowed
- files allowed
- approval thresholds
- closure authority

---

## 19. Policy Testing and CI Integration

### 19.1 Role

Policies should be tested like code.

### 19.2 Recommended Practices

- version policy files
- test Rego or equivalent
- run Conftest or equivalent in CI
- validate structured outputs against policy
- fail CI on critical policy violations

### 19.3 Product Function

This ensures policy drift is caught before runtime.

### 19.4 Product Rule

Non-negotiable policies should not depend on manual review alone.

---

## 20. Auditing and Traceability

### 20.1 Role

Every important policy decision should be explainable later.

### 20.2 What Must Be Auditable

- why a tool was allowed or denied
- why a write required approval
- why a gate was blocked
- why a subagent received a given capability set
- why a policy test failed

### 20.3 Product Rule

If a policy decision cannot be inspected later, the system is not trustworthy enough for professional workflows.

---

## 21. What ZAOS Should Explicitly Take from Existing Systems

### 21.1 From Claude Code

- settings hierarchy
- allow / ask / deny permissions
- runtime hooks
- subagent-level isolation
- sandbox-aware controls

### 21.2 From OpenAI Guardrails

- input / output / tool guardrail separation
- structured safety controls

### 21.3 From GitHub Agentic Workflows

- safe outputs
- staged writes
- threat detection
- permissions discipline

### 21.4 From Cursor and Cline

- scoped rules
- conditional rule activation
- compact project guidance

### 21.5 From OPA and Conftest

- deterministic policy-as-code
- CI-testable policy enforcement

---

## 22. Anti-Patterns to Explicitly Reject

ZAOS should explicitly reject:

- relying on a giant prompt file as the main control system
- treating guidance as equivalent to enforcement
- dumping all rules into every context
- leaving sensitive files protected only by “don’t touch this” prose
- granting unrestricted write permissions by default
- trusting third-party MCP servers automatically
- bypass modes as normal workflow
- untested policies
- UI-only policy warnings without runtime enforcement

---

## 23. Product Definition of Done for the Policy System

The policy layer of ZAOS can be considered complete when all of the following are true in practice:

- guidance, enforcement, deterministic policy, approvals, and subagent boundaries are clearly separated
- critical rules are executable and testable
- hooks enforce runtime-sensitive policy
- personas and phases have differentiated controls
- protected files and sensitive paths are explicitly guarded
- MCP usage is governed and auditable
- safe-output patterns exist for high-impact mutations
- CI validates policy conformance
- policy decisions are traceable

---

## 24. Final Policy Statement

The ultimate ZAOS policy system should be:

**a hierarchical and executable control architecture in which prompt guidance shapes behavior, scoped rules reduce noise, hooks enforce runtime constraints, deterministic policy-as-code governs non-negotiable boundaries, approvals arbitrate high-impact actions, and subagents isolate context and privilege so that agent behavior remains governable, testable, and auditable.**

---

## 25. Final Recommendation

If the final ZAOS policy model had to be summarized in one synthesis line:

- Claude Code for settings, hooks, permissions, and subagent boundaries
- OpenAI Guardrails for structured guardrail thinking
- GitHub Agentic Workflows for staged writes and trust boundaries
- Cursor and Cline for scoped, conditional rule guidance
- OPA and Conftest for deterministic policy-as-code and CI enforcement

That combination is the strongest candidate for the ultimate policy architecture of ZAOS.
