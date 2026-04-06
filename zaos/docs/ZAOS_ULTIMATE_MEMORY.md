# ZAOS - Ultimate Memory Vision

## 1. Purpose

This document defines the target memory architecture for the final version of ZAOS.

It exists to answer a central product question:

**What kind of memory system is required if ZAOS is to reliably compensate for the known weaknesses of LLMs in professional software delivery?**

The answer is not a single vector database, not a single Markdown folder, and not a single shared state layer.

The answer is a **governed memory architecture with multiple layers, clear ownership, explicit promotion rules, and strong integration into workflow, validation, security, and resumption.**

---

## 2. Why ZAOS Needs an Ultimate Memory

LLMs fail in predictable ways:

- they forget constraints over time
- they lose track of prior decisions
- they confuse active truth and historical residue
- they hallucinate when evidence is weak
- they drift away from product intent
- they retry without learning
- they fail to preserve diagnosis across sessions
- they turn long projects into fragmented context islands

If ZAOS is meant to be a professional delivery cockpit, memory cannot be treated as a helper feature.
It must be one of the core reliability systems of the product.

The ultimate memory of ZAOS must therefore solve for:

- continuity across sessions
- continuity across personas and agents
- continuity across workflow phases
- anti-drift between intent, architecture, implementation, and evidence
- controlled recall inside limited context windows
- truthful resumption
- professional auditability

---

## 3. Design Principles

The final memory design of ZAOS must follow these principles:

- Markdown canonical sources remain human-readable and diffable
- derived indexes are never the ultimate source of truth
- each memory layer has a clear responsibility
- each memory item has a clear owner
- the system must distinguish fact, decision, assumption, risk, and evidence
- recall must prefer routing over dumping
- memory retrieval must be provenance-aware and freshness-aware
- operational memory must capture reality without automatically becoming canonical truth
- reflection can suggest, but not silently promote
- live coordination state must not be confused with durable project memory

---

## 4. The Repositories and Ideas This Builds On

The ZAOS memory vision synthesizes the strongest ideas from several systems:

- [memsearch](https://github.com/zilliztech/memsearch)
- [memX](https://github.com/MehulG/memX)
- [ReMe](https://github.com/agentscope-ai/ReMe)
- [Mimir](https://github.com/orneryd/Mimir)
- [Letta memory blocks](https://docs.letta.com/guides/core-concepts/memory/memory-blocks)
- [MemGPT architecture](https://docs.letta.com/guides/agents/architectures/memgpt)
- [Mem0 memory types](https://docs.mem0.ai/core-concepts/memory-types)
- [Graphiti / Zep](https://www.getzep.com/product/open-source)
- [Hindsight](https://github.com/vectorize-io/hindsight)
- [Anthropic context management](https://claude.com/blog/context-management)

Their best contributions can be summarized as:

- memsearch: Markdown-first truth plus derived hybrid retrieval
- ReMe: memory compaction, inter-session persistence, and journal thinking
- memX: schema-validated live shared memory for multi-agent coordination
- Mimir: relationship-rich memory and task-aware structure
- Letta / MemGPT: layered memory and always-visible memory blocks
- Mem0: clean memory scopes
- Graphiti: temporal relationships and validity windows
- Hindsight: retain, recall, and reflect for durable improvement

---

## 5. High-Level Memory Model

The final ZAOS memory should be built as **six layers**:

1. Core Memory Blocks
2. Canonical Project Memory
3. Operational Memory
4. Live Shared Coordination Memory
5. Retrieval and Index Layer
6. Graph and Reflection Layer

These layers must cooperate, but they must not collapse into one.

---

## 6. Layer 1 - Core Memory Blocks

### 6.1 Role

Core Memory Blocks contain the minimum set of truths that should remain continuously visible to the runtime or injected with the highest priority.

This layer exists to prevent immediate drift.

### 6.2 Typical Contents

- active product vision summary
- active product contract summary
- current workflow phase and gate
- current epic and task
- active hard constraints
- current critical open risks
- active phase policies
- recommended persona path

### 6.3 Inspiration

- Letta memory blocks
- MEX-like routing discipline

### 6.4 Characteristics

- small
- stable
- curated
- always relevant
- updated deliberately

### 6.5 Product Function

This layer keeps the system anchored.
It is the anti-drift anchor at the top of the memory stack.

---

## 7. Layer 2 - Canonical Project Memory

### 7.1 Role

Canonical Project Memory is the official human-readable truth of the project.

### 7.2 Location

Primary location:

- `.memory/`

### 7.3 Typical Contents

- product vision
- product contract
- architecture
- domain rules
- implementation standards
- security expectations
- decision records
- workflow documents
- current epic details
- maintenance runbooks

### 7.4 Inspiration

- memsearch
- ReMe file-based memory
- the ZAOS canonical memory philosophy

### 7.5 Characteristics

- human-editable
- diffable
- stable
- promoted deliberately
- not auto-mutated freely by agents

### 7.6 Product Function

This layer is the reference truth the cockpit should trust when asking:

- what are we building
- what is the official decision
- what constraints are binding
- what architecture is currently valid

---

## 8. Layer 3 - Operational Memory

### 8.1 Role

Operational Memory captures what actually happened during execution.

It is where the system stores runtime reality without confusing it with canonical truth.

### 8.2 Location

Primary location:

- `.zaos/memory/`

### 8.3 Typical Contents

- event journals
- session snapshots
- compacted session summaries
- review outputs
- test outputs
- diagnostics
- threat-model results
- security findings
- bug investigations
- incident records
- tool outputs worth retaining
- validation receipts

### 8.4 Inspiration

- ReMe journals and memory persistence
- memsearch transcript and layered retrieval
- Hindsight retain
- current ZAOS session insights direction

### 8.5 Characteristics

- detailed
- append-heavy
- operationally truthful
- useful for replay and audit
- not automatically canonical

### 8.6 Product Function

This layer supports:

- session resumption
- auditability
- diagnostics
- evidence retrieval
- maintenance continuity

---

## 9. Layer 4 - Live Shared Coordination Memory

### 9.1 Role

Live Shared Coordination Memory exists for real-time structured agent coordination.

It is not durable project truth.

### 9.2 Typical Contents

- active shared state
- temporary goals
- current handoffs
- locks
- in-flight statuses
- intermediate outputs
- runtime coordination markers

### 9.3 Inspiration

- memX

### 9.4 Characteristics

- schema-validated
- access-controlled
- real-time
- pub/sub-friendly
- optionally TTL-based
- explicitly non-canonical

### 9.5 Product Function

This layer prevents:

- context mismatch between active agents
- duplicated work
- uncontrolled overlapping edits
- invisible execution divergence

---

## 10. Layer 5 - Retrieval and Index Layer

### 10.1 Role

This layer exists to make recall efficient.

It is a shadow layer, not the source of truth.

### 10.2 Typical Contents

- BM25 indexes
- vector embeddings
- reranking metadata
- chunk hashes
- document-to-section maps
- transcript links
- evidence pointers
- search caches

### 10.3 Inspiration

- memsearch hybrid retrieval
- ReMe dual file/vector thinking
- Mem0 memory access patterns

### 10.4 Characteristics

- derived
- rebuildable
- query-oriented
- optimized for retrieval
- never canonical by itself

### 10.5 Product Function

This layer allows ZAOS to:

- find the right memory fast
- keep context small
- avoid prompt bloat
- support targeted recall instead of memory dumping

---

## 11. Layer 6 - Graph and Reflection Layer

### 11.1 Role

This layer exists to reason over relationships, time, contradictions, and learning.

### 11.2 Typical Contents

- entities such as features, files, risks, APIs, bugs, personas, integrations, decisions
- relationships such as `depends_on`, `blocked_by`, `validated_by`, `contradicts`, `affects`, `fixed_by`
- temporal validity markers such as `valid_from`, `valid_to`, `superseded_at`
- reflections and learned heuristics
- repeat incident patterns
- recurring anti-patterns

### 11.3 Inspiration

- Mimir
- Graphiti
- Hindsight reflect

### 11.4 Characteristics

- derived but valuable
- relationship-rich
- temporal
- useful for multi-hop reasoning
- not canonical unless promoted through governance

### 11.5 Product Function

This layer helps ZAOS:

- detect contradictions
- detect stale assumptions
- connect decisions to code, risks, and evidence
- improve over time from incidents and outcomes

---

## 12. Ownership Model

The ultimate ZAOS memory only works if ownership is explicit.

### 12.1 Ownership by Layer

- Core Memory Blocks: owned by ZAOS orchestration and explicit workflow updates
- Canonical Project Memory: owned by human-approved project truth and validated workflow transitions
- Operational Memory: owned by ZAOS runtime and execution logging
- Live Shared Coordination Memory: owned by active runtime processes and agent coordination
- Retrieval and Index Layer: owned by the system as a rebuildable derivative
- Graph and Reflection Layer: owned by the system as analysis and suggestion, not as automatic truth

### 12.2 Ownership Rules

- agents may write operational memory
- agents must not freely rewrite canonical truth
- reflections may suggest promotions
- only governed workflow transitions or explicit approval may promote to canonical memory
- live coordination memory must be treated as ephemeral unless deliberately persisted elsewhere

---

## 13. Memory Item Types

ZAOS should not store undifferentiated text blobs.

The final memory system should explicitly model at least:

- `fact`
- `constraint`
- `decision`
- `assumption`
- `risk`
- `policy`
- `procedure`
- `task`
- `validation`
- `evidence`
- `incident`
- `bug`
- `lesson`
- `preference`
- `integration_note`
- `security_finding`
- `release_note`

Each item should support metadata such as:

- `source`
- `owner`
- `scope`
- `created_at`
- `updated_at`
- `valid_from`
- `valid_to`
- `confidence`
- `canonical_status`
- `linked_evidence`

---

## 14. Promotion Rules

Promotion is the mechanism that keeps the memory system clean.

### 14.1 Default Rule

New information should enter operational memory first.

### 14.2 Promotion Conditions

An item should be promoted into canonical memory only when one or more of the following is true:

- it has passed a workflow gate
- it has been explicitly confirmed by a human
- it is promoted by an approved policy
- it is required as an official project truth

### 14.3 Examples

- an implementation hypothesis stays operational
- a validated architectural decision can become canonical
- raw test output remains evidence
- a repeated maintenance lesson may become a canonical procedure after review

### 14.4 Why This Matters

Without promotion rules, canonical memory becomes noisy and untrustworthy.

---

## 15. Retrieval Strategy

The final ZAOS memory should not rely on naive top-k retrieval.

It should use a recall cascade.

### 15.1 Recommended Recall Cascade

1. always-visible core blocks
2. route to the correct canonical file first
3. run hybrid retrieval across canonical and operational memory
4. expand related graph nodes if needed
5. apply time and freshness filters
6. drill down into evidence when necessary
7. produce a compact, provenance-aware brief for the LLM

### 15.2 Why This Is Better

This approach:

- reduces token waste
- avoids loading irrelevant history
- preserves traceability
- gives the model the right memory rather than the most text

---

## 16. Memory Health

The final memory system must expose a real memory health model.

### 16.1 What Memory Health Should Detect

- drift between product contract and implementation
- drift between workflow state and canonical memory
- stale canonical documents
- unresolved contradictions
- missing evidence for supposedly closed validations
- poor-quality session snapshots
- tasks closed without memory closure
- unresolved security findings
- repeated corrective loops without captured learning

### 16.2 Memory Health Output

Memory health should not be only a score.

It should provide:

- status
- causes
- severity
- affected scope
- recommended action

### 16.3 Product Role

Memory health is one of the main anti-drift instruments of the ZAOS cockpit.

---

## 17. Integration with the ZAOS Workflow

### 17.1 Macro-Pipeline

At the macro level, memory must preserve:

- vision
- product contract
- architecture
- security framing
- decision history

### 17.2 Task Micro-Loop

At the task level, memory must support:

- targeted recall
- local exploration
- implementation guidance
- evidence capture
- correction loops with diagnosis
- closure records

### 17.3 Maintenance

For maintenance, memory must preserve:

- prior incidents
- root causes
- previous fixes
- regressions
- operational lessons

### 17.4 Security

For security-sensitive projects, memory must preserve:

- threat models
- findings
- mitigations
- validation evidence
- release constraints

---

## 18. What ZAOS Should Explicitly Take from Each Repo

### 18.1 From memsearch

- Markdown-first truth
- shadow index philosophy
- hybrid retrieval
- progressive retrieval
- file watcher synchronization

### 18.2 From ReMe

- inter-session memory persistence
- compaction of long history
- dual file and vector logic
- memory as refinement rather than storage only

### 18.3 From memX

- schema-validated shared state
- ACL and controlled access
- real-time synchronization
- pub/sub coordination

### 18.4 From Mimir

- relationship modeling
- graph-driven understanding
- rich connection between tasks, files, and concepts

### 18.5 From Letta / MemGPT

- layered memory hierarchy
- always-visible memory blocks

### 18.6 From Mem0

- clean memory scopes

### 18.7 From Graphiti

- temporal validity and stale-awareness

### 18.8 From Hindsight

- retain, recall, reflect
- learning from incidents and corrections

---

## 19. What ZAOS Should Explicitly Reject

ZAOS should reject:

- making vector search the source of truth
- allowing agents to silently rewrite canonical memory
- mixing live coordination memory with durable project memory
- overbuilding graph complexity before governance is strong
- dumping huge memory contexts instead of routing intelligently
- promoting reflections or summaries into truth automatically

---

## 20. The Final Memory Statement

The ultimate ZAOS memory should be:

**a governed, layered, temporal, navigable memory system in which Markdown canonical memory remains the human source of truth, operational memory preserves execution reality and evidence, live shared memory coordinates active agents, retrieval and graph layers support precise recall, and reflection transforms incidents and outcomes into learning without bypassing workflow governance.**

---

## 21. Final Recommendation

If the final ZAOS memory had to be summarized in one synthesis line:

- memsearch for philosophy
- ReMe for memory lifecycle
- memX for live coordination
- Mimir for relationships
- Letta for visible memory anchors
- Mem0 for scopes
- Graphiti for time
- Hindsight for learning

That combination is the strongest candidate for the ultimate memory architecture of ZAOS.
