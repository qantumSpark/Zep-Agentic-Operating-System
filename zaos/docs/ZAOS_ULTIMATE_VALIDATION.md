# ZAOS - Ultimate Validation Vision

## 1. Purpose

This document defines the target validation, evaluation, and proof system for the final version of ZAOS.

It answers a core product question:

**How should ZAOS decide that work is truly correct, complete, and safe to advance, instead of merely sounding convincing?**

The answer is not:

- an agent saying "done"
- a single green test
- an LLM judge used as final authority
- a subjective feeling of progress

The answer is a **proof system external to the model**, built on:

- traces
- deterministic graders
- explicit gates
- replayable artifacts
- bounded correction loops
- anti-cheating checks
- human review where the risk demands it

---

## 2. Why ZAOS Needs an Ultimate Validation System

LLMs are not reliable judges of their own work.

They are prone to:

- declaring success too early
- hiding uncertainty
- modifying tests instead of fixing the product
- fabricating or overstating verification
- using weak shortcuts that create future regressions
- optimizing for passing signals instead of true correctness

In a professional delivery system, validation cannot be left to narrative.
It must be treated as a first-class subsystem.

ZAOS therefore needs a validation architecture that can answer:

- what was claimed
- what was actually executed
- what evidence exists
- what was graded
- why a gate passed or failed
- whether the agent may be cheating, drifting, or looping

---

## 3. Core Design Principles

The final ZAOS validation layer must follow these principles:

- no proof, no closure
- deterministic checks come before flexible judges
- the model must not be the sole judge of its own work
- every important validation should leave artifacts
- every gate should have explicit pass/fail criteria
- validation should grade traces, not just final answers
- closure should be based on observed reality, not agent confidence
- anti-cheat checks must be first-class
- retries must be bounded and diagnosis-driven
- human review should be required where risk or ambiguity justifies it

---

## 4. External Patterns This Builds On

The ZAOS validation vision synthesizes strong recurring patterns from:

- [OpenAI agent evals](https://platform.openai.com/docs/guides/agent-evals)
- [OpenAI evaluation best practices](https://platform.openai.com/docs/guides/evaluation-best-practices)
- [OpenAI trace grading](https://platform.openai.com/docs/guides/trace-grading)
- [OpenAI tracing](https://openai.github.io/openai-agents-python/tracing/)
- [Anthropic - Demystifying evals for AI agents](https://www.anthropic.com/engineering/demystifying-evals-for-ai-agents)
- [Claude Code hooks](https://docs.anthropic.com/en/docs/claude-code/hooks)
- [GitHub Agentic Workflows](https://github.github.com/gh-aw/)
- [GitHub threat detection](https://github.github.com/gh-aw/reference/threat-detection/)
- [aider lint and test workflow](https://aider.chat/docs/usage/lint-test.html)
- [OpenHands stuck detector](https://docs.openhands.dev/sdk/guides/agent-stuck-detector)
- [OpenHands evaluation harness](https://docs.openhands.dev/openhands/usage/developers/evaluation-harness)
- [OpenAI Evals repo](https://github.com/openai/evals)
- [LangSmith evaluation](https://www.langchain.com/langsmith/evaluation)
- [Langfuse docs](https://langfuse.com/docs)
- [SWE-bench evaluation](https://www.swebench.com/SWE-bench/guides/evaluation/)
- [SWE-Bench+](https://arxiv.org/abs/2410.06992)

These sources converge on a common model:

- traces first
- deterministic graders first
- explicit pass/fail gates
- replayable evidence
- bounded correction loops
- anti-cheating vigilance

---

## 5. Validation Philosophy

The strongest general rule for ZAOS is:

**The system decides closure, not the agent.**

The agent may propose that work is complete.
ZAOS must determine whether closure is actually justified.

This means validation is not one event.
It is a chain:

1. trace the work
2. collect artifacts
3. run deterministic checks
4. run flexible review where needed
5. apply gate criteria
6. approve or block
7. record the outcome

This is the backbone of the final ZAOS proof system.

---

## 6. Validation Architecture

The final ZAOS validation system should be built as six layers:

1. Trace Layer
2. Artifact Layer
3. Deterministic Grader Layer
4. Flexible Review Layer
5. Gate Layer
6. Closure and Replay Layer

These layers must cooperate, but they must not collapse into one implicit judgment.

---

## 7. Layer 1 - Trace Layer

### 7.1 Role

The trace layer captures what actually happened during execution.

### 7.2 What Must Be Traced

- task framing
- persona handoffs
- prompts and instructions at key transitions
- tool calls
- command executions
- file mutations
- approvals
- hook decisions
- subagent outputs
- test executions
- review events
- gate decisions

### 7.3 Why It Matters

Without traces:

- there is no reliable audit
- there is no reproducible validation
- there is no way to grade workflow behavior
- fake success becomes harder to detect

### 7.4 Product Function

In ZAOS, traces are the substrate for:

- artifact creation
- gate decisions
- anti-cheat checks
- replay
- diagnostics

---

## 8. Layer 2 - Artifact Layer

### 8.1 Role

Validation should rely on durable artifacts, not on chat summaries.

### 8.2 Acceptable Proof Artifacts

ZAOS should treat the following as proof-bearing artifacts:

- test output
- build output
- lint output
- runtime logs
- screenshots
- diffs
- review findings
- threat detection results
- structured reports such as `report.json`
- run identifiers
- CI results
- benchmark outputs
- trace segments

### 8.3 Artifact Principles

- artifacts should be replayable or inspectable
- artifacts should be attributable to a task or gate
- artifacts should be linked to closure decisions
- artifacts should outlive the immediate session

### 8.4 Product Function

Artifacts are what make closure auditable and resumption trustworthy.

---

## 9. Layer 3 - Deterministic Grader Layer

### 9.1 Role

Deterministic graders should be the first line of validation.

### 9.2 Examples

- lint
- tests
- build status
- typecheck
- schema validation
- required artifact presence
- path and policy checks
- file protection checks
- threat detection checks
- non-regression checks
- environment assertions

### 9.3 Why Deterministic First

Deterministic checks are:

- replayable
- debuggable
- less gameable than free-form judgment
- easier to use in gates

### 9.4 Product Rule

Whenever a validation question can be answered deterministically, ZAOS should prefer that route.

---

## 10. Layer 4 - Flexible Review Layer

### 10.1 Role

Some validation questions are not fully deterministic.

Examples:

- code quality nuance
- UX quality
- alignment with user intent
- architectural coherence
- writing quality
- suspicious but non-binary behavior

### 10.2 Allowed Review Modes

ZAOS may use:

- dedicated reviewer personas
- clean-room second-pass review
- LLM judges for narrow ambiguous questions
- human review for high-stakes cases

### 10.3 Rule of Use

Flexible review must be secondary to deterministic validation.

LLM judges should not be the single arbiter of closure.

### 10.4 Clean-Room Review

For sensitive or suspicious cases, ZAOS should support clean-room review:

- different reviewer
- different session
- minimal inheritance of shortcut assumptions

This is especially valuable for detecting fake success and hidden test manipulation.

---

## 11. Layer 5 - Gate Layer

### 11.1 Role

Gates are the formal transition checkpoints of the workflow.

### 11.2 Gate Inputs

Each gate should consume:

- required trace evidence
- required artifacts
- deterministic grader results
- review results if required
- policy conditions
- unresolved risk signals

### 11.3 Gate Outputs

A gate should produce:

- pass
- fail
- blocked pending evidence
- escalated pending human review

### 11.4 Gate Requirements

Every gate should have:

- explicit criteria
- explicit required evidence
- explicit blocking conditions
- explicit promotion effects

### 11.5 Product Rule

No workflow phase should advance based on implied confidence alone.

---

## 12. Layer 6 - Closure and Replay Layer

### 12.1 Role

Closure is not merely a status update.
It is a validated and replayable conclusion.

### 12.2 Closure Requirements

A task should close only when:

- required evidence exists
- deterministic checks required by the task have passed
- review requirements are satisfied
- no blocking contradiction remains
- the closure artifact has been written
- memory and workflow state have been updated

### 12.3 Replay Requirements

A closed task should be understandable later through:

- trace references
- proof artifacts
- closure summary
- gate outcome
- next-step recommendation if relevant

---

## 13. Validation Modes

The final ZAOS validation system should support different levels depending on risk.

### 13.1 Basic Validation

For low-risk tasks:

- diff present
- lint or build pass if applicable
- task artifact written
- no policy violation

### 13.2 Standard Validation

For normal implementation tasks:

- deterministic checks
- reviewer pass or structured review note
- trace coverage
- closure artifact

### 13.3 Sensitive Validation

For auth, payments, secrets, permissions, deployments, or high-impact edits:

- stronger gate requirements
- dedicated reviewer or specialist path
- threat detection
- stronger artifact requirements
- likely human review

---

## 14. Anti-Cheating Model

### 14.1 Why Anti-Cheating Must Be Explicit

Agentic coding systems commonly fail through "fake done" patterns such as:

- modifying the test instead of the product
- weakening assertions to make checks pass
- introducing mock data that hides broken logic
- declaring success without execution
- using demos that do not reflect actual behavior

### 14.2 Anti-Cheat Checks ZAOS Should Support

- detect suspicious test edits
- detect test weakening patterns
- compare changed tests against changed implementation
- require runtime execution evidence
- require real output capture for claims
- use second-pass review on suspicious closures
- inspect diffs for outcome-skewing shortcuts
- link claims to concrete artifacts

### 14.3 Product Rule

ZAOS should assume that "green" is not automatically trustworthy if the path to green is suspicious.

---

## 15. Judges, Graders, and Human Review

### 15.1 Priority Order

The final validation stack should prefer:

1. deterministic graders
2. structured trace graders
3. reviewer personas
4. LLM judges for ambiguous cases
5. human review for high-risk or unresolved cases

### 15.2 Why This Order

This order minimizes:

- subjective drift
- self-certification risk
- prompt sensitivity
- hidden shortcuts

### 15.3 Human Review Triggers

Human review should be strongly considered for:

- security-sensitive changes
- repeated failure loops
- unresolved disagreement between graders and reviewers
- high-impact deployments
- suspicious evidence patterns

---

## 16. Hook Integration

The final ZAOS validation system should integrate tightly with hooks.

### 16.1 Useful Validation Hook Points

- `PreToolUse`
- `PostToolUse`
- `OnWrite / OnEdit`
- `OnSubagentStop`
- `AsyncTestHook`
- `OnTaskClosure`
- `OnGateValidation`
- `Stop`

### 16.2 Hook Purposes

Hooks should be able to:

- block unsafe actions
- trigger tests
- collect proof
- detect suspicious changes
- refuse closure when evidence is missing
- write validation artifacts

### 16.3 Why Hooks Matter

Hooks make validation enforceable instead of aspirational.

---

## 17. Stuck Detection and Bounded Correction

### 17.1 Why It Matters

Validation is not complete if the system allows infinite correction loops.

### 17.2 What Must Be Detected

- repeated test failures of the same type
- repeated edits without progress
- repeated retries without diagnosis
- abnormal token burn without evidence improvement
- repeated closure attempts without sufficient proof

### 17.3 Circuit Breaker Outcomes

When these patterns appear, ZAOS should support:

- forced diagnosis
- return to planning
- persona switch
- escalation to reviewer
- escalation to human
- task pause

### 17.4 Product Rule

A correction loop without diagnosis is an anti-pattern and should not be treated as normal progress.

---

## 18. Validation Artifacts by Task Type

Different task classes should require different proof bundles.

### 18.1 Implementation Task

Typical proof set:

- diff
- lint and/or typecheck output
- tests output
- review result
- closure summary

### 18.2 Bugfix Task

Typical proof set:

- reproduction note
- fix diff
- regression proof
- review result if warranted
- closure summary

### 18.3 Refactor Task

Typical proof set:

- scope note
- non-regression checks
- build or test output
- risk note if architecture affected

### 18.4 Security-Sensitive Task

Typical proof set:

- code diff
- security review artifact
- threat-related checks
- tests
- approval or elevated gate result

---

## 19. What ZAOS Should Explicitly Take from Existing Systems

### 19.1 From OpenAI

- trace grading
- agent evals
- evaluation best practices
- structured safety thinking

### 19.2 From Anthropic

- hook-driven runtime enforcement
- multi-layer eval thinking

### 19.3 From GitHub Agentic Workflows

- safe outputs
- threat detection before application
- layered operational safety

### 19.4 From aider

- automatic lint/test loops after mutation

### 19.5 From OpenHands

- stuck detection
- iterative refinement with bounded loops

### 19.6 From SWE-bench and Related Work

- replayable evaluation harnesses
- caution about weak tests and solution leakage

---

## 20. Anti-Patterns to Explicitly Reject

ZAOS should explicitly reject:

- letting the same agent be the sole implementer and judge
- closing tasks on narrative confidence
- using LLM judges as the only final validation source
- accepting green tests without inspecting whether the path to green was legitimate
- relying on one final score without artifacts, logs, replay, or trace analysis
- allowing unlimited retries without escalation
- treating closure as a status flag rather than a proof-backed decision

---

## 21. Product Definition of Done for Validation

The validation layer of ZAOS can be considered complete when all of the following are true in practice:

- every important task produces traceable execution records
- every important closure is backed by proof artifacts
- deterministic checks are used wherever possible
- LLM-based review is secondary, not primary
- gate transitions are explicit and evidence-based
- anti-cheat signals are part of normal validation
- repeated loops trigger escalation instead of infinite retry
- sensitive changes require stronger proof and review paths
- replay and audit are possible after the fact

---

## 22. Final Validation Statement

The ultimate ZAOS validation system should be:

**a proof-driven validation architecture in which traces, artifacts, deterministic graders, explicit gates, anti-cheating checks, and bounded correction loops jointly determine whether work may close or progress, while LLM judges and human reviewers act as controlled secondary layers rather than as the primary source of truth.**

---

## 23. Final Recommendation

If the final ZAOS validation model had to be summarized in one synthesis line:

- OpenAI for evals, trace grading, and validation methodology
- Anthropic for hooks and workflow enforcement
- GitHub Agentic Workflows for operational safety and threat detection
- aider for practical post-edit test discipline
- OpenHands for stuck detection and bounded iteration
- SWE-bench thinking for replayable evaluation and anti-leakage vigilance

That combination is the strongest candidate for the ultimate validation architecture of ZAOS.
