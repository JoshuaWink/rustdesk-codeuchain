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

/// Enhanced error types for production use
#[derive(Debug, Clone)]
pub enum CodeUChainError {
    ConnectionError(String),
    AuthenticationError(String),
    MediaError(String),
    SecurityError(String),
    ConfigurationError(String),
    TimeoutError(String),
    ResourceError(String),
    ValidationError(String),
    ChainExecutionError(String),
}

impl std::fmt::Display for CodeUChainError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CodeUChainError::ConnectionError(msg) => write!(f, "Connection error: {}", msg),
            CodeUChainError::AuthenticationError(msg) => write!(f, "Authentication error: {}", msg),
            CodeUChainError::MediaError(msg) => write!(f, "Media error: {}", msg),
            CodeUChainError::SecurityError(msg) => write!(f, "Security error: {}", msg),
            CodeUChainError::ConfigurationError(msg) => write!(f, "Configuration error: {}", msg),
            CodeUChainError::TimeoutError(msg) => write!(f, "Timeout error: {}", msg),
            CodeUChainError::ResourceError(msg) => write!(f, "Resource error: {}", msg),
            CodeUChainError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
            CodeUChainError::ChainExecutionError(msg) => write!(f, "Chain execution error: {}", msg),
        }
    }
}

impl std::error::Error for CodeUChainError {}

impl From<&str> for CodeUChainError {
    fn from(s: &str) -> Self {
        CodeUChainError::ValidationError(s.to_string())
    }
}

impl From<String> for CodeUChainError {
    fn from(s: String) -> Self {
        CodeUChainError::ValidationError(s)
    }
}

impl From<serde_json::Error> for CodeUChainError {
    fn from(err: serde_json::Error) -> Self {
        CodeUChainError::ValidationError(format!("JSON error: {}", err))
    }
}

impl From<std::io::Error> for CodeUChainError {
    fn from(err: std::io::Error) -> Self {
        CodeUChainError::ResourceError(format!("IO error: {}", err))
    }
}

impl From<Box<dyn std::error::Error + Send + Sync>> for CodeUChainError {
    fn from(err: Box<dyn std::error::Error + Send + Sync>) -> Self {
        CodeUChainError::ChainExecutionError(err.to_string())
    }
}

/// Enhanced result type
pub type Result<T> = std::result::Result<T, CodeUChainError>;

/// Context metadata for tracking and debugging
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextMetadata {
    pub session_id: String,
    pub request_id: String,
    pub timestamp: u64,
    pub user_id: Option<String>,
    pub client_version: String,
    pub platform: String,
    pub security_level: SecurityLevel,
    pub priority: Priority,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecurityLevel {
    Public,
    Authenticated,
    Encrypted,
    HighSecurity,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Priority {
    Low,
    Normal,
    High,
    Critical,
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
    Completed {
        session: SessionContext,
        result: serde_json::Value,
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