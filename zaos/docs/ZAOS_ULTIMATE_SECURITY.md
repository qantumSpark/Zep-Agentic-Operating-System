# ZAOS - Ultimate Security Vision

## 1. Purpose

This document defines the target security architecture for the final version of ZAOS.

It answers a core product question:

**How should ZAOS secure agentic development workflows so that professional software, including auth, payments, webhooks, secrets, and deployments, can be handled with confidence?**

The answer is not a single prompt, a single model, or a single review checkbox.

The answer is a **secure SDLC for agents**, built on:

- least privilege
- staged writes
- explicit approvals
- structured outputs
- sandboxing
- secret isolation
- threat detection
- auditability
- deterministic validation
- human review on high-impact paths

---

## 2. Why ZAOS Needs an Ultimate Security Model

Agentic systems create a security problem that is different from traditional software:

- the model may follow plausible but unsafe paths
- untrusted inputs may influence tool use
- secrets may leak into traces or prompts
- writes may occur before review
- third-party MCP servers may expand the attack surface
- validation can become deceptive if the model is allowed to certify itself
- security-sensitive flows can look correct while missing critical controls

ZAOS is meant to help build professional software.
That means the product itself must treat security as a first-class workflow concern, not as a post-hoc checklist.

---

## 3. Security Goals

The final ZAOS security layer must achieve the following:

- protect secrets from direct exposure to the agent
- reduce the attack surface of tool usage
- ensure writes happen only through governed paths
- prevent prompt injection from steering sensitive actions
- keep authentication and session flows hard to break
- handle payments and webhooks safely
- make deployment actions auditable and gated
- prevent unsafe MCP expansion
- enforce least privilege by default
- preserve forensic evidence for every sensitive action

---

## 4. External Patterns This Builds On

The ZAOS security vision synthesizes strong recurring patterns from:

- [OWASP ASVS](https://owasp.org/www-project-application-security-verification-standard/)
- [OWASP AI Agent Security Cheat Sheet](https://cheatsheetseries.owasp.org/cheatsheets/AI_Agent_Security_Cheat_Sheet.html)
- [NIST Secure Software Development Framework](https://csrc.nist.gov/projects/ssdf)
- [OpenAI Safety in building agents](https://developers.openai.com/api/docs/guides/agent-builder-safety)
- [OpenAI agent evals](https://platform.openai.com/docs/guides/agent-evals)
- [OpenAI trace grading](https://platform.openai.com/docs/guides/trace-grading)
- [Anthropic Claude Code security](https://code.claude.com/docs/en/security)
- [Anthropic hooks guide](https://code.claude.com/docs/en/hooks-guide)
- [GitHub Agentic Workflows](https://github.github.com/gh-aw/)
- [GitHub threat detection](https://github.github.com/gh-aw/reference/threat-detection/)
- [GitHub Agentic Workflows security architecture](https://github.blog/ai-and-ml/generative-ai/under-the-hood-security-architecture-of-github-agentic-workflows/)
- [GitHub Actions security guidance](https://wellarchitected.github.com/library/application-security/recommendations/actions-security/)
- [OAuth 2.0 Security BCP / RFC 9700](https://www.ietf.org/rfc/rfc9700.html)
- [Stripe security docs](https://docs.stripe.com/security)
- [Stripe webhook docs](https://docs.stripe.com/webhooks)
- [OpenHands Secret Registry](https://docs.openhands.dev/sdk/guides/secrets)
- [OpenHands observability and tracing](https://docs.openhands.dev/sdk/guides/observability)
- [Anthropic MCP docs](https://code.claude.com/docs/en/mcp)

The common pattern across these sources is:

- trust nothing by default
- separate read, write, and approval responsibilities
- constrain inputs and outputs
- isolate execution
- log everything important
- validate sensitive operations outside the model

---

## 5. Security Principles

The final ZAOS security design must follow these principles:

- read-only by default
- write paths must be staged and vetted
- approvals must be explicit for sensitive actions
- secrets must not be directly exposed to the agent runtime
- untrusted text must not directly drive privileged behavior
- structured outputs must be used between sensitive nodes
- sandboxing should reduce blast radius
- third-party MCP should be treated as untrusted until governed
- validation must be external to the model
- human review should exist for high-impact flows

---

## 6. Security Architecture

The final ZAOS security model should be built as seven layers:

1. Trust Classification
2. Input Guardrails
3. Permission and Approval Layer
4. Secret Hygiene Layer
5. Execution Isolation Layer
6. Output Vetting Layer
7. Audit and Forensics Layer

These layers should cooperate but never collapse into one implicit notion of safety.

---

## 7. Layer 1 - Trust Classification

### 7.1 Role

Every task should be classified by sensitivity before execution.

### 7.2 Suggested Trust Levels

- low risk
- normal
- sensitive
- high impact
- security critical

### 7.3 What Should Increase Risk

- auth changes
- payment changes
- webhook changes
- secret handling
- deployment changes
- permission changes
- infrastructure changes
- external network access
- third-party MCP usage
- destructive operations

### 7.4 Product Function

Trust classification determines:

- which persona can execute
- which tools are allowed
- whether approvals are needed
- which validations are required
- whether human review is mandatory

---

## 8. Layer 2 - Input Guardrails

### 8.1 Role

This layer protects the system from untrusted input steering sensitive behavior.

### 8.2 Required Guardrails

- redact or isolate unsafe PII when needed
- detect jailbreak attempts
- block prompt injection patterns where possible
- prevent untrusted content from being inserted into developer-message-like privileged contexts
- only pass structured fields into sensitive logic

### 8.3 Structured Output Requirement

Where possible, ZAOS should use structured outputs between workflow nodes:

- enums
- fixed schemas
- validated JSON
- explicit required fields

This minimizes freeform channels that injection can exploit.

### 8.4 Product Function

Input guardrails reduce the chance that untrusted content directly controls tool calls or policy decisions.

---

## 9. Layer 3 - Permission and Approval Layer

### 9.1 Role

This layer decides whether an action may proceed.

### 9.2 Default Rule

Agentic work should run in read-only or plan mode until it is explicitly allowed to mutate.

### 9.3 Sensitive Actions Requiring Approval

- file writes
- deletions
- command execution with side effects
- MCP tool usage
- network access
- deployment operations
- auth or permission changes
- payment or billing changes
- secret retrieval or secret injection

### 9.4 Product Rule

No sensitive action should happen just because the model proposed it.

Approvals must exist as a system mechanism, not a stylistic recommendation.

---

## 10. Layer 4 - Secret Hygiene Layer

### 10.1 Role

Secrets must be kept out of the agent’s direct trust domain whenever possible.

### 10.2 Required Rules

- do not place raw secrets in prompts
- do not expose secrets directly to the model when a proxy or scoped credential can be used
- mask secrets in logs and traces
- rotate secrets regularly
- keep access scoped to the minimum necessary action

### 10.3 Secret Handling Model

The preferred model is:

- secret store or vault
- scoped proxy or secret registry
- masked runtime exposure
- limited lifetime access

### 10.4 Product Function

This reduces the risk of:

- secret exfiltration
- accidental logging
- accidental propagation through traces
- overbroad access in multi-agent workflows

---

## 11. Layer 5 - Execution Isolation Layer

### 11.1 Role

Execution must happen in a constrained environment.

### 11.2 Recommended Controls

- sandboxed filesystem access
- network egress control
- container or VM isolation where appropriate
- project-scoped write boundaries
- separate read and write jobs for sensitive flows

### 11.3 Read / Staging / Write Model

The strongest pattern is:

1. read-only exploration
2. staged generation of safe outputs
3. vetted write execution

### 11.4 Product Function

Isolation reduces blast radius and makes failures easier to reason about and audit.

---

## 12. Layer 6 - Output Vetting Layer

### 12.1 Role

Before a write or sensitive action becomes real, the output must be vetted.

### 12.2 Vetting Methods

- safe output filtering
- policy checks
- patch inspection
- secret scanning
- injection detection
- allowlist enforcement
- content sanitation

### 12.3 Why It Matters

The output of a model is not trustworthy just because it is well-formed.

It must be checked against the relevant policy and risk surface.

### 12.4 Product Function

Output vetting is the bridge between model suggestions and actual system mutation.

---

## 13. Layer 7 - Audit and Forensics Layer

### 13.1 Role

Security must be reconstructible after the fact.

### 13.2 What Must Be Logged

- approvals
- permission decisions
- write attempts
- blocked operations
- threat detection findings
- MCP usage
- network patterns
- critical file touches
- secret-related events
- deployment events

### 13.3 Why It Matters

Auditability supports:

- incident response
- forensic reconstruction
- policy validation
- compliance
- root cause analysis

### 13.4 Product Function

Logs and traces are not just telemetry.
They are the evidence layer for security decisions.

---

## 14. MCP Security

### 14.1 Principle

Third-party MCP servers must be treated as untrusted until explicitly governed.

### 14.2 Required Controls

- allowlist MCP servers by project and persona
- require approval on MCP use
- restrict tool scope per subagent
- treat unknown MCPs as a high-risk boundary
- record tool usage in traces and logs

### 14.3 Product Rule

MCP should increase capability without increasing blind trust.

---

## 15. Auth, Sessions, and Permissions

### 15.1 Why This Is Special

Auth and session flows are a classic high-risk zone because small mistakes can create major security problems.

### 15.2 Required Controls

- dedicated workflow path
- explicit approval
- structured validation
- no direct secret exposure
- clear audit trail
- non-regression coverage

### 15.3 Product Function

ZAOS should treat auth/session changes as sensitive by default.

---

## 16. Payments and Webhooks

### 16.1 Why This Is Special

Payments and webhooks require strict external verification and anti-replay behavior.

### 16.2 Required Controls

- raw body verification for webhook signatures
- signature validation before parsing
- anti-replay protection
- secret rotation
- idempotent handling
- dedicated tests for edge cases

### 16.3 Product Rule

Stripe and similar integrations should have their own security gate and validation bundle.

---

## 17. Deployment and CI Security

### 17.1 Why This Is Special

Deployments and CI are high-impact mutation surfaces.

### 17.2 Required Controls

- least privilege
- branch and environment restrictions
- pinned actions and dependencies
- approval before write or deploy steps
- no silent privilege expansion
- safe outputs or staged changes before mutation
- explicit logging for every trust boundary

### 17.3 Product Function

CI and deployment are not just execution surfaces.
They are security boundaries.

---

## 18. Prompt Injection Defense

### 18.1 Threat Model

Untrusted content may try to steer the agent into unsafe behavior.

### 18.2 Defenses

- isolate untrusted data
- avoid directly placing untrusted content into privileged channels
- use structured outputs
- keep approvals on
- block risky shell patterns
- inspect suspicious content
- prefer read-only analysis before mutation

### 18.3 Product Rule

No untrusted text should directly become a privileged command path.

---

## 19. Safe Output Model

### 19.1 Role

Safe outputs are the bridge between agent intention and permitted action.

### 19.2 Design Goals

- clearly define what the agent may request
- sanitize or reject unsafe content
- preserve human-visible artifacts
- restrict writes to approved scopes

### 19.3 Product Function

Safe outputs make writes auditable and policy-governed instead of freeform.

---

## 20. Validation Link

Security should not be separated from validation.

The final ZAOS security model must connect to the validation system so that:

- sensitive closures require evidence
- risky operations require traces
- blocked actions are visible
- audits can be replayed
- security gates cannot be satisfied by narrative alone

---

## 21. Anti-Patterns to Explicitly Reject

ZAOS should explicitly reject:

- giving the agent direct access to raw secrets
- allowing writes without staging or approval
- allowing third-party MCP by default without governance
- relying on prompt text as the only safety barrier
- parsing webhook bodies before verifying signatures
- letting the same agent both propose and certify a risky action
- deploying from unreviewed or unpinned automation
- hiding blocked operations from audit trails

---

## 22. Product Definition of Done for Security

The security layer of ZAOS can be considered complete when all of the following are true in practice:

- the default posture is read-only or plan-first
- sensitive actions require explicit approval
- secrets are not directly exposed to the agent runtime
- structured outputs are used for sensitive node-to-node communication
- sandboxing or isolation constrains blast radius
- audit logs capture the important trust boundaries
- auth, payments, webhooks, deploy, and MCP have dedicated security paths
- third-party MCP is treated as untrusted unless governed
- security-sensitive tasks have explicit validation and review
- validation and security are linked to evidence, not just prompts

---

## 23. Final Security Statement

The ultimate ZAOS security system should be:

**a layered secure SDLC for agents in which read-only by default, explicit approvals, secret isolation, sandboxed execution, structured outputs, threat detection, and auditable trust boundaries collectively prevent the agent from silently crossing sensitive security lines.**

---

## 24. Final Recommendation

If the final ZAOS security model had to be summarized in one synthesis line:

- OWASP for risk taxonomy and verification expectations
- NIST SSDF for secure SDLC integration
- OpenAI for approvals, structured outputs, trace-based safety
- Anthropic for read-only defaults, sandboxing, and MCP caution
- GitHub Agentic Workflows for safe outputs, threat detection, and stage-and-vet writes
- OpenHands for secret handling and controlled runtime patterns
- Stripe and OAuth guidance for real-world auth and payment hardening

That combination is the strongest candidate for the ultimate security architecture of ZAOS.
