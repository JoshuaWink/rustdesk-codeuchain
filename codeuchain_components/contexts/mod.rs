// Context management for CodeUChain-based RustDesk

pub use crate::types::*;
use crate::core::Context;
use std::collections::HashMap;
use serde_json;

/// Enhanced context wrapper with metadata and error handling
#[derive(Clone)]
pub struct RustDeskChainContext {
    inner: Context,
    metadata: ContextMetadata,
}

impl RustDeskChainContext {
    /// Create a new initial context with metadata
    pub fn new(peer_id: String, conn_type: ConnType, user_id: Option<String>) -> Self {
        let metadata = ContextMetadata {
            session_id: format!("session_{}", rand::random::<u64>()),
            request_id: format!("req_{}", rand::random::<u64>()),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
            user_id,
            client_version: env!("CARGO_PKG_VERSION").to_string(),
            platform: std::env::consts::OS.to_string(),
            security_level: SecurityLevel::Public,
            priority: Priority::Normal,
        };

        let connection_info = ConnectionInfo {
            peer_id,
            conn_type,
            secure_key: None,
            local_addr: None,
            peer_addr: None,
        };

        let context_data = RustDeskContext::Initial(connection_info);

        let mut data = HashMap::new();
        data.insert("rustdesk_context".to_string(), serde_json::to_value(context_data).unwrap());

        Self {
            inner: Context::new(data),
            metadata,
        }
    }

    /// Create from existing CodeUChain context
    pub fn from_context(context: Context) -> Result<Self> {
        let metadata = ContextMetadata {
            session_id: format!("session_{}", rand::random::<u64>()),
            request_id: format!("req_{}", rand::random::<u64>()),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
            user_id: None,
            client_version: env!("CARGO_PKG_VERSION").to_string(),
            platform: std::env::consts::OS.to_string(),
            security_level: SecurityLevel::Public,
            priority: Priority::Normal,
        };

        Ok(Self {
            inner: context,
            metadata,
        })
    }

    /// Get the current RustDesk context data with error handling
    pub fn data(&self) -> Result<RustDeskContext> {
        let json_value = self.inner.data().get("rustdesk_context")
            .ok_or(CodeUChainError::ValidationError("Missing rustdesk_context".to_string()))?;
        serde_json::from_value(json_value.clone())
            .map_err(|e| CodeUChainError::ValidationError(format!("Failed to deserialize context: {}", e)))
    }

    /// Insert new RustDesk context data with validation
    pub fn insert(self, rustdesk_data: RustDeskContext) -> Result<RustDeskChainContext> {
        // Validate context transition
        self.validate_transition(&rustdesk_data)?;

        let mut new_data = self.inner.data().clone();
        new_data.insert("rustdesk_context".to_string(), serde_json::to_value(&rustdesk_data)
            .map_err(|e| CodeUChainError::ValidationError(format!("Failed to serialize context: {}", e)))?);

        Ok(RustDeskChainContext {
            inner: Context::new(new_data),
            metadata: self.metadata,
        })
    }

    /// Update metadata
    pub fn with_metadata(mut self, metadata: ContextMetadata) -> Self {
        self.metadata = metadata;
        self
    }

    /// Get current metadata
    pub fn metadata(&self) -> &ContextMetadata {
        &self.metadata
    }

    /// Check if context is in error state
    pub fn is_error(&self) -> bool {
        matches!(self.data(), Ok(RustDeskContext::Error { .. }))
    }

    /// Get error information if in error state
    pub fn error_info(&self) -> Option<String> {
        if let Ok(RustDeskContext::Error { error, .. }) = self.data() {
            Some(error.clone())
        } else {
            None
        }
    }

    /// Validate context state transitions
    fn validate_transition(&self, new_context: &RustDeskContext) -> Result<()> {
        let current = self.data()?;

        let valid = match (&current, new_context) {
            (RustDeskContext::Initial(_), RustDeskContext::Connected(_)) => true,
            (RustDeskContext::Initial(_), RustDeskContext::Error { .. }) => true,
            (RustDeskContext::Connected(_), RustDeskContext::Streaming { .. }) => true,
            (RustDeskContext::Connected(_), RustDeskContext::Error { .. }) => true,
            (RustDeskContext::Streaming { .. }, RustDeskContext::Streaming { .. }) => true,
            (RustDeskContext::Streaming { .. }, RustDeskContext::Completed { .. }) => true,
            (RustDeskContext::Streaming { .. }, RustDeskContext::Error { .. }) => true,
            (RustDeskContext::Error { .. }, RustDeskContext::Initial(_)) => true, // Retry
            (RustDeskContext::Error { .. }, RustDeskContext::Connected(_)) => true, // Recovery
            (RustDeskContext::Completed { .. }, RustDeskContext::Initial(_)) => true, // New session
            _ => false,
        };

        if !valid {
            return Err(CodeUChainError::ValidationError(
                format!("Invalid context transition from {:?} to {:?}", current, new_context)
            ));
        }

        Ok(())
    }

    /// Get the underlying CodeUChain context
    pub fn into_inner(self) -> Context {
        self.inner
    }

    /// Get session info if available
    pub fn session(&self) -> Option<SessionContext> {
        match self.data().ok()? {
            RustDeskContext::Connected(session) => Some(session),
            RustDeskContext::Streaming { session, .. } => Some(session),
            RustDeskContext::Completed { session, .. } => Some(session),
            RustDeskContext::Error { session, .. } => session,
            _ => None,
        }
    }

    /// Get connection info if available
    pub fn connection_info(&self) -> Option<ConnectionInfo> {
        match self.data().ok()? {
            RustDeskContext::Initial(connection_info) => Some(connection_info),
            RustDeskContext::Connected(session) => Some(session.connection_info),
            RustDeskContext::Streaming { session, .. } => Some(session.connection_info),
            RustDeskContext::Completed { session, .. } => Some(session.connection_info),
            RustDeskContext::Error { session, .. } => session.as_ref().map(|s| s.connection_info.clone()),
        }
    }
}

/// Helper functions for context manipulation
pub mod helpers {
    use super::*;

    /// Create a connected context from connection info
    pub fn create_connected_context(
        connection_info: ConnectionInfo,
        session_id: u64,
        peer_info: PeerInfo,
        is_direct: bool,
    ) -> RustDeskContext {
        let session = SessionContext {
            connection_info,
            session_id,
            peer_info,
            is_direct,
        };
        RustDeskContext::Connected(session)
    }

    /// Create a streaming context with media data
    pub fn create_streaming_context(
        session: SessionContext,
        video_frame: Option<VideoFrame>,
        audio_frame: Option<AudioFrame>,
        clipboard: Option<ClipboardData>,
        pending_input: Vec<InputEvent>,
    ) -> RustDeskContext {
        RustDeskContext::Streaming {
            session,
            video_frame,
            audio_frame,
            clipboard,
            pending_input,
        }
    }

    /// Create an error context
    pub fn create_error_context(session: Option<SessionContext>, error: String) -> RustDeskContext {
        RustDeskContext::Error { session, error }
    }

    /// Extract video frame from streaming context
    pub fn extract_video_frame(context: &RustDeskContext) -> Option<&VideoFrame> {
        match context {
            RustDeskContext::Streaming { video_frame, .. } => video_frame.as_ref(),
            _ => None,
        }
    }

    /// Extract audio frame from streaming context
    pub fn extract_audio_frame(context: &RustDeskContext) -> Option<&AudioFrame> {
        match context {
            RustDeskContext::Streaming { audio_frame, .. } => audio_frame.as_ref(),
            _ => None,
        }
    }

    /// Extract clipboard data from streaming context
    pub fn extract_clipboard(context: &RustDeskContext) -> Option<&ClipboardData> {
        match context {
            RustDeskContext::Streaming { clipboard, .. } => clipboard.as_ref(),
            _ => None,
        }
    }

    /// Extract pending input events from streaming context
    pub fn extract_pending_input(context: &RustDeskContext) -> Option<&[InputEvent]> {
        match context {
            RustDeskContext::Streaming { pending_input, .. } => Some(pending_input),
            _ => None,
        }
    }
}