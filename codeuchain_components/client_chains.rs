use codeuchain::core::Chain;
use crate::client_links::*;
use crate::middleware::*;
use crate::types::*;
use crate::contexts::*;
use codeuchain::{Context, LegacyLink};
use async_trait::async_trait;
use std::sync::Arc;

/// Client Orchestrator Link - coordinates all client operations sequentially
pub struct ClientOrchestratorLink {
    connection_link: ConnectionEstablishmentLink,
    media_link: MediaStreamingLink,
    input_link: InputProcessingLink,
    file_link: FileTransferLink,
    quality_link: QualityControlLink,
}

impl ClientOrchestratorLink {
    pub fn new() -> Self {
        Self {
            connection_link: ConnectionEstablishmentLink::new(),
            media_link: MediaStreamingLink::new(),
            input_link: InputProcessingLink::new(),
            file_link: FileTransferLink::new(),
            quality_link: QualityControlLink::new(),
        }
    }
}

#[async_trait]
impl LegacyLink for ClientOrchestratorLink {
    async fn call(&self, ctx: Context) -> LinkResult<Context> {
        println!("🎯 Starting client orchestration");

        // Step 1: Establish connection
        let ctx = self.connection_link.call(ctx).await?;
        println!("✅ Connection established");

        // Step 2: Initialize media streaming
        let ctx = self.media_link.call(ctx).await?;
        println!("✅ Media streaming initialized");

        // Step 3: Setup input processing
        let ctx = self.input_link.call(ctx).await?;
        println!("✅ Input processing initialized");

        // Step 4: Setup file transfer (parallel to other operations)
        let ctx = self.file_link.call(ctx).await?;
        println!("✅ File transfer initialized");

        // Step 5: Initialize quality control
        let ctx = self.quality_link.call(ctx).await?;
        println!("✅ Quality control initialized");

        println!("🎉 Client orchestration completed successfully");
        Ok(ctx)
    }
}

/// Client Chain Factory - creates processing chains for different client workflows
pub struct ClientChainFactory;

impl ClientChainFactory {
    /// Create a comprehensive client chain that handles all client operations
    pub fn create_client_chain() -> Chain {
        let mut chain = Chain::new();

        // Create single orchestrator link that handles all client operations sequentially
        let orchestrator = ClientOrchestratorLink::new();
        chain.add_link("client_orchestrator".to_string(), Box::new(orchestrator));

        // Add comprehensive middleware stack
        chain.use_middleware(Box::new(LoggingMiddleware::new().with_level("info")));
        chain.use_middleware(Box::new(PerformanceMiddleware::new()));
        chain.use_middleware(Box::new(SecurityMiddleware::new()));
        chain.use_middleware(Box::new(RateLimitMiddleware::new(1000))); // High rate limit for client operations

        chain
    }

    /// Create a connection sub-chain for connection establishment only
    pub fn create_connection_subchain() -> Chain {
        let mut chain = Chain::new();

        let connection_link = ConnectionEstablishmentLink::new();
        chain.add_link("connection".to_string(), Box::new(connection_link));

        chain.use_middleware(Box::new(LoggingMiddleware::new().with_level("debug")));
        chain.use_middleware(Box::new(SecurityMiddleware::new()));

        chain
    }

    /// Create a streaming sub-chain for media operations
    pub fn create_streaming_subchain() -> Chain {
        let mut chain = Chain::new();

        let media_link = MediaStreamingLink::new();
        let quality_link = QualityControlLink::new();

        chain.add_link("media".to_string(), Box::new(media_link));
        chain.add_link("quality".to_string(), Box::new(quality_link));

        chain.connect("media".to_string(), "quality".to_string(), |_| true);

        chain.use_middleware(Box::new(PerformanceMiddleware::new()));
        chain.use_middleware(Box::new(LoggingMiddleware::new().with_level("info")));

        chain
    }

    /// Create an input sub-chain for input handling
    pub fn create_input_subchain() -> Chain {
        let mut chain = Chain::new();

        let input_link = InputProcessingLink::new();
        chain.add_link("input".to_string(), Box::new(input_link));

        chain.use_middleware(Box::new(SecurityMiddleware::new()));
        chain.use_middleware(Box::new(RateLimitMiddleware::new(500))); // Moderate rate limit for input

        chain
    }
}

/// Pre-configured client chains for common use cases
pub mod client_chains {
    use super::*;

    lazy_static::lazy_static! {
        /// Global client chain instance
        pub static ref CLIENT_CHAIN: Chain = ClientChainFactory::create_client_chain();

        /// Global connection sub-chain instance
        pub static ref CONNECTION_SUBCHAIN: Chain = ClientChainFactory::create_connection_subchain();

        /// Global streaming sub-chain instance
        pub static ref STREAMING_SUBCHAIN: Chain = ClientChainFactory::create_streaming_subchain();

        /// Global input sub-chain instance
        pub static ref INPUT_SUBCHAIN: Chain = ClientChainFactory::create_input_subchain();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contexts::*;
    use std::collections::HashMap;

    #[tokio::test]
    async fn test_client_chain() {
        let chain = ClientChainFactory::create_client_chain();

        // Create test context with connection parameters
        let ctx = RustDeskChainContext::new(
            "test-peer-123".to_string(),
            crate::types::ConnType::DEFAULT_CONN,
            Some("test-user".to_string())
        );

        let mut connection_params = HashMap::new();
        connection_params.insert("peer_id".to_string(), serde_json::Value::String("test-peer-123".to_string()));
        connection_params.insert("relay_server".to_string(), serde_json::Value::String("test.relay.com".to_string()));
        connection_params.insert("key".to_string(), serde_json::Value::String("test-key".to_string()));

        // Create a CodeUChain context with the connection parameters
        let mut data = HashMap::new();
        data.insert("connection_params".to_string(), serde_json::to_value(connection_params).unwrap());
        let codeuchain_ctx = codeuchain::Context::new(data);

        // Run the chain
        let result = chain.run(codeuchain_ctx).await;
        match &result {
            Ok(ctx) => {
                println!("Chain succeeded. Final context keys: {:?}", ctx.data().keys().collect::<Vec<_>>());
                for (key, value) in ctx.data().iter() {
                    println!("  {}: {:?}", key, value);
                }
            }
            Err(e) => println!("Chain failed with error: {:?}", e),
        }
        assert!(result.is_ok(), "Client chain should succeed: {:?}", result.err());

        let result_ctx = result.unwrap();

        // Verify all links processed successfully
        assert_eq!(result_ctx.get("connection_status").unwrap().as_str().unwrap(), "established");
        assert!(result_ctx.get("media_streaming_ready").unwrap().as_bool().unwrap());
        assert!(result_ctx.get("input_processing_ready").unwrap().as_bool().unwrap());
        assert!(result_ctx.get("file_transfer_ready").unwrap().as_bool().unwrap());
        assert!(result_ctx.get("quality_control_active").unwrap().as_bool().unwrap());

        println!("✅ Client chain test passed");
    }

    #[tokio::test]
    async fn test_connection_subchain() {
        let chain = ClientChainFactory::create_connection_subchain();

        let mut connection_params = HashMap::new();
        connection_params.insert("peer_id".to_string(), serde_json::Value::String("test-peer".to_string()));
        connection_params.insert("relay_server".to_string(), serde_json::Value::String("relay.example.com".to_string()));

        let mut data = HashMap::new();
        data.insert("connection_params".to_string(), serde_json::to_value(connection_params).unwrap());
        let codeuchain_ctx = codeuchain::Context::new(data);

        let result = chain.run(codeuchain_ctx).await;
        assert!(result.is_ok(), "Connection subchain should succeed");

        println!("✅ Connection subchain test passed");
    }

    #[tokio::test]
    async fn test_streaming_subchain() {
        let chain = ClientChainFactory::create_streaming_subchain();

        // Pre-establish connection for streaming subchain
        let mut data = HashMap::new();
        data.insert("connection_status".to_string(), serde_json::Value::String("established".to_string()));
        let codeuchain_ctx = codeuchain::Context::new(data);

        let result = chain.run(codeuchain_ctx).await;
        assert!(result.is_ok(), "Streaming subchain should succeed");

        let result_ctx = result.unwrap();
        assert!(result_ctx.get("media_streaming_ready").unwrap().as_bool().unwrap());
        assert!(result_ctx.get("quality_control_active").unwrap().as_bool().unwrap());

        println!("✅ Streaming subchain test passed");
    }
}