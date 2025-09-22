use crate::types::*;
use crate::core_main_links::*;
use crate::core::{Chain, Context, Link, Middleware};
use async_trait::async_trait;
use serde_json;

/// Factory for creating core main processing chains
pub struct ApplicationChainFactory;

impl ApplicationChainFactory {
    pub fn new() -> Self {
        Self
    }

    /// Creates a complete application startup chain
    pub fn create_application_chain(&self) -> Chain {
        let mut chain = Chain::new();

        // Add core processing links
        chain.add_link("argument_processing".to_string(), Box::new(ArgumentProcessingLink::new()));
        chain.add_link("configuration".to_string(), Box::new(ConfigurationLink::new()));
        chain.add_link("service_initialization".to_string(), Box::new(ServiceInitializationLink::new()));
        chain.add_link("lifecycle_management".to_string(), Box::new(LifecycleManagementLink::new()));

        // Connect links in sequence with simple predicates that always pass
        // (CodeUChain may require connections even for sequential execution)
        chain.connect("argument_processing".to_string(), "configuration".to_string(), |_| true);
        chain.connect("configuration".to_string(), "service_initialization".to_string(), |_| true);
        chain.connect("service_initialization".to_string(), "lifecycle_management".to_string(), |_| true);

        chain
    }
}

/// Orchestrator link that coordinates the complete application startup flow
pub struct ApplicationOrchestratorLink {
    chain_factory: ApplicationChainFactory,
}

impl ApplicationOrchestratorLink {
    pub fn new() -> Self {
        Self {
            chain_factory: ApplicationChainFactory::new(),
        }
    }

    /// Processes the complete application startup flow
    pub async fn process_application_flow(&self, initial_context: Context) -> LinkResult<Context> {
        // For debugging, manually execute links instead of using Chain::run
        let arg_link = ArgumentProcessingLink::new();
        let config_link = ConfigurationLink::new();
        let service_link = ServiceInitializationLink::new();
        let lifecycle_link = LifecycleManagementLink::new();

        // Execute links in sequence
        let ctx1 = arg_link.call(initial_context).await?;
        let ctx2 = config_link.call(ctx1).await?;
        let ctx3 = service_link.call(ctx2).await?;
        let result_ctx = lifecycle_link.call(ctx3).await?;

        // Check if application should terminate early
        let should_terminate = result_ctx.data().get("should_terminate")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        if should_terminate {
            let termination_reason = result_ctx.data().get("termination_reason")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown");

            println!("Application terminating early: {}", termination_reason);
        } else {
            println!("Application startup completed successfully");
        }

        Ok(result_ctx)
    }
}

#[async_trait]
impl Link for ApplicationOrchestratorLink {
    async fn call(&self, ctx: Context) -> LinkResult<Context> {
        self.process_application_flow(ctx).await
    }
}