use codeuchain::core::Chain;
use crate::ipc_links::*;
use crate::middleware::*;
use std::sync::Arc;

/// IPC Chain Factory - creates processing chains for different IPC workflows
pub struct IPCChainFactory;

impl IPCChainFactory {
    /// Create a config management chain
    pub fn create_config_chain() -> Chain {
        let mut chain = Chain::new();

        // Add config processing links
        chain.add_link("config_validator".to_string(), Box::new(ConfigValidatorLink::new()));
        chain.add_link("config_processor".to_string(), Box::new(ConfigProcessorLink::new()));

        // Connect links in sequence
        chain.connect("config_validator".to_string(), "config_processor".to_string(), |ctx| {
            // Always proceed to processor after validation
            true
        });

        // Add middleware
        chain.use_middleware(Box::new(LoggingMiddleware::new()));
        chain.use_middleware(Box::new(SecurityMiddleware::new()));
        chain.use_middleware(Box::new(RateLimitMiddleware::new(50))); // 50 config ops per window

        chain
    }

    /// Create a message processing chain
    pub fn create_message_chain() -> Chain {
        let mut chain = Chain::new();

        // Add message processing links
        chain.add_link("message_validator".to_string(), Box::new(MessageValidatorLink::new()));
        chain.add_link("message_processor".to_string(), Box::new(MessageProcessorLink::new()));

        // Connect links in sequence
        chain.connect("message_validator".to_string(), "message_processor".to_string(), |ctx| {
            // Always proceed to processor after validation
            true
        });

        // Add middleware
        chain.use_middleware(Box::new(LoggingMiddleware::new()));
        chain.use_middleware(Box::new(SecurityMiddleware::new()));
        chain.use_middleware(Box::new(RateLimitMiddleware::new(100))); // 100 messages per window

        chain
    }

    /// Create a system info chain
    pub fn create_system_chain() -> Chain {
        let mut chain = Chain::new();

        // Add system info link
        chain.add_link("system_info".to_string(), Box::new(SystemInfoLink::new()));

        // Add middleware
        chain.use_middleware(Box::new(LoggingMiddleware::new()));
        chain.use_middleware(Box::new(PerformanceMiddleware::new()));

        chain
    }

    /// Create a comprehensive IPC chain that handles all message types
    pub fn create_unified_ipc_chain() -> Chain {
        let mut chain = Chain::new();

        // Create individual processing links
        let config_validator = Box::new(ConfigValidatorLink::new());
        let config_processor = Box::new(ConfigProcessorLink::new());
        let message_validator = Box::new(MessageValidatorLink::new());
        let message_processor = Box::new(MessageProcessorLink::new());
        let system_info = Box::new(SystemInfoLink::new());

        // Create router link that routes based on IPC action
        let router_link = Box::new(IPCRouterLink::new()
            .with_config_validator(config_validator)
            .with_config_processor(config_processor)
            .with_message_validator(message_validator)
            .with_message_processor(message_processor)
            .with_system_info(system_info));

        // Add router as the main processing link
        chain.add_link("ipc_router".to_string(), router_link);

        // Add middleware stack
        chain.use_middleware(Box::new(LoggingMiddleware::new().with_level("info")));
        chain.use_middleware(Box::new(SecurityMiddleware::new()));
        chain.use_middleware(Box::new(RateLimitMiddleware::new(100)));

        chain
    }
}

/// Pre-configured IPC chains for common use cases
pub mod ipc_chains {
    use super::*;

    lazy_static::lazy_static! {
        /// Global config chain instance
        pub static ref CONFIG_CHAIN: Chain = IPCChainFactory::create_config_chain();

        /// Global message chain instance
        pub static ref MESSAGE_CHAIN: Chain = IPCChainFactory::create_message_chain();

        /// Global system chain instance
        pub static ref SYSTEM_CHAIN: Chain = IPCChainFactory::create_system_chain();

        /// Global unified IPC chain instance
        pub static ref UNIFIED_IPC_CHAIN: Chain = IPCChainFactory::create_unified_ipc_chain();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contexts::*;
    use std::collections::HashMap;

    #[tokio::test]
    async fn test_config_chain() {
        let chain = IPCChainFactory::create_config_chain();

        // Create test context with proper initialization
        let ctx = RustDeskChainContext::new(
            "test-peer".to_string(),
            crate::types::ConnType::DEFAULT_CONN,
            Some("test-user".to_string())
        );
        let mut ipc_data = HashMap::new();
        ipc_data.insert("ipc_action".to_string(), serde_json::Value::String("config".to_string()));
        ipc_data.insert("config_name".to_string(), serde_json::Value::String("test_config".to_string()));
        ipc_data.insert("config_value".to_string(), serde_json::Value::String("test_value".to_string()));

        // Create a CodeUChain context with the IPC data
        let mut data = HashMap::new();
        data.insert("ipc_data".to_string(), serde_json::to_value(ipc_data).unwrap());
        let codeuchain_ctx = codeuchain::Context::new(data);

        // Run the chain
        let result = chain.run(codeuchain_ctx).await;
        assert!(result.is_ok(), "Config chain should succeed");

        println!("✅ Config chain test passed");
    }

    #[tokio::test]
    async fn test_message_chain() {
        let chain = IPCChainFactory::create_message_chain();

        // Create test context with proper initialization
        let ctx = RustDeskChainContext::new(
            "test-peer".to_string(),
            crate::types::ConnType::DEFAULT_CONN,
            Some("test-user".to_string())
        );
        let mut ipc_data = HashMap::new();
        ipc_data.insert("ipc_action".to_string(), serde_json::Value::String("chat".to_string()));
        ipc_data.insert("chat_text".to_string(), serde_json::Value::String("Hello, World!".to_string()));

        // Create a CodeUChain context with the IPC data
        let mut data = HashMap::new();
        data.insert("ipc_data".to_string(), serde_json::to_value(ipc_data).unwrap());
        let codeuchain_ctx = codeuchain::Context::new(data);

        // Run the chain
        let result = chain.run(codeuchain_ctx).await;
        assert!(result.is_ok(), "Message chain should succeed");

        println!("✅ Message chain test passed");
    }

    #[tokio::test]
    async fn test_system_chain() {
        let chain = IPCChainFactory::create_system_chain();

        // Create test context with proper initialization
        let ctx = RustDeskChainContext::new(
            "test-peer".to_string(),
            crate::types::ConnType::DEFAULT_CONN,
            Some("test-user".to_string())
        );
        let mut ipc_data = HashMap::new();
        ipc_data.insert("ipc_action".to_string(), serde_json::Value::String("system_info".to_string()));

        // Create a CodeUChain context with the IPC data
        let mut data = HashMap::new();
        data.insert("ipc_data".to_string(), serde_json::to_value(ipc_data).unwrap());
        let codeuchain_ctx = codeuchain::Context::new(data);

        // Run the chain
        let result = chain.run(codeuchain_ctx).await;
        assert!(result.is_ok(), "System chain should succeed");

        println!("✅ System chain test passed");
    }

    #[tokio::test]
    async fn test_unified_ipc_chain_routing() {
        let chain = IPCChainFactory::create_unified_ipc_chain();

        // Test config routing
        let ctx = RustDeskChainContext::new(
            "test-peer".to_string(),
            crate::types::ConnType::DEFAULT_CONN,
            Some("test-user".to_string())
        );
        let mut ipc_data = HashMap::new();
        ipc_data.insert("ipc_action".to_string(), serde_json::Value::String("config".to_string()));
        ipc_data.insert("config_name".to_string(), serde_json::Value::String("id".to_string()));
        let mut data = HashMap::new();
        data.insert("ipc_data".to_string(), serde_json::to_value(ipc_data).unwrap());
        let codeuchain_ctx = codeuchain::Context::new(data);

        let result = chain.run(codeuchain_ctx).await;
        assert!(result.is_ok(), "Unified chain config routing should succeed");

        // Test message routing
        let ctx = RustDeskChainContext::new(
            "test-peer".to_string(),
            crate::types::ConnType::DEFAULT_CONN,
            Some("test-user".to_string())
        );
        let mut ipc_data = HashMap::new();
        ipc_data.insert("ipc_action".to_string(), serde_json::Value::String("login".to_string()));
        ipc_data.insert("peer_id".to_string(), serde_json::Value::String("test-peer".to_string()));
        ipc_data.insert("login_id".to_string(), serde_json::Value::String("123".to_string()));
        let mut data = HashMap::new();
        data.insert("ipc_data".to_string(), serde_json::to_value(ipc_data).unwrap());
        let codeuchain_ctx = codeuchain::Context::new(data);

        let result = chain.run(codeuchain_ctx).await;
        assert!(result.is_ok(), "Unified chain message routing should succeed");

        println!("✅ Unified IPC chain routing test passed");
    }
}