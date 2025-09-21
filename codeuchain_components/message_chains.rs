// Message Processing Chains for CodeUChain-based RustDesk
// Orchestrates message routing and processing based on protobuf message types

use crate::types::*;
use crate::contexts::*;
use crate::message_links::*;
use crate::middleware::*;
use codeuchain::{Chain, Context, LegacyLink};
use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::mpsc;

// Import RustDesk protobuf types
use hbb_common::message_proto::*;

/// Message Processing Chain - Routes and processes incoming messages
pub struct MessageProcessingChain {
    chain: Chain,
    ui_sender: Option<Arc<mpsc::UnboundedSender<UiUpdate>>>,
}

impl MessageProcessingChain {
    pub fn new() -> Self {
        let mut chain = Chain::new();

        // Create message processing links
        let video_link = Arc::new(VideoMessageLink::new());
        let audio_link = Arc::new(AudioMessageLink::new());
        let clipboard_link = Arc::new(ClipboardMessageLink::new());
        let input_link = Arc::new(InputMessageLink::new());
        let file_transfer_link = Arc::new(FileTransferMessageLink::new());

        // Create router link that routes based on message type
        let router_link = Arc::new(MessageRouterLink::new()
            .with_video_link(video_link)
            .with_audio_link(audio_link)
            .with_clipboard_link(clipboard_link)
            .with_input_link(input_link)
            .with_file_transfer_link(file_transfer_link));

        // Add router as the main processing link
        chain.add_link("message_router".to_string(), router_link);

        // Add comprehensive middleware stack
        chain = chain.with_middleware(LoggingMiddleware::new().with_level("info"));
        chain = chain.with_middleware(PerformanceMiddleware::new());
        chain = chain.with_middleware(ErrorHandlingMiddleware::new().with_retries(3));
        chain = chain.with_middleware(SecurityMiddleware::new());

        Self {
            chain,
            ui_sender: None,
        }
    }

    pub fn with_ui_sender(mut self, sender: mpsc::UnboundedSender<UiUpdate>) -> Self {
        self.ui_sender = Some(Arc::new(sender));

        // Update all links with the UI sender
        if let Some(link) = self.chain.get_link_mut("message_router") {
            // This would need to be implemented to update the router's links
            // For now, we'll handle this in the process_message method
        }

        self
    }

    /// Process a single message through the chain
    pub async fn process_message(&self, message: &Message, session_ctx: &SessionContext) -> Result<Context, Box<dyn std::error::Error + Send + Sync>> {
        // Convert protobuf message to CodeUChain context
        let context = self.message_to_context(message, session_ctx)?;

        // Process through the chain
        self.chain.run(context).await
    }

    /// Convert protobuf Message to CodeUChain context
    fn message_to_context(&self, message: &Message, session_ctx: &SessionContext) -> Result<Context, Box<dyn std::error::Error + Send + Sync>> {
        let mut context_data = std::collections::HashMap::new();

        // Add session context
        let rustdesk_ctx = RustDeskContext::Streaming {
            session: session_ctx.clone(),
            video_frame: None,
            audio_frame: None,
            clipboard: None,
            pending_input: vec![],
        };
        context_data.insert("rustdesk_context".to_string(), serde_json::to_value(rustdesk_ctx)?);

        // Convert message based on its type
        if let Some(video_frame) = message.video_frame.as_ref() {
            let frame = VideoFrame {
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)?
                    .as_millis() as u64,
                width: video_frame.width as u32,
                height: video_frame.height as u32,
                data: video_frame.data.clone(),
                format: "Unknown".to_string(), // Would need to map from codec
                codec: "Unknown".to_string(),
            };
            context_data.insert("video_frame".to_string(), serde_json::to_value(frame)?);
        }

        if let Some(audio_frame) = message.audio_frame.as_ref() {
            let frame = AudioFrame {
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)?
                    .as_millis() as u64,
                sample_rate: audio_frame.sample_rate() as u32,
                channels: audio_frame.channels as u16,
                data: audio_frame.data.clone(),
                format: "Unknown".to_string(),
            };
            context_data.insert("audio_frame".to_string(), serde_json::to_value(frame)?);
        }

        if let Some(clipboard) = message.clipboard.as_ref() {
            let clipboard_data = ClipboardData {
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)?
                    .as_millis() as u64,
                content_type: "text".to_string(), // Would need to determine from clipboard type
                data: clipboard.data.clone(),
                size: clipboard.data.len(),
            };
            context_data.insert("clipboard".to_string(), serde_json::to_value(clipboard_data)?);
        }

        // Handle keyboard/mouse input events
        let mut input_events = Vec::new();

        if message.key_event.is_some() {
            let input_event = InputEvent {
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)?
                    .as_millis() as u64,
                event_type: "keyboard".to_string(),
                data: vec![1, 2, 3, 4], // Would need proper serialization
            };
            input_events.push(input_event);
        }

        if message.mouse_event.is_some() {
            let input_event = InputEvent {
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)?
                    .as_millis() as u64,
                event_type: "mouse".to_string(),
                data: vec![1, 2, 3, 4], // Would need proper serialization
            };
            input_events.push(input_event);
        }

        if !input_events.is_empty() {
            context_data.insert("input_events".to_string(), serde_json::to_value(input_events)?);
        }

        // Handle file transfer messages
        if message.file_transfer.is_some() {
            // Would need to extract file transfer data
            let file_transfer_data = serde_json::json!({
                "active": true,
                "size": 0,
                "chunks": []
            });
            context_data.insert("file_transfer".to_string(), file_transfer_data);
        }

        Ok(Context::new(context_data))
    }

    /// Get the underlying chain for advanced operations
    pub fn chain(&self) -> &Chain {
        &self.chain
    }

    /// Get mutable access to the underlying chain
    pub fn chain_mut(&mut self) -> &mut Chain {
        &mut self.chain
    }
}

/// Connection Management Chain - Handles connection establishment and lifecycle
pub struct ConnectionManagementChain {
    chain: Chain,
}

impl ConnectionManagementChain {
    pub fn new() -> Self {
        let mut chain = Chain::new();

        // Add connection-related links
        chain.add_link("connection_establishment".to_string(), Box::new(crate::links::ConnectionLink::new()));
        chain.add_link("connection_monitoring".to_string(), Box::new(ConnectionMonitoringLink::new()));
        chain.add_link("connection_cleanup".to_string(), Box::new(ConnectionCleanupLink::new()));

        // Set up connection lifecycle flow
        chain.connect("connection_establishment".to_string(), "connection_monitoring".to_string(), |ctx| {
            // Only proceed to monitoring if connection was successful
            if let Some(rustdesk_ctx_json) = ctx.data().get("rustdesk_context") {
                if let Ok(rustdesk_ctx) = serde_json::from_value::<RustDeskContext>(rustdesk_ctx_json.clone()) {
                    matches!(rustdesk_ctx, RustDeskContext::Connected(_))
                } else {
                    false
                }
            } else {
                false
            }
        });

        // Add middleware for connection management
        chain = chain.with_middleware(LoggingMiddleware::new().with_level("info"));
        chain = chain.with_middleware(PerformanceMiddleware::new());
        chain = chain.with_middleware(ErrorHandlingMiddleware::new().with_retries(5)); // More retries for connections
        chain = chain.with_middleware(SecurityMiddleware::new().with_encryption(true));

        Self { chain }
    }

    /// Process connection-related operations
    pub async fn process_connection(&self, ctx: Context) -> Result<Context, Box<dyn std::error::Error + Send + Sync>> {
        self.chain.run(ctx).await
    }
}

/// Connection Monitoring Link - Monitors connection health and quality
pub struct ConnectionMonitoringLink {
    pub ping_interval_ms: u64,
    pub max_latency_ms: u64,
    pub enable_quality_reporting: bool,
}

impl ConnectionMonitoringLink {
    pub fn new() -> Self {
        Self {
            ping_interval_ms: 5000, // 5 seconds
            max_latency_ms: 1000,   // 1 second
            enable_quality_reporting: true,
        }
    }

    pub fn with_ping_interval(mut self, interval_ms: u64) -> Self {
        self.ping_interval_ms = interval_ms;
        self
    }

    pub fn with_max_latency(mut self, latency_ms: u64) -> Self {
        self.max_latency_ms = latency_ms;
        self
    }
}

#[async_trait]
impl LegacyLink for ConnectionMonitoringLink {
    async fn call(&self, ctx: Context) -> Result<Context, Box<dyn std::error::Error + Send + Sync>> {
        let data = ctx.data().clone();

        // Check connection health
        let connection_healthy = self.check_connection_health().await?;

        let mut new_data = data.clone();
        new_data.insert("connection_healthy".to_string(), serde_json::to_value(connection_healthy)?);

        if self.enable_quality_reporting {
            let quality_metrics = self.measure_connection_quality().await?;
            new_data.insert("connection_quality".to_string(), serde_json::to_value(quality_metrics)?);
        }

        Ok(Context::new(new_data))
    }
}

impl ConnectionMonitoringLink {
    async fn check_connection_health(&self) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        // Here we would implement actual connection health checking
        // For now, return true
        Ok(true)
    }

    async fn measure_connection_quality(&self) -> Result<serde_json::Value, Box<dyn std::error::Error + Send + Sync>> {
        // Here we would measure latency, bandwidth, packet loss, etc.
        // For now, return mock metrics
        let metrics = serde_json::json!({
            "latency_ms": 45,
            "bandwidth_mbps": 50.5,
            "packet_loss_percent": 0.1,
            "jitter_ms": 2
        });
        Ok(metrics)
    }
}

/// Connection Cleanup Link - Handles connection teardown and resource cleanup
pub struct ConnectionCleanupLink {
    pub cleanup_timeout_ms: u64,
}

impl ConnectionCleanupLink {
    pub fn new() -> Self {
        Self {
            cleanup_timeout_ms: 5000, // 5 seconds
        }
    }

    pub fn with_timeout(mut self, timeout_ms: u64) -> Self {
        self.cleanup_timeout_ms = timeout_ms;
        self
    }
}

#[async_trait]
impl LegacyLink for ConnectionCleanupLink {
    async fn call(&self, ctx: Context) -> Result<Context, Box<dyn std::error::Error + Send + Sync>> {
        let data = ctx.data().clone();

        // Perform cleanup operations
        self.perform_cleanup().await?;

        let mut new_data = data.clone();
        new_data.insert("connection_cleaned".to_string(), serde_json::to_value(true)?);

        Ok(Context::new(new_data))
    }
}

impl ConnectionCleanupLink {
    async fn perform_cleanup(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Here we would clean up resources, close connections, etc.
        // For now, just simulate cleanup
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        Ok(())
    }
}

/// Complete Remote Desktop Chain - Combines all processing chains
pub struct RemoteDesktopProcessingChain {
    message_chain: MessageProcessingChain,
    connection_chain: ConnectionManagementChain,
}

impl RemoteDesktopProcessingChain {
    pub fn new() -> Self {
        Self {
            message_chain: MessageProcessingChain::new(),
            connection_chain: ConnectionManagementChain::new(),
        }
    }

    pub fn with_ui_sender(mut self, sender: mpsc::UnboundedSender<UiUpdate>) -> Self {
        self.message_chain = self.message_chain.with_ui_sender(sender);
        self
    }

    /// Process a message through the appropriate chain
    pub async fn process(&self, message: &Message, session_ctx: &SessionContext) -> Result<Context, Box<dyn std::error::Error + Send + Sync>> {
        // Route to message processing chain
        self.message_chain.process_message(message, session_ctx).await
    }

    /// Process connection operations
    pub async fn process_connection(&self, ctx: Context) -> Result<Context, Box<dyn std::error::Error + Send + Sync>> {
        self.connection_chain.process_connection(ctx).await
    }

    /// Get access to individual chains for configuration
    pub fn message_chain(&self) -> &MessageProcessingChain {
        &self.message_chain
    }

    pub fn message_chain_mut(&mut self) -> &mut MessageProcessingChain {
        &mut self.message_chain
    }

    pub fn connection_chain(&self) -> &ConnectionManagementChain {
        &self.connection_chain
    }

    pub fn connection_chain_mut(&mut self) -> &mut ConnectionManagementChain {
        &mut self.connection_chain
    }
}

/// Helper functions for chain management
pub mod helpers {
    use super::*;

    /// Create a default message processing chain
    pub fn create_message_chain() -> MessageProcessingChain {
        MessageProcessingChain::new()
    }

    /// Create a default connection management chain
    pub fn create_connection_chain() -> ConnectionManagementChain {
        ConnectionManagementChain::new()
    }

    /// Create a complete remote desktop processing chain
    pub fn create_remote_desktop_chain() -> RemoteDesktopProcessingChain {
        RemoteDesktopProcessingChain::new()
    }

    /// Create a context for message processing
    pub fn create_message_context(message: &Message, session: &SessionContext) -> Result<Context, Box<dyn std::error::Error + Send + Sync>> {
        let chain = MessageProcessingChain::new();
        chain.message_to_context(message, session)
    }
}