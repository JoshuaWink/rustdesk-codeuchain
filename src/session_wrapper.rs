// CodeUChain Session Wrapper for RustDesk Integration
// Implements the Interface trait to provide seamless integration with existing RustDesk UI session handling

use crate::client::{Data, Interface, LoginConfigHandler};
use crate::ui_session_interface::InvokeUiSession;
use crate::message_proto::*;
use crate::rendezvous_proto::ConnType;
use hbb_common::tokio::sync::mpsc;
use hbb_common::{allow_err, config::*, log, message_proto, rendezvous_proto};
use std::sync::{Arc, RwLock};
use async_trait::async_trait;
use std::collections::HashMap;

// Import CodeUChain components
use codeuchain_rustdesk::{
    chains::ClientChain,
    contexts::RustDeskChainContext,
    middleware::{LoggingMiddleware, PerformanceMiddleware, ErrorHandlingMiddleware},
    types::*,
};

/// CodeUChain-based session wrapper that implements the Interface trait
/// This allows RustDesk to use CodeUChain's modular Links and Chains for message processing
/// while maintaining full compatibility with the existing UI session interface
pub struct CodeUChainSession<T: InvokeUiSession> {
    /// The UI handler for communicating with Flutter UI
    ui_handler: T,
    /// The login configuration handler
    lc: Arc<RwLock<LoginConfigHandler>>,
    /// Channel sender for sending data to the io_loop
    sender: Arc<RwLock<Option<mpsc::UnboundedSender<Data>>>>,
    /// The CodeUChain client chain for processing messages
    chain: ClientChain,
    /// Current session context
    current_context: Arc<RwLock<Option<RustDeskChainContext>>>,
}

impl<T: InvokeUiSession> CodeUChainSession<T> {
    /// Create a new CodeUChain session with default middleware stack
    pub fn new(ui_handler: T, lc: Arc<RwLock<LoginConfigHandler>>) -> Self {
        let mut chain = ClientChain::new();

        // Add comprehensive middleware stack
        chain = chain
            .with_middleware(LoggingMiddleware::new().with_level("info"))
            .with_middleware(PerformanceMiddleware::new())
            .with_middleware(ErrorHandlingMiddleware::new().with_retries(3))
            .with_middleware(ErrorHandlingMiddleware::new());

        Self {
            ui_handler,
            lc,
            sender: Arc::new(RwLock::new(None)),
            chain,
            current_context: Arc::new(RwLock::new(None)),
        }
    }

    /// Set the channel sender for communication with io_loop
    pub fn set_sender(&self, sender: mpsc::UnboundedSender<Data>) {
        *self.sender.write().unwrap() = Some(sender);
    }

    /// Process a message through the CodeUChain
    pub async fn process_message(&self, data: Data) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Convert RustDesk Data to CodeUChain context
        let context = self.data_to_context(&data)?;

        // Process through CodeUChain
        let result_context = self.chain.process(context).await?;

        // Update current context
        let rustdesk_context = RustDeskChainContext::from_context(result_context);
        *self.current_context.write().unwrap() = Some(rustdesk_context);

        Ok(())
    }

    /// Convert RustDesk Data enum to CodeUChain context
    fn data_to_context(&self, data: &Data) -> Result<codeuchain::Context, Box<dyn std::error::Error + Send + Sync>> {
        let mut context_data = HashMap::new();

        match data {
            Data::Message(msg) => {
                // Extract message type and create appropriate context
                if let Some(video_frame) = msg.video_frame.as_ref() {
                    let rustdesk_ctx = RustDeskContext::Streaming {
                        session: self.get_current_session()?,
                        video_frame: Some(VideoFrame {
                            timestamp: std::time::SystemTime::now()
                                .duration_since(std::time::UNIX_EPOCH)?
                                .as_millis() as u64,
                            width: video_frame.width as u32,
                            height: video_frame.height as u32,
                            data: video_frame.data.clone(),
                            format: "Unknown".to_string(), // Would need to map from codec
                            codec: "Unknown".to_string(),
                        }),
                        audio_frame: None,
                        clipboard: None,
                        pending_input: vec![],
                    };
                    context_data.insert("rustdesk_context".to_string(), serde_json::to_value(rustdesk_ctx)?);
                } else if let Some(audio_frame) = msg.audio_frame.as_ref() {
                    let rustdesk_ctx = RustDeskContext::Streaming {
                        session: self.get_current_session()?,
                        video_frame: None,
                        audio_frame: Some(AudioFrame {
                            timestamp: std::time::SystemTime::now()
                                .duration_since(std::time::UNIX_EPOCH)?
                                .as_millis() as u64,
                            sample_rate: audio_frame.sample_rate() as u32,
                            channels: audio_frame.channels as u16,
                            data: audio_frame.data.clone(),
                            format: "Unknown".to_string(),
                        }),
                        clipboard: None,
                        pending_input: vec![],
                    };
                    context_data.insert("rustdesk_context".to_string(), serde_json::to_value(rustdesk_ctx)?);
                } else if let Some(clipboard) = msg.clipboard.as_ref() {
                    let rustdesk_ctx = RustDeskContext::Streaming {
                        session: self.get_current_session()?,
                        video_frame: None,
                        audio_frame: None,
                        clipboard: Some(ClipboardData {
                            timestamp: std::time::SystemTime::now()
                                .duration_since(std::time::UNIX_EPOCH)?
                                .as_millis() as u64,
                            content_type: "text".to_string(), // Would need to determine from clipboard type
                            data: clipboard.data.clone(),
                            size: clipboard.data.len(),
                        }),
                        pending_input: vec![],
                    };
                    context_data.insert("rustdesk_context".to_string(), serde_json::to_value(rustdesk_ctx)?);
                } else {
                    // Generic message - create basic connected context
                    let rustdesk_ctx = RustDeskContext::Connected(self.get_current_session()?);
                    context_data.insert("rustdesk_context".to_string(), serde_json::to_value(rustdesk_ctx)?);
                }
            }
            Data::Login(username, password, remember) => {
                // Login data - create initial context
                let connection_info = ConnectionInfo {
                    peer_id: self.lc.read().unwrap().id.clone(),
                    conn_type: ConnType::DEFAULT_CONN,
                    secure_key: None,
                    local_addr: None,
                    peer_addr: None,
                };
                let rustdesk_ctx = RustDeskContext::Initial(connection_info);
                context_data.insert("rustdesk_context".to_string(), serde_json::to_value(rustdesk_ctx)?);
            }
            _ => {
                // For other data types, use current context or create basic one
                if let Some(current) = self.current_context.read().unwrap().as_ref() {
                    let rustdesk_ctx = current.data()?;
                    context_data.insert("rustdesk_context".to_string(), serde_json::to_value(rustdesk_ctx)?);
                } else {
                    // Create basic connected context as fallback
                    let connection_info = ConnectionInfo {
                        peer_id: self.lc.read().unwrap().id.clone(),
                        conn_type: ConnType::DEFAULT_CONN,
                        secure_key: None,
                        local_addr: None,
                        peer_addr: None,
                    };
                    let session = SessionContext {
                        connection_info,
                        session_id: 0, // Would need to be set properly
                        peer_info: PeerInfo {
                            version: "Unknown".to_string(),
                            platform: "Unknown".to_string(),
                            username: "Unknown".to_string(),
                            hostname: "Unknown".to_string(),
                            supported_encodings: vec![],
                        },
                        is_direct: false,
                    };
                    let rustdesk_ctx = RustDeskContext::Connected(session);
                    context_data.insert("rustdesk_context".to_string(), serde_json::to_value(rustdesk_ctx)?);
                }
            }
        }

        Ok(codeuchain::Context::new(context_data))
    }

    /// Get the current session context
    fn get_current_session(&self) -> Result<SessionContext, Box<dyn std::error::Error + Send + Sync>> {
        if let Some(context) = self.current_context.read().unwrap().as_ref() {
            if let Some(session) = context.session() {
                Ok(session.clone())
            } else {
                Err("No active session in current context".into())
            }
        } else {
            Err("No current context available".into())
        }
    }

    /// Send data through the channel
    fn send_data(&self, data: Data) {
        if let Some(sender) = self.sender.read().unwrap().as_ref() {
            allow_err!(sender.send(data));
        }
    }
}

#[async_trait]
impl<T: InvokeUiSession> Interface for CodeUChainSession<T> {
    fn get_lch(&self) -> Arc<RwLock<LoginConfigHandler>> {
        self.lc.clone()
    }

    fn send(&self, data: Data) {
        // Process the data through CodeUChain first
        let sender = self.sender.clone();
        let chain = &self.chain;
        let current_context = self.current_context.clone();

        // Spawn async task to process through CodeUChain
        tokio::spawn(async move {
            // Convert data to context and process
            if let Ok(context) = Self::data_to_context_static(&data, &current_context) {
                if let Ok(result_context) = chain.process(context).await {
                    // Update context
                    let rustdesk_context = RustDeskChainContext::from_context(result_context);
                    *current_context.write().unwrap() = Some(rustdesk_context);
                }
            }

            // Send the original data through the channel
            if let Some(sender) = sender.read().unwrap().as_ref() {
                allow_err!(sender.send(data));
            }
        });
    }

    fn msgbox(&self, msgtype: &str, title: &str, text: &str, link: &str) {
        self.ui_handler.msgbox(msgtype, title, text, link, false);
    }

    fn handle_login_error(&self, err: &str) -> bool {
        // Process login error through CodeUChain context
        if let Some(mut context) = self.current_context.write().unwrap().as_mut() {
            if let Ok(mut rustdesk_ctx) = context.data() {
                if let RustDeskContext::Connected(session) = rustdesk_ctx {
                    let error_ctx = RustDeskContext::Error {
                        session: Some(session),
                        error: err.to_string(),
                    };
                    if let Ok(new_context) = context.insert(error_ctx) {
                        *self.current_context.write().unwrap() = Some(new_context);
                    }
                }
            }
        }

        // Use default login error handling
        crate::client::handle_login_error(self.lc.clone(), err, self)
    }

    fn handle_peer_info(&self, mut pi: PeerInfo) {
        // Update session context with peer info
        if let Some(mut context) = self.current_context.write().unwrap().as_mut() {
            if let Ok(rustdesk_ctx) = context.data() {
                if let RustDeskContext::Connected(mut session) = rustdesk_ctx {
                    session.peer_info = pi.clone();
                    let updated_ctx = RustDeskContext::Connected(session);
                    if let Ok(new_context) = context.insert(updated_ctx) {
                        *self.current_context.write().unwrap() = Some(new_context);
                    }
                }
            }
        }

        // Call original peer info handling
        self.ui_handler.set_peer_info(&pi);
    }

    fn set_multiple_windows_session(&self, sessions: Vec<WindowsSession>) {
        self.ui_handler.set_multiple_windows_session(sessions);
    }

    async fn handle_hash(&self, pass: &str, hash: Hash, peer: &mut Stream) {
        crate::client::handle_hash(self.lc.clone(), pass, hash, self, peer).await;
    }

    async fn handle_login_from_ui(
        &self,
        os_username: String,
        os_password: String,
        password: String,
        remember: bool,
        peer: &mut Stream,
    ) {
        crate::client::handle_login_from_ui(
            self.lc.clone(),
            os_username,
            os_password,
            password,
            remember,
            peer,
        )
        .await;
    }

    async fn handle_test_delay(&self, t: TestDelay, peer: &mut Stream) {
        crate::client::handle_test_delay(t, peer).await;
    }
}

impl<T: InvokeUiSession> CodeUChainSession<T> {
    /// Static helper for data to context conversion (used in async context)
    fn data_to_context_static(
        data: &Data,
        current_context: &Arc<RwLock<Option<RustDeskChainContext>>>,
    ) -> Result<codeuchain::Context, Box<dyn std::error::Error + Send + Sync>> {
        let mut context_data = HashMap::new();

        // Simplified version for static context
        match data {
            Data::Message(_) => {
                // For messages, use current context or create basic one
                if let Some(context) = current_context.read().unwrap().as_ref() {
                    if let Ok(rustdesk_ctx) = context.data() {
                        context_data.insert("rustdesk_context".to_string(), serde_json::to_value(rustdesk_ctx)?);
                    }
                }
            }
            _ => {
                // For other data types, create basic context
                let connection_info = ConnectionInfo {
                    peer_id: "unknown".to_string(), // Would need proper peer ID
                    conn_type: ConnType::DEFAULT_CONN,
                    secure_key: None,
                    local_addr: None,
                    peer_addr: None,
                };
                let rustdesk_ctx = RustDeskContext::Initial(connection_info);
                context_data.insert("rustdesk_context".to_string(), serde_json::to_value(rustdesk_ctx)?);
            }
        }

        Ok(codeuchain::Context::new(context_data))
    }
}

impl<T: InvokeUiSession> std::ops::Deref for CodeUChainSession<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.ui_handler
    }
}

impl<T: InvokeUiSession> std::ops::DerefMut for CodeUChainSession<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.ui_handler
    }
}