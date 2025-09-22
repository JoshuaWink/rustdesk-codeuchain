use crate::types::*;
use crate::contexts::*;
use codeuchain::{Context, Chain, LegacyLink};
use async_trait::async_trait;
use std::sync::Arc;
use crate::middleware::*;
use crate::server_links::*;

/// Server Orchestrator Link - Coordinates sequential execution of server operations
pub struct ServerOrchestratorLink {
    service_orchestrator: ServiceOrchestrationLink,
    connection_lifecycle: ConnectionLifecycleLink,
    media_capture: MediaCaptureLink,
    security_enforcement: SecurityEnforcementLink,
    resource_management: ResourceManagementLink,
}

impl ServerOrchestratorLink {
    pub fn new() -> Self {
        Self {
            service_orchestrator: ServiceOrchestrationLink::new(),
            connection_lifecycle: ConnectionLifecycleLink::new(),
            media_capture: MediaCaptureLink::new(),
            security_enforcement: SecurityEnforcementLink::new(),
            resource_management: ResourceManagementLink::new(),
        }
    }
}

#[async_trait]
impl LegacyLink for ServerOrchestratorLink {
    async fn call(&self, ctx: Context) -> LinkResult<Context> {
        let data = ctx.data().clone();

        println!("🔗 Starting server orchestration sequence...");

        // Step 1: Service Orchestration - Initialize server services
        println!("📋 Step 1: Orchestrating server services");
        let ctx1 = self.service_orchestrator.call(ctx).await?;
        println!("✅ Services initialized successfully");

        // Step 2: Connection Lifecycle - Establish and manage connection
        println!("📋 Step 2: Managing connection lifecycle");
        let ctx2 = self.connection_lifecycle.call(ctx1).await?;
        println!("✅ Connection lifecycle established");

        // Step 3: Media Capture - Set up media capture capabilities
        println!("📋 Step 3: Configuring media capture");
        let ctx3 = self.media_capture.call(ctx2).await?;
        println!("✅ Media capture configured");

        // Step 4: Security Enforcement - Apply security policies
        println!("📋 Step 4: Enforcing security policies");
        let ctx4 = self.security_enforcement.call(ctx3).await?;
        println!("✅ Security policies enforced");

        // Step 5: Resource Management - Monitor and allocate resources
        println!("📋 Step 5: Managing server resources");
        let ctx5 = self.resource_management.call(ctx4).await?;
        println!("✅ Resources allocated and monitored");

        println!("🎉 Server orchestration completed successfully!");
        Ok(ctx5)
    }
}

/// Server Chain Factory - Creates configured server processing chains
pub struct ServerChainFactory;

impl ServerChainFactory {
    pub fn new() -> Self {
        Self
    }

    pub fn create_server_chain(&self) -> Chain {
        let mut chain = Chain::new();

        // Add server orchestrator link
        chain.add_link("server_orchestrator".to_string(), Box::new(ServerOrchestratorLink::new()));

        // Add middleware stack
        chain.use_middleware(Box::new(LoggingMiddleware::new()));
        chain.use_middleware(Box::new(PerformanceMiddleware::new()));
        chain.use_middleware(Box::new(SecurityMiddleware::new()));
        chain.use_middleware(Box::new(RateLimitMiddleware::new(100))); // 100 requests per second

        chain
    }

    pub fn create_minimal_server_chain(&self) -> Chain {
        let mut chain = Chain::new();

        // Add individual server links for testing
        chain.add_link("service_orchestration".to_string(), Box::new(ServiceOrchestrationLink::new()));
        chain.add_link("connection_lifecycle".to_string(), Box::new(ConnectionLifecycleLink::new()));
        chain.add_link("media_capture".to_string(), Box::new(MediaCaptureLink::new()));
        chain.add_link("security_enforcement".to_string(), Box::new(SecurityEnforcementLink::new()));
        chain.add_link("resource_management".to_string(), Box::new(ResourceManagementLink::new()));

        chain.connect(
            "service_orchestration".to_string(),
            "connection_lifecycle".to_string(),
            Box::new(|ctx: &Context| ctx.get("services_initialized").unwrap_or(&serde_json::Value::Bool(false)).as_bool().unwrap_or(false))
        );

        chain.connect(
            "connection_lifecycle".to_string(),
            "media_capture".to_string(),
            Box::new(|ctx: &Context| ctx.get("connection_status").unwrap_or(&serde_json::Value::String("".to_string())).as_str().unwrap_or("") == "active")
        );

        chain.connect(
            "media_capture".to_string(),
            "security_enforcement".to_string(),
            Box::new(|ctx: &Context| ctx.get("media_capture_active").unwrap_or(&serde_json::Value::Bool(false)).as_bool().unwrap_or(false))
        );

        chain.connect(
            "security_enforcement".to_string(),
            "resource_management".to_string(),
            Box::new(|ctx: &Context| ctx.get("security_validated").unwrap_or(&serde_json::Value::Bool(false)).as_bool().unwrap_or(false))
        );

        // Add middleware
        chain.use_middleware(Box::new(LoggingMiddleware::new()));

        chain
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[tokio::test]
    async fn test_server_orchestrator() {
        let orchestrator = ServerOrchestratorLink::new();

        // Create test context with server configuration
        let mut initial_data = std::collections::HashMap::new();
        initial_data.insert("service_config".to_string(), json!({
            "enable_video": true,
            "enable_audio": true,
            "enable_clipboard": true,
            "enable_input": false
        }));
        initial_data.insert("client_id".to_string(), json!("test_client_123"));
        initial_data.insert("lifecycle_action".to_string(), json!("establish"));
        initial_data.insert("security_config".to_string(), json!({
            "require_auth": true,
            "enable_encryption": true,
            "max_sessions": 5
        }));
        initial_data.insert("resource_config".to_string(), json!({
            "max_cpu_usage": 75.0,
            "max_memory_mb": 512,
            "enable_monitoring": true
        }));

        let ctx = Context::new(initial_data);

        // Run orchestrator
        let result_ctx = orchestrator.call(ctx).await.unwrap();

        // Verify orchestration completed successfully
        assert!(result_ctx.get("services_initialized").unwrap().as_bool().unwrap());
        assert_eq!(result_ctx.get("connection_status").unwrap().as_str().unwrap(), "active");
        assert!(result_ctx.get("media_capture_active").unwrap().as_bool().unwrap());
        assert!(result_ctx.get("security_validated").unwrap().as_bool().unwrap());
        assert!(result_ctx.get("resources_allocated").unwrap().as_bool().unwrap());

        println!("✅ Server orchestrator test passed!");
    }

    #[tokio::test]
    async fn test_server_chain_factory() {
        let factory = ServerChainFactory::new();

        // Test full orchestrator chain
        let chain = factory.create_server_chain();

        // Create test context
        let mut initial_data = std::collections::HashMap::new();
        initial_data.insert("service_config".to_string(), json!({
            "enable_video": true,
            "enable_audio": true,
            "enable_clipboard": true,
            "enable_input": false
        }));
        initial_data.insert("client_id".to_string(), json!("test_client_456"));
        initial_data.insert("lifecycle_action".to_string(), json!("establish"));
        initial_data.insert("security_config".to_string(), json!({
            "require_auth": false,
            "enable_encryption": true,
            "max_sessions": 10
        }));
        initial_data.insert("resource_config".to_string(), json!({
            "max_cpu_usage": 80.0,
            "max_memory_mb": 1024,
            "enable_monitoring": true
        }));

        let ctx = Context::new(initial_data);

        // Run chain
        let result_ctx = chain.run(ctx).await.unwrap();

        // Verify chain execution
        assert!(result_ctx.get("services_initialized").unwrap().as_bool().unwrap());
        assert!(result_ctx.get("resources_allocated").unwrap().as_bool().unwrap());

        println!("✅ Server chain factory test passed!");
    }

    #[tokio::test]
    async fn test_minimal_server_chain() {
        let factory = ServerChainFactory::new();
        let chain = factory.create_minimal_server_chain();

        // Create minimal test context
        let mut initial_data = std::collections::HashMap::new();
        initial_data.insert("service_config".to_string(), json!({
            "enable_video": true,
            "enable_audio": true,
            "enable_clipboard": true,
            "enable_input": false
        }));

        let ctx = Context::new(initial_data);

        // Run chain - should execute service orchestration first
        let result_ctx = chain.run(ctx).await.unwrap();

        // Verify first link executed
        assert!(result_ctx.get("services_initialized").unwrap().as_bool().unwrap());

        println!("✅ Minimal server chain test passed!");
    }
}