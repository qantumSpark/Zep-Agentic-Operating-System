# ZAOS - Ultimate Artifacts Vision

## 1. Purpose

This document defines the target artifact, evidence, provenance, and replay model for the final version of ZAOS.

It answers a central product question:

**What should ZAOS keep as durable artifacts so that work can be validated, resumed, audited, and improved without relying on fragile summaries or raw transcript archaeology?**

The answer is not:

- one large narrative summary
- raw logs with no structure
- unlinked files stored ad hoc
- artifacts with no provenance or replay value

The answer is a **structured artifact system** built on:

- task-linked artifacts
- evidence packs
- session cards
- trace bundles
- provenance records
- replayable and comparable outputs
- retention and anti-noise rules

---

## 2. Why ZAOS Needs an Ultimate Artifact System

Agentic software work produces many outputs, but most are not useful if they cannot be linked, trusted, replayed, or interpreted later.

Without a strong artifact model, teams lose:

- proof of what happened
- proof of why a gate passed
- the ability to resume without rereading everything
- the ability to audit or replay a run
- the ability to compare outcomes over time
- the ability to transform prior work into evaluation datasets

For ZAOS to be a real professional cockpit, artifacts must become first-class objects.

---

## 3. Core Design Principles

The final ZAOS artifact system must follow these principles:

- the summary is an index, not the truth
- evidence is more important than narrative
- every artifact should be linked to session, task, and gate context
- useful artifacts should be replayable or inspectable
- provenance should be explicit
- artifacts should be small enough to remain usable
- retention should be intentional
- artifacts should support validation, resumption, and forensics
- outputs should be structured when crossing trust boundaries
- noise should be actively controlled

---

## 4. External Patterns This Builds On

The ZAOS artifact vision synthesizes strong recurring patterns from:

- [GitHub Actions workflow artifacts](https://docs.github.com/en/actions/concepts/workflows-and-actions/workflow-artifacts)
- [upload-artifact v4](https://github.com/actions/upload-artifact)
- [Artifact retention docs](https://docs.github.com/en/organizations/managing-organization-settings/configuring-the-retention-period-for-github-actions-artifacts-and-logs-in-your-organization)
- [Artifact attestations](https://docs.github.com/actions/security-guides/using-artifact-attestations-to-establish-provenance-for-builds)
- [GitHub Agentic Workflows - Safe Outputs](https://github.github.com/gh-aw/reference/safe-outputs/)
- [GitHub Agentic Workflows - Compilation Process](https://github.github.com/gh-aw/reference/compilation-process/)
- [OpenAI Safety in building agents](https://platform.openai.com/docs/guides/agent-builder-safety)
- [OpenAI trace grading](https://platform.openai.com/docs/guides/trace-grading)
- [OpenAI agent evals](https://platform.openai.com/docs/guides/agent-evals)
- [OpenAI structured outputs](https://platform.openai.com/docs/api-reference/chat/create-chat-completion)
- [Anthropic Claude Code hooks](https://docs.anthropic.com/en/docs/claude-code/hooks)
- [Anthropic Claude Code memory](https://docs.anthropic.com/en/docs/claude-code/memory)
- [LangSmith observability concepts](https://docs.langchain.com/langsmith/observability-concepts)
- [LangSmith manage trace](https://docs.langchain.com/langsmith/manage-trace)
- [LangSmith datasets](https://docs.langchain.com/langsmith/manage-datasets)
- [LangSmith observability studio](https://docs.langchain.com/langsmith/observability-studio)
- [Langfuse docs](https://langfuse.com/docs)
- [Langfuse demo](https://langfuse.com/docs/demo)
- [OpenHands persistence](https://docs.openhands.dev/sdk/guides/convo-persistence)
- [OpenHands observability](https://docs.openhands.dev/sdk/guides/observability)
- [OpenHands headless JSON](https://docs.openhands.dev/openhands/usage/how-to/headless-mode)
- [OpenHands session resume](https://docs.openhands.dev/openhands/usage/cli/command-reference)
- [SWE-bench](https://github.com/SWE-bench/SWE-bench)
- [SWE-bench evaluation guide](https://www.swebench.com/SWE-bench/guides/evaluation/)
- [OpenAI SWE-bench Verified](https://openai.com/index/introducing-swe-bench-verified/)

These sources converge on a common model:

- structured outputs
- replayable traces
- linked evidence
- run artifacts
- provenance and retention

---

## 5. Artifact Philosophy

The strongest general rule for ZAOS is:

**Artifacts are the durable truth layer of execution.**

The summary helps find them.
The trace explains them.
The evidence proves them.
The provenance validates where they came from.

This is the backbone of the final ZAOS artifact system.

---

## 6. The Six Standard Artifact Objects

The final ZAOS artifact architecture should be built around six standard objects:

1. Task Card
2. Session Snapshot
3. Handoff Artifact
4. Evidence Pack
5. Trace Bundle
6. Provenance Record

Dataset exports should exist as a seventh derivative class of artifact for evaluation and regression learning.

---

## 7. Artifact 1 - Task Card

### 7.1 Role

The Task Card is the compact structural artifact for one unit of work.

### 7.2 What It Should Capture

- `task_id`
- phase
- owner persona
- objective
- current gate state
- current status
- next step

### 7.3 Product Function

The Task Card is the main anchor for linking:

- evidence
- trace
- review
- closure

### 7.4 Product Rule

Every important task should have a stable Task Card.

---

## 8. Artifact 2 - Session Snapshot

### 8.1 Role

The Session Snapshot is the compact index for resumption.

### 8.2 What It Should Capture

- `session_id`
- project
- phase
- epic
- task
- decisions
- risks
- blockers
- next step
- next persona
- gate state
- evidence links
- trace links

### 8.3 Product Function

The Session Snapshot helps the user re-enter quickly.

### 8.4 Product Rule

The snapshot should point to artifacts rather than duplicating all detail.

---

## 9. Artifact 3 - Handoff Artifact

### 9.1 Role

The Handoff Artifact transfers continuity between actors or sessions.

### 9.2 What It Should Capture

- current objective
- most important decisions
- active blocker
- next action
- owner persona
- evidence links
- checkpoint or trace references

### 9.3 Product Function

This artifact reduces restart cost and protects continuity across interruptions.

---

## 10. Artifact 4 - Evidence Pack

### 10.1 Role

The Evidence Pack is the primary proof artifact.

### 10.2 Typical Contents

- test output
- build output
- lint output
- diffs
- screenshots
- logs
- review notes
- approval artifacts
- threat detection results
- deployment receipts
- incident notes

### 10.3 Product Function

The Evidence Pack exists to answer:

- what proves this task or gate
- what was actually executed
- what can be inspected later

### 10.4 Product Rule

If a task or gate is important, it should be closable only with a linked Evidence Pack.

---

## 11. Artifact 5 - Trace Bundle

### 11.1 Role

The Trace Bundle captures observable execution.

### 11.2 What It Should Capture

- execution timeline
- tool usage
- agent and subagent activity
- approvals
- hook outcomes
- gate transitions
- relevant logs

### 11.3 Product Function

This artifact supports:

- forensics
- audit
- validation
- replay
- dataset extraction

### 11.4 Product Rule

The Trace Bundle should be linked, not hidden behind raw log storage.

---

## 12. Artifact 6 - Provenance Record

### 12.1 Role

The Provenance Record captures origin, integrity, and lineage.

### 12.2 What It Should Capture

- `artifact_id`
- `artifact_digest`
- `project_id`
- `session_id`
- `task_id`
- `gate_id`
- `trace_id`
- `run_id`
- `checkpoint_id`
- producing persona or runtime
- approver where relevant
- creation time
- source workflow or pipeline

### 12.3 Product Function

This artifact allows ZAOS to answer:

- where did this come from
- who produced it
- which run produced it
- can integrity be checked

### 12.4 Product Rule

Important artifacts should not exist without provenance.

---

## 13. Derived Artifact Class - Dataset Export

### 13.1 Role

Dataset exports transform prior traces and evidence into evaluation material.

### 13.2 What They Should Support

- regression suites
- trace-based evals
- benchmark examples
- task outcome comparison
- future quality improvement

### 13.3 Product Function

This artifact class closes the loop between:

- production work
- evidence
- evaluation
- system improvement

---

## 14. Artifact Identity and Linking

### 14.1 Required Stable Identifiers

The artifact system should consistently use:

- `project_id`
- `session_id`
- `task_id`
- `gate_id`
- `trace_id`
- `run_id`
- `checkpoint_id`
- `artifact_id`
- `artifact_digest`

### 14.2 Why This Matters

Without identifiers, artifacts become:

- hard to trust
- hard to query
- hard to replay
- hard to compare

### 14.3 Product Rule

No important artifact should be orphaned from its task, session, or gate context.

---

## 15. Artifact Hierarchy

### 15.1 Recommended Hierarchy

The most useful hierarchy for ZAOS is:

- session
- trace
- run
- task
- artifact
- replay

### 15.2 Why This Hierarchy Matters

It allows:

- task-scoped validation
- session-scoped resumption
- trace-scoped forensics
- run-scoped comparison

### 15.3 Product Rule

The cockpit and workflow should both be able to navigate this hierarchy directly.

---

## 16. Structured Outputs and Boundary Artifacts

### 16.1 Role

Whenever information crosses trust boundaries, it should prefer structured form.

### 16.2 Why This Matters

Structured outputs reduce:

- ambiguity
- propagation of prompt injection
- artifact interpretation mismatch

### 16.3 Typical Uses

- inter-node outputs
- gate artifacts
- task closure records
- dataset exports
- provenance records

### 16.4 Product Rule

Free-form prose can complement an artifact, but should not replace structured machine-readable form where validation or promotion depends on it.

---

## 17. Replay, Comparison, and Promotion

### 17.1 Replay

Artifacts should be replayable where relevant:

- trace replay
- run replay
- session replay

### 17.2 Comparison

ZAOS should support comparing:

- runs
- traces
- evidence sets
- outcomes over time

### 17.3 Promotion

Some artifacts may later be promoted into:

- canonical memory
- evaluation datasets
- maintenance procedures
- lessons learned

### 17.4 Product Rule

Promotion should be explicit and governed.

---

## 18. Retention and Noise Control

### 18.1 Role

Artifacts only remain useful if noise is controlled.

### 18.2 Common Noise Problems

- giant transcript blobs
- duplicate summaries
- unfiltered logs
- unlinked outputs
- low-value artifacts retained forever

### 18.3 Retention Principles

- high-value artifacts get longer retention
- low-value raw outputs may expire
- summaries should remain compact
- trace storage may require TTL policies

### 18.4 Product Rule

ZAOS should prefer fewer, linked, meaningful artifacts over unlimited storage of low-signal material.

---

## 19. Artifacts for Validation

### 19.1 Validation Role

Artifacts are the backbone of proof in ZAOS.

### 19.2 Validation-Relevant Artifacts

- evidence pack
- trace bundle
- provenance record
- gate artifact
- review note

### 19.3 Product Rule

Validation should consume artifacts, not rely on human memory of what happened.

---

## 20. Artifacts for Resumption

### 20.1 Resumption Role

Artifacts are also the backbone of continuity.

### 20.2 Resumption-Relevant Artifacts

- session snapshot
- handoff artifact
- checkpoint references
- evidence links
- decision notes

### 20.3 Product Rule

The summary should route back to the proof-bearing artifacts.

---

## 21. Artifacts for Security and Operations

### 21.1 Security Role

Sensitive work requires stronger artifact discipline.

### 21.2 Sensitive Artifact Examples

- approval records
- threat detection outputs
- webhook verification evidence
- deployment receipts
- incident documents
- security review notes

### 21.3 Product Rule

Sensitive paths should leave stronger and more explicit artifacts than ordinary tasks.

---

## 22. What ZAOS Should Explicitly Take from Existing Systems

### 22.1 From GitHub Actions and GitHub Agentic Workflows

- artifact immutability thinking
- retention controls
- digests
- attestations
- safe outputs and staged writes

### 22.2 From OpenAI

- structured outputs
- trace-based evaluation
- artifact-friendly trace grading patterns

### 22.3 From Anthropic

- hook-triggered artifact capture
- session-aware execution lifecycle

### 22.4 From LangSmith and Langfuse

- traces and sessions as first-class linked objects
- dataset exports from traces
- replay and comparison value

### 22.5 From OpenHands

- persistence-backed session artifacts
- headless JSON outputs
- replay and observability integration

### 22.6 From SWE-bench Thinking

- reproducible harnesses
- strong run artifacts
- validation through execution, not only narrative

---

## 23. Anti-Patterns to Explicitly Reject

ZAOS should explicitly reject:

- one large narrative summary as the only artifact
- artifacts with no identifiers
- artifacts with no provenance or digest
- transcripts without indexing
- duplicate state copied into many inconsistent files
- evidence that cannot be tied back to task, session, or gate
- storing everything forever with no retention logic
- hiding proof artifacts behind raw logs

---

## 24. Product Definition of Done for Artifacts

The artifact layer of ZAOS can be considered complete when all of the following are true in practice:

- each important task has stable artifact linkage
- each important session has resumable summary artifacts
- gates are linked to explicit proof artifacts
- traces are replayable and connected to evidence
- provenance exists for important outputs
- artifacts support validation, resumption, and forensics
- retention is intentional rather than uncontrolled
- the cockpit can surface artifacts as first-class objects

---

## 25. Final Artifact Statement

The ultimate ZAOS artifact system should be:

**a linked, structured, replayable artifact architecture in which task cards, session snapshots, handoff artifacts, evidence packs, trace bundles, and provenance records work together so that every important piece of work can be validated, resumed, audited, compared, and promoted without depending on fragile summaries or unstructured logs.**

---

## 26. Final Recommendation

If the final ZAOS artifact model had to be summarized in one synthesis line:

- GitHub for artifact immutability, retention, provenance, and staged outputs
- OpenAI for structured outputs and trace-linked evaluation
- Anthropic for session lifecycle hooks
- LangSmith and Langfuse for trace, session, dataset, and replay thinking
- OpenHands for persistence and replayable execution artifacts
- SWE-bench-style harnesses for reproducible run outputs

That combination is the strongest candidate for the ultimate artifact architecture of ZAOS.
