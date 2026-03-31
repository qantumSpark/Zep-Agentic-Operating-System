//! Compile-time embedded agent and rule content.
//!
//! Uses `include_str!()` to bake reference files directly into the binary,
//! eliminating the need for a runtime `reference/` directory.

/// An embedded file: its deploy filename and content.
pub struct EmbeddedFile {
    pub filename: &'static str,
    pub content: &'static str,
}

// ── Agents ──────────────────────────────────────────────────────────────────

pub const AGENTS: &[EmbeddedFile] = &[
    EmbeddedFile {
        filename: "architect.md",
        content: include_str!("../../../reference/agents/architect.md"),
    },
    EmbeddedFile {
        filename: "coder.md",
        content: include_str!("../../../reference/agents/coder.md"),
    },
    EmbeddedFile {
        filename: "playtester.md",
        content: include_str!("../../../reference/agents/playtester.md"),
    },
    EmbeddedFile {
        filename: "researcher.md",
        content: include_str!("../../../reference/agents/researcher.md"),
    },
    EmbeddedFile {
        filename: "reviewer.md",
        content: include_str!("../../../reference/agents/reviewer.md"),
    },
    EmbeddedFile {
        filename: "tester.md",
        content: include_str!("../../../reference/agents/tester.md"),
    },
];

// ── Rules ───────────────────────────────────────────────────────────────────

pub const RULES: &[EmbeddedFile] = &[
    EmbeddedFile {
        filename: "01-always.mdc",
        content: include_str!("../../../reference/rules/01-always.mdc"),
    },
    EmbeddedFile {
        filename: "02-anti-hallucination.mdc",
        content: include_str!("../../../reference/rules/02-anti-hallucination.mdc"),
    },
    EmbeddedFile {
        filename: "03-code-review.mdc",
        content: include_str!("../../../reference/rules/03-code-review.mdc"),
    },
    EmbeddedFile {
        filename: "04-memory-hygiene.mdc",
        content: include_str!("../../../reference/rules/04-memory-hygiene.mdc"),
    },
];
