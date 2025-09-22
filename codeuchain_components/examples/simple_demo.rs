// Comprehensive example of CodeUChain-based RustDesk end-to-end processing

use codeuchain::{Context, Link};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::collections::HashMap;

// Simplified types for the example
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RustDeskContext {
    Initial(ConnectionInfo),
    Connected(SessionContext),
    Streaming {
        session: SessionContext,
        video_frame: Option<VideoFrame>,
        audio_frame: Option<AudioFrame>,
        clipboard_data: Option<ClipboardData>,
        input_events: Vec<InputEvent>,
    },
    Error {
        session: Option<SessionContext>,
        error: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionInfo {
    pub peer_id: String,
    pub password: Option<String>,
    pub conn_type: ConnType,
    pub secure_key: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConnType {
    DEFAULT_CONN,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionContext {
    pub connection_info: ConnectionInfo,
    pub session_id: u64,
    pub peer_info: PeerInfo,
    pub is_direct: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerInfo {
    pub version: String,
    pub platform: String,
    pub username: String,
    pub hostname: String,
    pub supported_encodings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoFrame {
    pub timestamp: u64,
    pub width: usize,
    pub height: usize,
    pub data: Vec<u8>,
    pub format: String,
    pub codec: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioFrame {
    pub timestamp: u64,
    pub sample_rate: u32,
    pub channels: u16,
    pub data: Vec<i16>,
    pub format: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipboardData {
    pub timestamp: u64,
    pub content_type: String,
    pub data: Vec<u8>,
    pub size: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputEvent {
    pub timestamp: u64,
    pub event_type: String,
    pub data: Vec<u8>,
}

// Simplified Connection Link
pub struct ConnectionLink;

#[async_trait]
impl Link for ConnectionLink {
    async fn call(&self, ctx: Context) -> Result<Context, Box<dyn std::error::Error + Send + Sync>> {
        let data = ctx.data().clone();
        let rustdesk_ctx_json = data.get("rustdesk_context")
            .ok_or("Missing rustdesk_context")?;
        let rustdesk_ctx: RustDeskContext = serde_json::from_value(rustdesk_ctx_json.clone())?;

        match rustdesk_ctx {
            RustDeskContext::Initial(connection_info) => {
                // Mock connection establishment
                let peer_info = PeerInfo {
                    version: "1.2.3".to_string(),
                    platform: "Test Platform".to_string(),
                    username: "testuser".to_string(),
                    hostname: "testhost".to_string(),
                    supported_encodings: vec!["vp8".to_string(), "vp9".to_string()],
                };

                let session = SessionContext {
                    connection_info,
                    session_id: rand::random::<u64>(),
                    peer_info,
                    is_direct: true,
                };

                let new_data = RustDeskContext::Connected(session);
                let mut new_data_map = data.clone();
                new_data_map.insert("rustdesk_context".to_string(), serde_json::to_value(new_data)?);
                Ok(Context::new(new_data_map))
            }
            _ => Ok(ctx),
        }
    }
}

// Simplified Video Link
pub struct VideoLink;

#[async_trait]
impl Link for VideoLink {
    async fn call(&self, ctx: Context) -> Result<Context, Box<dyn std::error::Error + Send + Sync>> {
        let data = ctx.data().clone();
        let rustdesk_ctx_json = data.get("rustdesk_context")
            .ok_or("Missing rustdesk_context")?;
        let rustdesk_ctx: RustDeskContext = serde_json::from_value(rustdesk_ctx_json.clone())?;

        match rustdesk_ctx {
            RustDeskContext::Connected(session) => {
                let video_frame = VideoFrame {
                    timestamp: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_millis() as u64,
                    width: 1920,
                    height: 1080,
                    data: vec![0u8; 1920 * 1080 * 4],
                    format: "RGBA".to_string(),
                    codec: "vp8".to_string(),
                };

                let streaming_data = RustDeskContext::Streaming {
                    session,
                    video_frame: Some(video_frame),
                    audio_frame: None,
                    clipboard_data: None,
                    input_events: vec![],
                };

                let mut new_data_map = data.clone();
                new_data_map.insert("rustdesk_context".to_string(), serde_json::to_value(streaming_data)?);
                Ok(Context::new(new_data_map))
            }
            RustDeskContext::Streaming { mut session, video_frame, audio_frame, clipboard_data, input_events } => {
                let new_video_frame = VideoFrame {
                    timestamp: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_millis() as u64,
                    width: 1920,
                    height: 1080,
                    data: vec![0u8; 1920 * 1080 * 4],
                    format: "RGBA".to_string(),
                    codec: "vp8".to_string(),
                };

                let streaming_data = RustDeskContext::Streaming {
                    session,
                    video_frame: Some(new_video_frame),
                    audio_frame,
                    clipboard_data,
                    input_events,
                };
                let mut new_data_map = data.clone();
                new_data_map.insert("rustdesk_context".to_string(), serde_json::to_value(streaming_data)?);
                Ok(Context::new(new_data_map))
            }
            _ => Ok(ctx),
        }
    }
}

// Simplified Audio Link
pub struct AudioLink;

#[async_trait]
impl Link for AudioLink {
    async fn call(&self, ctx: Context) -> Result<Context, Box<dyn std::error::Error + Send + Sync>> {
        let data = ctx.data().clone();
        let rustdesk_ctx_json = data.get("rustdesk_context")
            .ok_or("Missing rustdesk_context")?;
        let rustdesk_ctx: RustDeskContext = serde_json::from_value(rustdesk_ctx_json.clone())?;

        match rustdesk_ctx {
            RustDeskContext::Streaming { mut session, video_frame, audio_frame, clipboard_data, input_events } => {
                let new_audio_frame = AudioFrame {
                    timestamp: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_millis() as u64,
                    sample_rate: 44100,
                    channels: 2,
                    data: vec![0i16; 1024],
                    format: "PCM_S16LE".to_string(),
                };

                let streaming_data = RustDeskContext::Streaming {
                    session,
                    video_frame,
                    audio_frame: Some(new_audio_frame),
                    clipboard_data,
                    input_events,
                };
                let mut new_data_map = data.clone();
                new_data_map.insert("rustdesk_context".to_string(), serde_json::to_value(streaming_data)?);
                Ok(Context::new(new_data_map))
            }
            _ => Ok(ctx),
        }
    }
}

// Simplified Clipboard Link
pub struct ClipboardLink;

#[async_trait]
impl Link for ClipboardLink {
    async fn call(&self, ctx: Context) -> Result<Context, Box<dyn std::error::Error + Send + Sync>> {
        let data = ctx.data().clone();
        let rustdesk_ctx_json = data.get("rustdesk_context")
            .ok_or("Missing rustdesk_context")?;
        let rustdesk_ctx: RustDeskContext = serde_json::from_value(rustdesk_ctx_json.clone())?;

        match rustdesk_ctx {
            RustDeskContext::Streaming { mut session, video_frame, audio_frame, clipboard_data, input_events } => {
                let new_clipboard_data = ClipboardData {
                    timestamp: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_millis() as u64,
                    content_type: "text".to_string(),
                    data: b"Hello from clipboard!".to_vec(),
                    size: 21,
                };

                let streaming_data = RustDeskContext::Streaming {
                    session,
                    video_frame,
                    audio_frame,
                    clipboard_data: Some(new_clipboard_data),
                    input_events,
                };
                let mut new_data_map = data.clone();
                new_data_map.insert("rustdesk_context".to_string(), serde_json::to_value(streaming_data)?);
                Ok(Context::new(new_data_map))
            }
            _ => Ok(ctx),
        }
    }
}

// Simplified Input Link
pub struct InputLink;

#[async_trait]
impl Link for InputLink {
    async fn call(&self, ctx: Context) -> Result<Context, Box<dyn std::error::Error + Send + Sync>> {
        let data = ctx.data().clone();
        let rustdesk_ctx_json = data.get("rustdesk_context")
            .ok_or("Missing rustdesk_context")?;
        let rustdesk_ctx: RustDeskContext = serde_json::from_value(rustdesk_ctx_json.clone())?;

        match rustdesk_ctx {
            RustDeskContext::Streaming { mut session, video_frame, audio_frame, clipboard_data, mut input_events } => {
                let new_input_event = InputEvent {
                    timestamp: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_millis() as u64,
                    event_type: "keyboard".to_string(),
                    data: vec![1, 2, 3, 4],
                };

                input_events.push(new_input_event);

                let streaming_data = RustDeskContext::Streaming {
                    session,
                    video_frame,
                    audio_frame,
                    clipboard_data,
                    input_events,
                };
                let mut new_data_map = data.clone();
                new_data_map.insert("rustdesk_context".to_string(), serde_json::to_value(streaming_data)?);
                Ok(Context::new(new_data_map))
            }
            _ => Ok(ctx),
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("CodeUChain-based RustDesk End-to-End Example");
    println!("===========================================");

    // Create remote desktop chain
    let mut remote_desktop_chain = codeuchain::Chain::new();

    // Add links
    remote_desktop_chain.add_link("connection".to_string(), Box::new(ConnectionLink));
    remote_desktop_chain.add_link("video".to_string(), Box::new(VideoLink));
    remote_desktop_chain.add_link("audio".to_string(), Box::new(AudioLink));
    remote_desktop_chain.add_link("clipboard".to_string(), Box::new(ClipboardLink));
    remote_desktop_chain.add_link("input".to_string(), Box::new(InputLink));

    // Set up processing flow
    remote_desktop_chain.connect("connection".to_string(), "video".to_string(), |ctx| {
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

    remote_desktop_chain.connect("video".to_string(), "audio".to_string(), |ctx| {
        if let Some(rustdesk_ctx_json) = ctx.data().get("rustdesk_context") {
            if let Ok(rustdesk_ctx) = serde_json::from_value::<RustDeskContext>(rustdesk_ctx_json.clone()) {
                matches!(rustdesk_ctx, RustDeskContext::Streaming { .. })
            } else {
                false
            }
        } else {
            false
        }
    });

    remote_desktop_chain.connect("audio".to_string(), "clipboard".to_string(), |ctx| {
        if let Some(rustdesk_ctx_json) = ctx.data().get("rustdesk_context") {
            if let Ok(rustdesk_ctx) = serde_json::from_value::<RustDeskContext>(rustdesk_ctx_json.clone()) {
                matches!(rustdesk_ctx, RustDeskContext::Streaming { .. })
            } else {
                false
            }
        } else {
            false
        }
    });

    remote_desktop_chain.connect("clipboard".to_string(), "input".to_string(), |ctx| {
        if let Some(rustdesk_ctx_json) = ctx.data().get("rustdesk_context") {
            if let Ok(rustdesk_ctx) = serde_json::from_value::<RustDeskContext>(rustdesk_ctx_json.clone()) {
                matches!(rustdesk_ctx, RustDeskContext::Streaming { .. })
            } else {
                false
            }
        } else {
            false
        }
    });

    // Example: Peer ID Connection
    println!("\n--- Example: Peer ID Connection via Rendezvous ---");
    let peer_id_context = create_connection_context("123456789");
    println!("Initial context: Peer ID connection to 123456789");

    let peer_result = remote_desktop_chain.run(peer_id_context).await;
    match peer_result {
        Ok(result) => println!("Result: {:?}", result.data()),
        Err(e) => println!("Error: {}", e),
    }

    // Example: Full Streaming Session
    println!("\n--- Example: Full Streaming Session ---");
    let streaming_context = create_streaming_context();
    println!("Initial context: Starting streaming session");

    // Process multiple frames to simulate streaming
    let mut current_context = streaming_context;
    for frame in 0..3 {
        println!("Processing frame {}", frame + 1);
        match remote_desktop_chain.run(current_context).await {
            Ok(new_context) => {
                current_context = new_context;

                if let Some(rustdesk_ctx_json) = current_context.data().get("rustdesk_context") {
                    if let Ok(rustdesk_ctx) = serde_json::from_value::<RustDeskContext>(rustdesk_ctx_json.clone()) {
                        match rustdesk_ctx {
                            RustDeskContext::Streaming { video_frame, audio_frame, clipboard_data, input_events, .. } => {
                                if video_frame.is_some() {
                                    println!("  ✓ Video frame captured");
                                }
                                if audio_frame.is_some() {
                                    println!("  ✓ Audio frame processed");
                                }
                                if clipboard_data.is_some() {
                                    println!("  ✓ Clipboard data synchronized");
                                }
                                if !input_events.is_empty() {
                                    println!("  ✓ {} input events processed", input_events.len());
                                }
                            }
                            RustDeskContext::Error { error, .. } => {
                                println!("  ✗ Error: {}", error);
                                break;
                            }
                            _ => {
                                println!("  ? Unexpected context state");
                            }
                        }
                    } else {
                        println!("  ✗ Failed to deserialize context");
                        break;
                    }
                } else {
                    println!("  ✗ Missing rustdesk_context");
                    break;
                }
            }
            Err(e) => {
                println!("  ✗ Chain execution failed: {}", e);
                break;
            }
        }

        // Small delay between frames
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }

    println!("\n--- Chain Architecture Benefits ---");
    println!("✓ Modular: Each Link handles one responsibility");
    println!("✓ Composable: Links can be rearranged or replaced");
    println!("✓ Testable: Individual components can be unit tested");
    println!("✓ Observable: Middleware provides cross-cutting insights");
    println!("✓ Maintainable: Clear separation of concerns");
    println!("✓ Extensible: New Links can be added without touching existing code");

    println!("\nEnd-to-end example completed successfully!");
    Ok(())
}

/// Create a connection context for different connection types
fn create_connection_context(peer_id: &str) -> Context {
    let connection_info = ConnectionInfo {
        peer_id: peer_id.to_string(),
        password: Some("test-password".to_string()),
        conn_type: ConnType::DEFAULT_CONN,
        secure_key: Some("test-secure-key".to_string()),
    };

    let initial_context = RustDeskContext::Initial(connection_info);
    let mut data = std::collections::HashMap::new();
    data.insert("rustdesk_context".to_string(), serde_json::to_value(initial_context).unwrap());
    Context::new(data)
}

/// Create a streaming context to demonstrate media processing
fn create_streaming_context() -> Context {
    let connection_info = ConnectionInfo {
        peer_id: "streaming-peer-123".to_string(),
        password: Some("stream-password".to_string()),
        conn_type: ConnType::DEFAULT_CONN,
        secure_key: Some("stream-secure-key".to_string()),
    };

    let peer_info = PeerInfo {
        version: "1.2.3".to_string(),
        platform: "Test Platform".to_string(),
        username: "testuser".to_string(),
        hostname: "testhost".to_string(),
        supported_encodings: vec!["vp8".to_string(), "vp9".to_string(), "h264".to_string()],
    };

    let session = SessionContext {
        connection_info,
        session_id: 123456789,
        peer_info,
        is_direct: true,
    };

    let streaming_context = RustDeskContext::Connected(session);
    let mut data = std::collections::HashMap::new();
    data.insert("rustdesk_context".to_string(), serde_json::to_value(streaming_context).unwrap());
    Context::new(data)
}