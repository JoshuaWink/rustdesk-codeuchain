// Message Processing Links for CodeUChain-based RustDesk
// These Links handle specific protobuf message types and replace the direct message processing in io_loop.rs

use crate::types::*;
use crate::contexts::*;
use codeuchain::{Context, LegacyLink};
use async_trait::async_trait;
use std::result::Result as StdResult;
use std::sync::Arc;
use tokio::sync::mpsc;

// Import RustDesk protobuf types
use hbb_common::message_proto::*;
use hbb_common::protobuf::Message as ProtoMessage;

/// Result type for message processing links
pub type MessageResult<T> = StdResult<T, Box<dyn std::error::Error + Send + Sync>>;

/// Video Message Link - Handles VideoFrame protobuf messages
pub struct VideoMessageLink {
    pub ui_sender: Option<Arc<mpsc::UnboundedSender<UiUpdate>>>,
    pub enable_hw_accel: bool,
    pub max_fps: u32,
}

impl VideoMessageLink {
    pub fn new() -> Self {
        Self {
            ui_sender: None,
            enable_hw_accel: false,
            max_fps: 30,
        }
    }

    pub fn with_ui_sender(mut self, sender: mpsc::UnboundedSender<UiUpdate>) -> Self {
        self.ui_sender = Some(Arc::new(sender));
        self
    }

    pub fn with_hw_accel(mut self, enable: bool) -> Self {
        self.enable_hw_accel = enable;
        self
    }

    pub fn with_max_fps(mut self, fps: u32) -> Self {
        self.max_fps = fps;
        self
    }
}

#[async_trait]
impl LegacyLink for VideoMessageLink {
    async fn call(&self, ctx: Context) -> MessageResult<Context> {
        let data = ctx.data().clone();

        // Extract video frame from context data
        if let Some(video_data) = data.get("video_frame") {
            if let Ok(video_frame) = serde_json::from_value::<VideoFrame>(video_data.clone()) {
                // Process video frame (decode, render, etc.)
                self.process_video_frame(&video_frame).await?;

                // Send to UI if sender available
                if let Some(sender) = &self.ui_sender {
                    let ui_update = UiUpdate::VideoFrame(video_frame);
                    let _ = sender.send(ui_update);
                }

                // Update context with processed video data
                let mut new_data = data.clone();
                new_data.insert("last_video_timestamp".to_string(), serde_json::to_value(video_frame.timestamp)?);
                return Ok(Context::new(new_data));
            }
        }

        // If no video frame in context, pass through unchanged
        Ok(ctx)
    }
}

impl VideoMessageLink {
    async fn process_video_frame(&self, frame: &VideoFrame) -> MessageResult<()> {
        // Here we would integrate with scrap for video decoding
        // For now, just validate the frame data
        if frame.data.is_empty() {
            return Err("Empty video frame data".into());
        }

        if frame.width == 0 || frame.height == 0 {
            return Err("Invalid video frame dimensions".into());
        }

        // Calculate expected frame rate and validate
        let expected_fps = 1000.0 / (frame.timestamp as f64 - 0.0); // Simplified
        if expected_fps > self.max_fps as f64 {
            log::warn!("Video frame rate {} exceeds max_fps {}", expected_fps, self.max_fps);
        }

        Ok(())
    }
}

/// Audio Message Link - Handles AudioFrame protobuf messages
pub struct AudioMessageLink {
    pub ui_sender: Option<Arc<mpsc::UnboundedSender<UiUpdate>>>,
    pub enable_playback: bool,
    pub enable_recording: bool,
    pub buffer_size: usize,
}

impl AudioMessageLink {
    pub fn new() -> Self {
        Self {
            ui_sender: None,
            enable_playback: true,
            enable_recording: true,
            buffer_size: 1024,
        }
    }

    pub fn with_ui_sender(mut self, sender: mpsc::UnboundedSender<UiUpdate>) -> Self {
        self.ui_sender = Some(Arc::new(sender));
        self
    }

    pub fn playback_only(mut self) -> Self {
        self.enable_recording = false;
        self.enable_playback = true;
        self
    }

    pub fn recording_only(mut self) -> Self {
        self.enable_playback = false;
        self.enable_recording = true;
        self
    }
}

#[async_trait]
impl LegacyLink for AudioMessageLink {
    async fn call(&self, ctx: Context) -> MessageResult<Context> {
        let data = ctx.data().clone();

        // Extract audio frame from context data
        if let Some(audio_data) = data.get("audio_frame") {
            if let Ok(audio_frame) = serde_json::from_value::<AudioFrame>(audio_data.clone()) {
                // Process audio frame (decode, play, etc.)
                self.process_audio_frame(&audio_frame).await?;

                // Send to UI if sender available
                if let Some(sender) = &self.ui_sender {
                    let ui_update = UiUpdate::AudioFrame(audio_frame);
                    let _ = sender.send(ui_update);
                }

                // Update context with processed audio data
                let mut new_data = data.clone();
                new_data.insert("last_audio_timestamp".to_string(), serde_json::to_value(audio_frame.timestamp)?);
                return Ok(Context::new(new_data));
            }
        }

        Ok(ctx)
    }
}

impl AudioMessageLink {
    async fn process_audio_frame(&self, frame: &AudioFrame) -> MessageResult<()> {
        // Here we would integrate with cpal for audio playback
        // For now, just validate the frame data
        if frame.data.is_empty() {
            return Err("Empty audio frame data".into());
        }

        if frame.channels == 0 || frame.sample_rate == 0 {
            return Err("Invalid audio frame parameters".into());
        }

        // Validate buffer size
        let expected_samples = frame.data.len() / frame.channels as usize;
        if expected_samples > self.buffer_size {
            log::warn!("Audio frame size {} exceeds buffer size {}", expected_samples, self.buffer_size);
        }

        Ok(())
    }
}

/// Clipboard Message Link - Handles Clipboard protobuf messages
pub struct ClipboardMessageLink {
    pub ui_sender: Option<Arc<mpsc::UnboundedSender<UiUpdate>>>,
    pub max_clipboard_size: usize,
    pub enable_sync: bool,
}

impl ClipboardMessageLink {
    pub fn new() -> Self {
        Self {
            ui_sender: None,
            max_clipboard_size: 10 * 1024 * 1024, // 10MB
            enable_sync: true,
        }
    }

    pub fn with_ui_sender(mut self, sender: mpsc::UnboundedSender<UiUpdate>) -> Self {
        self.ui_sender = Some(Arc::new(sender));
        self
    }

    pub fn with_max_size(mut self, size: usize) -> Self {
        self.max_clipboard_size = size;
        self
    }

    pub fn disable_sync(mut self) -> Self {
        self.enable_sync = false;
        self
    }
}

#[async_trait]
impl LegacyLink for ClipboardMessageLink {
    async fn call(&self, ctx: Context) -> MessageResult<Context> {
        let data = ctx.data().clone();

        // Extract clipboard data from context
        if let Some(clipboard_data) = data.get("clipboard") {
            if let Ok(clipboard) = serde_json::from_value::<ClipboardData>(clipboard_data.clone()) {
                // Process clipboard data
                self.process_clipboard_data(&clipboard).await?;

                // Send to UI if sender available
                if let Some(sender) = &self.ui_sender {
                    let ui_update = UiUpdate::ClipboardUpdate(clipboard);
                    let _ = sender.send(ui_update);
                }

                // Update context with processed clipboard data
                let mut new_data = data.clone();
                new_data.insert("last_clipboard_timestamp".to_string(), serde_json::to_value(clipboard.timestamp)?);
                return Ok(Context::new(new_data));
            }
        }

        Ok(ctx)
    }
}

impl ClipboardMessageLink {
    async fn process_clipboard_data(&self, clipboard: &ClipboardData) -> MessageResult<()> {
        // Here we would integrate with arboard or clipboard-master
        // For now, just validate the clipboard data
        if clipboard.data.len() > self.max_clipboard_size {
            return Err(format!("Clipboard data size {} exceeds maximum {}", clipboard.data.len(), self.max_clipboard_size).into());
        }

        if !self.enable_sync {
            log::debug!("Clipboard sync disabled, skipping processing");
            return Ok(());
        }

        // Validate content type
        match clipboard.content_type.as_str() {
            "text" | "image" | "files" => {
                // Valid content types
            }
            _ => {
                log::warn!("Unknown clipboard content type: {}", clipboard.content_type);
            }
        }

        Ok(())
    }
}

/// Input Message Link - Handles keyboard and mouse input messages
pub struct InputMessageLink {
    pub ui_sender: Option<Arc<mpsc::UnboundedSender<UiUpdate>>>,
    pub enable_keyboard: bool,
    pub enable_mouse: bool,
    pub input_buffer_size: usize,
}

impl InputMessageLink {
    pub fn new() -> Self {
        Self {
            ui_sender: None,
            enable_keyboard: true,
            enable_mouse: true,
            input_buffer_size: 100,
        }
    }

    pub fn with_ui_sender(mut self, sender: mpsc::UnboundedSender<UiUpdate>) -> Self {
        self.ui_sender = Some(Arc::new(sender));
        self
    }

    pub fn keyboard_only(mut self) -> Self {
        self.enable_mouse = false;
        self.enable_keyboard = true;
        self
    }

    pub fn mouse_only(mut self) -> Self {
        self.enable_keyboard = false;
        self.enable_mouse = true;
        self
    }
}

#[async_trait]
impl LegacyLink for InputMessageLink {
    async fn call(&self, ctx: Context) -> MessageResult<Context> {
        let data = ctx.data().clone();

        // Extract input events from context
        if let Some(input_data) = data.get("input_events") {
            if let Ok(events) = serde_json::from_value::<Vec<InputEvent>>(input_data.clone()) {
                // Process input events
                self.process_input_events(&events).await?;

                // Send to UI if sender available (for local echo or feedback)
                if let Some(sender) = &self.ui_sender {
                    for event in &events {
                        // Input events are typically sent to the remote peer, not UI
                        // But we might want to send feedback for some events
                    }
                }

                // Update context with processed input events
                let mut new_data = data.clone();
                let processed_count = events.len();
                new_data.insert("processed_input_count".to_string(), serde_json::to_value(processed_count)?);
                return Ok(Context::new(new_data));
            }
        }

        Ok(ctx)
    }
}

impl InputMessageLink {
    async fn process_input_events(&self, events: &[InputEvent]) -> MessageResult<()> {
        // Here we would integrate with rdev for input simulation
        // For now, just validate the input events
        if events.len() > self.input_buffer_size {
            return Err(format!("Input events count {} exceeds buffer size {}", events.len(), self.input_buffer_size).into());
        }

        for event in events {
            if event.data.is_empty() {
                return Err("Empty input event data".into());
            }

            match event.event_type.as_str() {
                "keyboard" if !self.enable_keyboard => {
                    return Err("Keyboard input disabled".into());
                }
                "mouse" if !self.enable_mouse => {
                    return Err("Mouse input disabled".into());
                }
                "keyboard" | "mouse" => {
                    // Valid event types
                }
                _ => {
                    log::warn!("Unknown input event type: {}", event.event_type);
                }
            }
        }

        Ok(())
    }
}

/// File Transfer Message Link - Handles file transfer messages
pub struct FileTransferMessageLink {
    pub ui_sender: Option<Arc<mpsc::UnboundedSender<UiUpdate>>>,
    pub max_file_size: u64,
    pub enable_compression: bool,
    pub chunk_size: usize,
}

impl FileTransferMessageLink {
    pub fn new() -> Self {
        Self {
            ui_sender: None,
            max_file_size: 100 * 1024 * 1024, // 100MB
            enable_compression: true,
            chunk_size: 64 * 1024, // 64KB
        }
    }

    pub fn with_ui_sender(mut self, sender: mpsc::UnboundedSender<UiUpdate>) -> Self {
        self.ui_sender = Some(Arc::new(sender));
        self
    }

    pub fn with_max_file_size(mut self, size: u64) -> Self {
        self.max_file_size = size;
        self
    }

    pub fn disable_compression(mut self) -> Self {
        self.enable_compression = false;
        self
    }
}

#[async_trait]
impl LegacyLink for FileTransferMessageLink {
    async fn call(&self, ctx: Context) -> MessageResult<Context> {
        let data = ctx.data().clone();

        // Extract file transfer data from context
        if let Some(file_data) = data.get("file_transfer") {
            // Process file transfer data
            self.process_file_transfer(file_data).await?;

            // Update context with file transfer status
            let mut new_data = data.clone();
            new_data.insert("file_transfer_active".to_string(), serde_json::to_value(true)?);
            return Ok(Context::new(new_data));
        }

        Ok(ctx)
    }
}

impl FileTransferMessageLink {
    async fn process_file_transfer(&self, file_data: &serde_json::Value) -> MessageResult<()> {
        // Here we would integrate with RustDesk's file transfer logic
        // For now, just validate the file data structure
        if let Some(size) = file_data.get("size").and_then(|s| s.as_u64()) {
            if size > self.max_file_size {
                return Err(format!("File size {} exceeds maximum {}", size, self.max_file_size).into());
            }
        }

        if let Some(chunks) = file_data.get("chunks").and_then(|c| c.as_array()) {
            for chunk in chunks {
                if let Some(chunk_size) = chunk.get("size").and_then(|s| s.as_u64()) {
                    if chunk_size > self.chunk_size as u64 {
                        return Err(format!("Chunk size {} exceeds maximum {}", chunk_size, self.chunk_size).into());
                    }
                }
            }
        }

        Ok(())
    }
}

/// Message Router Link - Routes incoming messages to appropriate processing links
pub struct MessageRouterLink {
    pub video_link: Option<Arc<dyn LegacyLink + Send + Sync>>,
    pub audio_link: Option<Arc<dyn LegacyLink + Send + Sync>>,
    pub clipboard_link: Option<Arc<dyn LegacyLink + Send + Sync>>,
    pub input_link: Option<Arc<dyn LegacyLink + Send + Sync>>,
    pub file_transfer_link: Option<Arc<dyn LegacyLink + Send + Sync>>,
}

impl MessageRouterLink {
    pub fn new() -> Self {
        Self {
            video_link: None,
            audio_link: None,
            clipboard_link: None,
            input_link: None,
            file_transfer_link: None,
        }
    }

    pub fn with_video_link(mut self, link: Arc<dyn LegacyLink + Send + Sync>) -> Self {
        self.video_link = Some(link);
        self
    }

    pub fn with_audio_link(mut self, link: Arc<dyn LegacyLink + Send + Sync>) -> Self {
        self.audio_link = Some(link);
        self
    }

    pub fn with_clipboard_link(mut self, link: Arc<dyn LegacyLink + Send + Sync>) -> Self {
        self.clipboard_link = Some(link);
        self
    }

    pub fn with_input_link(mut self, link: Arc<dyn LegacyLink + Send + Sync>) -> Self {
        self.input_link = Some(link);
        self
    }

    pub fn with_file_transfer_link(mut self, link: Arc<dyn LegacyLink + Send + Sync>) -> Self {
        self.file_transfer_link = Some(link);
        self
    }
}

#[async_trait]
impl LegacyLink for MessageRouterLink {
    async fn call(&self, ctx: Context) -> MessageResult<Context> {
        let data = ctx.data().clone();

        // Determine message type and route to appropriate link
        if data.contains_key("video_frame") {
            if let Some(link) = &self.video_link {
                return link.call(ctx).await;
            }
        } else if data.contains_key("audio_frame") {
            if let Some(link) = &self.audio_link {
                return link.call(ctx).await;
            }
        } else if data.contains_key("clipboard") {
            if let Some(link) = &self.clipboard_link {
                return link.call(ctx).await;
            }
        } else if data.contains_key("input_events") {
            if let Some(link) = &self.input_link {
                return link.call(ctx).await;
            }
        } else if data.contains_key("file_transfer") {
            if let Some(link) = &self.file_transfer_link {
                return link.call(ctx).await;
            }
        }

        // If no specific link matches, pass through unchanged
        Ok(ctx)
    }
}