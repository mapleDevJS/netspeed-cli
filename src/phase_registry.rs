//! Registry for phase functions allowing open/closed extension.
use crate::orchestrator::Orchestrator;
use crate::phases::{PhaseContext, PhaseFn, PhaseOutcome};
use std::collections::HashMap;
// Arc not needed – phases are stored in a Vec

/// Holds the ordered list of phases.
#[derive(Default)]
pub struct PhaseRegistry {
    phases: Vec<PhaseFn>,
    // optional map for named lookup if needed later
    #[allow(dead_code)]
    map: HashMap<String, usize>,
}

impl PhaseRegistry {
    /// Create a new empty registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a phase function (pushes to end of execution order).
    pub fn register(&mut self, name: impl Into<String>, phase: PhaseFn) {
        let idx = self.phases.len();
        self.map.insert(name.into(), idx);
        self.phases.push(phase);
    }

    /// Iterate over registered phases.
    pub fn iter(&self) -> impl Iterator<Item = &PhaseFn> {
        self.phases.iter()
    }
}

/// Execute all registered phases using the provided orchestrator.
pub async fn run_all_registered(
    orch: &Orchestrator,
    registry: &PhaseRegistry,
) -> Result<(), crate::error::Error> {
    let mut ctx = PhaseContext::new(orch.services_arc());
    for phase in registry.iter() {
        match phase(orch, &mut ctx).await {
            PhaseOutcome::PhaseCompleted => {}
            PhaseOutcome::PhaseEarlyExit => break,
            PhaseOutcome::PhaseError(e) => return Err(e),
        }
    }
    Ok(())
}
