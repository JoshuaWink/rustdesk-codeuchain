// Context management for CodeUChain-based RustDesk

use crate::types::*;
use codeuchain::core::Context;
use std::collections::HashMap;
use serde_json;

/// Wrapper for CodeUChain Context with RustDesk-specific data
pub struct RustDeskChainContext {
    inner: Context,
}

impl RustDeskChainContext {
    /// Create a new initial context
    pub fn new(peer_id: String, conn_type: ConnType) -> Self {
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
        }
    }

    /// Create from existing CodeUChain context
    pub fn from_context(context: Context) -> Self {
        Self { inner: context }
    }

    /// Get the current RustDesk context data
    pub fn data(&self) -> Result<RustDeskContext, Box<dyn std::error::Error + Send + Sync>> {
        let json_value = self.inner.data().get("rustdesk_context")
            .ok_or("Missing rustdesk_context")?;
        Ok(serde_json::from_value(json_value.clone())?)
    }

    /// Insert new RustDesk context data, creating a new context
    pub fn insert(self, rustdesk_data: RustDeskContext) -> Result<RustDeskChainContext, Box<dyn std::error::Error + Send + Sync>> {
        let mut new_data = self.inner.data().clone();
        new_data.insert("rustdesk_context".to_string(), serde_json::to_value(rustdesk_data)?);
        Ok(RustDeskChainContext {
            inner: Context::new(new_data),
        })
    }

    /// Get the underlying CodeUChain context
    pub fn into_inner(self) -> Context {
        self.inner
    }

    /// Check if context is in error state
    pub fn is_error(&self) -> bool {
        self.data().map(|ctx| ctx.is_error()).unwrap_or(true)
    }

    /// Get current session if available
    pub fn session(&self) -> Option<SessionContext> {
        self.data().ok()?.session().cloned()
    }

    /// Get connection info if available
    pub fn connection_info(&self) -> Option<ConnectionInfo> {
        match self.data().ok()? {
            RustDeskContext::Initial(info) => Some(info.clone()),
            RustDeskContext::Connected(session) => Some(session.connection_info.clone()),
            RustDeskContext::Streaming { session, .. } => Some(session.connection_info.clone()),
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