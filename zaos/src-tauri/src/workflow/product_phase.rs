use serde::{Deserialize, Serialize};
use std::fmt;

use super::state::WorkflowState;

/// Product-level phase derived from the workflow state.
/// Maps internal pipeline phases to user-facing product phases.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProductPhase {
    None,
    Imagine,
    Shape,
    Design,
    Build,
    Verify,
    Release,
    Learn,
}

impl fmt::Display for ProductPhase {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let label = match self {
            ProductPhase::None => "None",
            ProductPhase::Imagine => "Imagine",
            ProductPhase::Shape => "Shape",
            ProductPhase::Design => "Design",
            ProductPhase::Build => "Build",
            ProductPhase::Verify => "Verify",
            ProductPhase::Release => "Release",
            ProductPhase::Learn => "Learn",
        };
        write!(f, "{}", label)
    }
}

impl Default for ProductPhase {
    fn default() -> Self {
        ProductPhase::None
    }
}

/// Ordered list of active product phases (excludes None).
pub const PRODUCT_PHASE_ORDER: [ProductPhase; 7] = [
    ProductPhase::Imagine,
    ProductPhase::Shape,
    ProductPhase::Design,
    ProductPhase::Build,
    ProductPhase::Verify,
    ProductPhase::Release,
    ProductPhase::Learn,
];

/// Derives the product-level phase from the current workflow state.
pub fn derive_product_phase(state: &WorkflowState) -> ProductPhase {
    match state.phase.as_str() {
        "idle" => {
            // Check if we just came back from closure (post-release retrospective)
            if let Some(last) = state.history.last() {
                if last.from_phase == "closure" && last.to_phase == "idle" {
                    return ProductPhase::Learn;
                }
            }
            // Epic defined but not yet started
            if !state.epic.is_empty() {
                return ProductPhase::Imagine;
            }
            ProductPhase::None
        }
        "comprehension" => ProductPhase::Imagine,
        "specification" => ProductPhase::Shape,
        "architecture" => ProductPhase::Design,
        "implementation" => ProductPhase::Build,
        "review" => ProductPhase::Verify,
        "test" => ProductPhase::Verify,
        "closure" => ProductPhase::Release,
        _ => ProductPhase::None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::state::PhaseTransition;

    fn make_transition(from: &str, to: &str) -> PhaseTransition {
        PhaseTransition {
            from_phase: from.to_string(),
            to_phase: to.to_string(),
            timestamp: "2026-01-01T00:00:00Z".to_string(),
            reason: None,
        }
    }

    #[test]
    fn test_idle_no_epic() {
        let state = WorkflowState::default();
        assert_eq!(derive_product_phase(&state), ProductPhase::None);
    }

    #[test]
    fn test_idle_with_epic() {
        let mut state = WorkflowState::default();
        state.epic = "MyEpic".to_string();
        assert_eq!(derive_product_phase(&state), ProductPhase::Imagine);
    }

    #[test]
    fn test_idle_post_closure_learn() {
        let mut state = WorkflowState::default();
        state.history.push(make_transition("closure", "idle"));
        assert_eq!(derive_product_phase(&state), ProductPhase::Learn);
    }

    #[test]
    fn test_idle_post_closure_with_epic_learn() {
        let mut state = WorkflowState::default();
        state.epic = "MyEpic".to_string();
        state.history.push(make_transition("closure", "idle"));
        assert_eq!(derive_product_phase(&state), ProductPhase::Learn);
    }

    #[test]
    fn test_comprehension() {
        let mut state = WorkflowState::default();
        state.phase = "comprehension".to_string();
        assert_eq!(derive_product_phase(&state), ProductPhase::Imagine);
    }

    #[test]
    fn test_specification() {
        let mut state = WorkflowState::default();
        state.phase = "specification".to_string();
        assert_eq!(derive_product_phase(&state), ProductPhase::Shape);
    }

    #[test]
    fn test_architecture() {
        let mut state = WorkflowState::default();
        state.phase = "architecture".to_string();
        assert_eq!(derive_product_phase(&state), ProductPhase::Design);
    }

    #[test]
    fn test_implementation() {
        let mut state = WorkflowState::default();
        state.phase = "implementation".to_string();
        assert_eq!(derive_product_phase(&state), ProductPhase::Build);
    }

    #[test]
    fn test_review() {
        let mut state = WorkflowState::default();
        state.phase = "review".to_string();
        assert_eq!(derive_product_phase(&state), ProductPhase::Verify);
    }

    #[test]
    fn test_test() {
        let mut state = WorkflowState::default();
        state.phase = "test".to_string();
        assert_eq!(derive_product_phase(&state), ProductPhase::Verify);
    }

    #[test]
    fn test_closure() {
        let mut state = WorkflowState::default();
        state.phase = "closure".to_string();
        assert_eq!(derive_product_phase(&state), ProductPhase::Release);
    }

    #[test]
    fn test_unknown_phase() {
        let mut state = WorkflowState::default();
        state.phase = "unknown".to_string();
        assert_eq!(derive_product_phase(&state), ProductPhase::None);
    }

    #[test]
    fn test_learn_requires_last_transition() {
        let mut state = WorkflowState::default();
        state.history.push(make_transition("idle", "comprehension"));
        state.history.push(make_transition("comprehension", "closure"));
        state.history.push(make_transition("closure", "idle"));
        assert_eq!(derive_product_phase(&state), ProductPhase::Learn);
    }

    #[test]
    fn test_idle_not_learn_if_last_not_closure() {
        let mut state = WorkflowState::default();
        state.history.push(make_transition("comprehension", "idle"));
        assert_eq!(derive_product_phase(&state), ProductPhase::None);
    }

    #[test]
    fn test_product_phase_order_length() {
        assert_eq!(PRODUCT_PHASE_ORDER.len(), 7);
    }

    #[test]
    fn test_product_phase_order_no_none() {
        for phase in &PRODUCT_PHASE_ORDER {
            assert_ne!(*phase, ProductPhase::None);
        }
    }
}
