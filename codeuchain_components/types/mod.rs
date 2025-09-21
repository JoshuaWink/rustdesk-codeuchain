// CodeUChain-based RustDesk - Core Types and Contexts

use codeuchain::core::{Context, Link};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Connection types for remote desktop sessions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConnType {
    DEFAULT_CONN,
    FILE_TRANSFER,
    PORT_FORWARD,
    RDP,
}

/// Basic connection information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionInfo {
    pub peer_id: String,
    pub conn_type: ConnType,
    pub secure_key: Option<Vec<u8>>,
    pub local_addr: Option<String>,
    pub peer_addr: Option<String>,
}

/// Session context after connection is established
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionContext {
    pub connection_info: ConnectionInfo,
    pub session_id: u64,
    pub peer_info: PeerInfo,
    pub is_direct: bool,
}

/// Peer information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerInfo {
    pub version: String,
    pub platform: String,
    pub username: String,
    pub hostname: String,
    pub supported_encodings: Vec<String>,
}

/// Video frame data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoFrame {
    pub timestamp: u64,
    pub width: u32,
    pub height: u32,
    pub data: Vec<u8>,
    pub format: String,
    pub codec: String,
}

/// Audio frame data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioFrame {
    pub timestamp: u64,
    pub sample_rate: u32,
    pub channels: u16,
    pub data: Vec<i16>,
    pub format: String,
}

/// Clipboard data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipboardData {
    pub timestamp: u64,
    pub content_type: String,
    pub data: Vec<u8>,
    pub size: usize,
}

/// Input event (keyboard/mouse)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputEvent {
    pub timestamp: u64,
    pub event_type: String,
    pub data: Vec<u8>,
}

/// UI update data
#[derive(Debug, Clone)]
pub enum UiUpdate {
    VideoFrame(VideoFrame),
    AudioFrame(AudioFrame),
    ClipboardUpdate(ClipboardData),
    ConnectionStatus(String),
}

/// Main application context that evolves through the processing chain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RustDeskContext {
    Initial(ConnectionInfo),
    Connected(SessionContext),
    Streaming {
        session: SessionContext,
        video_frame: Option<VideoFrame>,
        audio_frame: Option<AudioFrame>,
        clipboard: Option<ClipboardData>,
        pending_input: Vec<InputEvent>,
    },
    Error {
        session: Option<SessionContext>,
        error: String,
    },
}

impl RustDeskContext {
    /// Get the current session if available
    pub fn session(&self) -> Option<&SessionContext> {
        match self {
            RustDeskContext::Connected(session) => Some(session),
            RustDeskContext::Streaming { session, .. } => Some(session),
            RustDeskContext::Error { session, .. } => session.as_ref(),
            _ => None,
        }
    }

    /// Check if context represents an error state
    pub fn is_error(&self) -> bool {
        matches!(self, RustDeskContext::Error { .. })
    }

    /// Get error message if in error state
    pub fn error_message(&self) -> Option<&str> {
        match self {
            RustDeskContext::Error { error, .. } => Some(error),
            _ => None,
        }
    }
}

/// Trait for context conversion helpers
pub trait ContextHelpers {
    fn as_rustdesk_context(&self) -> Option<&RustDeskContext>;
    fn into_rustdesk_context(self) -> Option<RustDeskContext>;
}

impl<T> ContextHelpers for Context<T> {
    fn as_rustdesk_context(&self) -> Option<&RustDeskContext> {
        // This would need to be implemented based on how we store the context
        // For now, returning None as placeholder
        None
    }

    fn into_rustdesk_context(self) -> Option<RustDeskContext> {
        // This would need to be implemented based on how we store the context
        // For now, returning None as placeholder
        None
    }
}