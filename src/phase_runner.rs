//! Trait to run all test phases. Allows custom implementations for testing.
use crate::{error::Error, orchestrator::Orchestrator};

use async_trait::async_trait;

#[async_trait]
pub trait PhaseRunner: Send + Sync {
    async fn run_all(&self, orch: &Orchestrator) -> Result<(), Error>;
}

/// Production runner that delegates to the existing `crate::phases` module.
pub struct DefaultPhaseRunner {
    registry: crate::phase_registry::PhaseRegistry,
}

impl Default for DefaultPhaseRunner {
    fn default() -> Self {
        Self::new()
    }
}

impl DefaultPhaseRunner {
    /// Build a runner with the default phase registry.
    pub fn new() -> Self {
        let mut reg = crate::phase_registry::PhaseRegistry::new();
        // Register the core phases in the same order as before.
        reg.register("early_exit", crate::phases::run_early_exit);
        reg.register("header", crate::phases::run_header);
        reg.register("server_discovery", crate::phases::run_server_discovery);
        reg.register("ip_discovery", crate::phases::run_ip_discovery);
        reg.register("ping", crate::phases::run_ping);
        reg.register("download", crate::phases::run_download);
        reg.register("upload", crate::phases::run_upload);
        reg.register("result", crate::phases::run_result);
        Self { registry: reg }
    }

    // Convenience wrapper used by legacy tests
    pub async fn run_all(
        &self,
        orch: &crate::orchestrator::Orchestrator,
    ) -> Result<(), crate::error::Error> {
        crate::phase_registry::run_all_registered(orch, &self.registry).await
    }
}

#[async_trait]
impl PhaseRunner for DefaultPhaseRunner {
    async fn run_all(&self, orch: &Orchestrator) -> Result<(), Error> {
        crate::phase_registry::run_all_registered(orch, &self.registry).await
    }
}
