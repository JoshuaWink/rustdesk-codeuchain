use codeuchain::core::Chain;
use crate::ui_links::*;
use crate::middleware::*;
use crate::types::*;
use codeuchain::{Context, LegacyLink};
use async_trait::async_trait;
use serde_json;

/// Type aliases for compatibility
pub type LinkResult<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

/// UI orchestrator link that coordinates UI operations in sequence
pub struct UIOrchestratorLink {
    event_routing: EventRoutingLink,
    session_management: SessionManagementLink,
    state_sync: StateSynchronizationLink,
    user_interaction: UserInteractionLink,
}

impl UIOrchestratorLink {
    pub fn new() -> Self {
        Self {
            event_routing: EventRoutingLink::new(),
            session_management: SessionManagementLink::new(),
            state_sync: StateSynchronizationLink::new(),
            user_interaction: UserInteractionLink::new(),
        }
    }

    /// Process UI operations in the correct sequence
    async fn process_ui_flow(&self, ctx: &Context) -> LinkResult<Context> {
        // Step 1: Route events first (mouse, keyboard, clipboard, file transfer)
        println!("🎯 UI Flow Step 1: Event routing");
        let ctx1 = self.event_routing.call(ctx.clone()).await?;

        // Step 2: Handle session management (login, connection state, permissions)
        println!("🔐 UI Flow Step 2: Session management");
        let ctx2 = self.session_management.call(ctx1).await?;

        // Step 3: Synchronize state (status, options, config)
        println!("🔄 UI Flow Step 3: State synchronization");
        let ctx3 = self.state_sync.call(ctx2).await?;

        // Step 4: Handle user interactions (get/set options, peer config, favorites)
        println!("👤 UI Flow Step 4: User interaction");
        let ctx4 = self.user_interaction.call(ctx3).await?;

        println!("✅ UI flow processing completed successfully");
        Ok(ctx4)
    }
}

#[async_trait]
impl LegacyLink for UIOrchestratorLink {
    async fn call(&self, ctx: Context) -> LinkResult<Context> {
        println!("🎭 Starting UI orchestration flow");

        // Process the UI flow
        match self.process_ui_flow(&ctx).await {
            Ok(result_ctx) => {
                println!("✅ UI orchestration completed successfully");
                Ok(result_ctx)
            }
            Err(e) => {
                println!("❌ UI orchestration failed: {}", e);
                Err(Box::new(CodeUChainError::ChainExecutionError(e.to_string())))
            }
        }
    }
}

/// Factory for creating UI processing chains with middleware
pub struct UIChainFactory;

impl UIChainFactory {
    pub fn new() -> Self {
        Self
    }

    /// Create a complete UI processing chain
    pub fn create_ui_chain(&self) -> Chain {
        let mut chain = Chain::new();

        // Add the main UI orchestrator
        chain.add_link("ui_orchestrator".to_string(), Box::new(UIOrchestratorLink::new()));

        // Add middleware to the chain
        chain.use_middleware(Box::new(LoggingMiddleware::new()));
        chain.use_middleware(Box::new(PerformanceMiddleware::new()));
        chain.use_middleware(Box::new(SecurityMiddleware::new()));
        chain.use_middleware(Box::new(RateLimitMiddleware::new(1000))); // 1000 requests per second

        chain
    }

    /// Create a lightweight UI chain for simple operations
    pub fn create_lightweight_ui_chain(&self) -> Chain {
        let mut chain = Chain::new();

        // Add individual UI links for specific operations
        chain.add_link("event_routing".to_string(), Box::new(EventRoutingLink::new()));
        chain.add_link("session_management".to_string(), Box::new(SessionManagementLink::new()));
        chain.add_link("state_sync".to_string(), Box::new(StateSynchronizationLink::new()));
        chain.add_link("user_interaction".to_string(), Box::new(UserInteractionLink::new()));

        // Add basic middleware only
        chain.use_middleware(Box::new(LoggingMiddleware::new()));

        // Connect links in sequence
        chain.connect(
            "event_routing".to_string(),
            "session_management".to_string(),
            Box::new(|ctx: &Context| ctx.get("event_routing_complete").unwrap_or(&serde_json::Value::Bool(false)).as_bool().unwrap_or(false)),
        );

        chain.connect(
            "session_management".to_string(),
            "state_sync".to_string(),
            Box::new(|ctx: &Context| ctx.get("session_active").unwrap_or(&serde_json::Value::Bool(false)).as_bool().unwrap_or(false)),
        );

        chain.connect(
            "state_sync".to_string(),
            "user_interaction".to_string(),
            Box::new(|ctx: &Context| ctx.get("ui_ready").unwrap_or(&serde_json::Value::Bool(false)).as_bool().unwrap_or(false)),
        );

        chain
    }

    /// Create a UI chain for testing purposes
    #[cfg(test)]
    pub fn create_test_chain(&self) -> Chain {
        let mut chain = Chain::new();

        // Add test orchestrator
        chain.add_link("test_ui_orchestrator".to_string(), Box::new(UIOrchestratorLink::new()));

        // Minimal middleware for testing
        chain.use_middleware(Box::new(LoggingMiddleware::new()));

        chain
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[tokio::test]
    async fn test_ui_orchestrator() {
        let orchestrator = UIOrchestratorLink::new();

        // Create context with UI events
        let mut initial_data = std::collections::HashMap::new();
        initial_data.insert("ui_events".to_string(), json!(["mouse_event", "keyboard_event"]));

        let ctx = Context::new(initial_data);

        let result = orchestrator.call(ctx).await.unwrap();

        // Verify the flow completed
        assert!(result.get("ui_fully_operational").unwrap().as_bool().unwrap());
        assert!(result.get("interactions_processed").unwrap().as_bool().unwrap());
    }

    #[tokio::test]
    async fn test_ui_chain_factory() {
        let factory = UIChainFactory::new();

        let chain = factory.create_ui_chain();

        // Test with a simple context
        let mut initial_data = std::collections::HashMap::new();
        initial_data.insert("ui_events".to_string(), json!(["test_event"]));

        let ctx = Context::new(initial_data);

        let result = chain.run(ctx).await.unwrap();

        // Verify processing occurred
        assert!(result.get("ui_fully_operational").unwrap().as_bool().unwrap());
    }

    #[tokio::test]
    async fn test_lightweight_ui_chain() {
        let factory = UIChainFactory::new();

        let chain = factory.create_lightweight_ui_chain();

        // Create test context with proper initialization
        let mut initial_data = std::collections::HashMap::new();
        initial_data.insert("ui_events".to_string(), json!(["test_event"]));
        initial_data.insert("event_routing_complete".to_string(), json!(true)); // Initialize for first link
        initial_data.insert("session_active".to_string(), json!(true)); // Initialize for second link
        initial_data.insert("ui_ready".to_string(), json!(true)); // Initialize for third link

        let ctx = Context::new(initial_data);

        let result = chain.run(ctx).await.unwrap();

        // Verify chain execution
        assert!(result.get("event_routing_complete").unwrap().as_bool().unwrap());
        assert!(result.get("session_active").unwrap().as_bool().unwrap());
        assert!(result.get("ui_ready").unwrap().as_bool().unwrap());
        assert!(result.get("ui_fully_operational").unwrap().as_bool().unwrap());
    }
}