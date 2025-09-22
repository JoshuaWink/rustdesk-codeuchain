use crate::types::*;
use crate::contexts::*;
use codeuchain::{Context, LegacyLink};
use async_trait::async_trait;
use std::result::Result as StdResult;
use std::sync::Arc;
use serde_json;

/// Type aliases for compatibility
pub type ResultType<T> = crate::types::Result<T>;
pub type LinkResult<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

/// IPC Config Validator Link - Validates configuration requests
pub struct ConfigValidatorLink;

impl ConfigValidatorLink {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl LegacyLink for ConfigValidatorLink {
    async fn call(&self, ctx: Context) -> LinkResult<Context> {
        let data = ctx.data().clone();

        // Extract IPC data from context
        let ipc_data_value = data.get("ipc_data")
            .ok_or_else(|| CodeUChainError::ValidationError("Missing IPC data".to_string()))?;
        let ipc_data: serde_json::Value = serde_json::from_value(ipc_data_value.clone())
            .map_err(|_| CodeUChainError::ValidationError("Invalid IPC data format".to_string()))?;

        // Extract IPC action
        let action = ipc_data.get("ipc_action")
            .and_then(|v| v.as_str())
            .ok_or_else(|| CodeUChainError::ValidationError("Missing IPC action".to_string()))?;

        match action {
            "config" => {
                // Validate config name
                let config_name = ipc_data.get("config_name")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| CodeUChainError::ValidationError("Missing config name".to_string()))?;

                if config_name.trim().is_empty() {
                    return Err(Box::new(CodeUChainError::ValidationError("Empty config name".to_string())));
                }
                println!("✅ Config validation passed for: {}", config_name);
            }
            "options" => {
                println!("✅ Options validation passed");
            }
            _ => {
                return Err(Box::new(CodeUChainError::ValidationError(format!("Unknown IPC action: {}", action))));
            }
        }

        Ok(ctx)
    }
}

/// IPC Config Processor Link - Processes configuration requests
pub struct ConfigProcessorLink;

impl ConfigProcessorLink {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl LegacyLink for ConfigProcessorLink {
    async fn call(&self, ctx: Context) -> LinkResult<Context> {
        let data = ctx.data().clone();

        // Extract IPC data
        let ipc_data_value = data.get("ipc_data")
            .ok_or_else(|| CodeUChainError::ValidationError("Missing IPC data".to_string()))?;
        let ipc_data: serde_json::Value = serde_json::from_value(ipc_data_value.clone())
            .map_err(|_| CodeUChainError::ValidationError("Invalid IPC data format".to_string()))?;

        let action = ipc_data.get("ipc_action")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        match action {
            "config" => {
                let config_name = ipc_data.get("config_name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");

                // Process config get/set
                if let Some(config_value) = ipc_data.get("config_value") {
                    // Set operation
                    let value_str = config_value.as_str().unwrap_or("");
                    println!("📝 Setting config {} = {}", config_name, value_str);

                    // In real implementation, would update actual config storage
                    // For now, just log the operation
                } else {
                    // Get operation
                    println!("📖 Getting config: {}", config_name);

                    // In real implementation, would retrieve from config storage
                    // For now, return mock response
                    let mock_response = match config_name {
                        "id" => Some("rustdesk_test_id".to_string()),
                        "permanent-password" => Some("mock_password".to_string()),
                        _ => None,
                    };

                    if let Some(response) = mock_response {
                        let mut new_data = data.clone();
                        new_data.insert("config_response".to_string(), serde_json::Value::String(response));
                        return Ok(Context::new(new_data));
                    }
                }
            }
            "options" => {
                if let Some(options_str) = ipc_data.get("options").and_then(|v| v.as_str()) {
                    // Set options
                    if let Ok(options) = serde_json::from_str::<std::collections::HashMap<String, String>>(options_str) {
                        println!("📝 Setting options: {:?}", options);
                        // In real implementation, would update options storage
                    }
                } else {
                    // Get options
                    println!("📖 Getting options");
                    let mock_options = std::collections::HashMap::from([
                        ("test_option".to_string(), "test_value".to_string()),
                        ("enable_hwcodec".to_string(), "true".to_string()),
                    ]);

                    let mut new_data = data.clone();
                    new_data.insert("options_response".to_string(),
                        serde_json::Value::String(serde_json::to_string(&mock_options).unwrap_or_default()));
                    return Ok(Context::new(new_data));
                }
            }
            _ => {}
        }

        Ok(ctx)
    }
}

/// IPC Message Validator Link - Validates message requests
pub struct MessageValidatorLink;

impl MessageValidatorLink {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl LegacyLink for MessageValidatorLink {
    async fn call(&self, ctx: Context) -> LinkResult<Context> {
        let data = ctx.data().clone();

        // Extract IPC data
        let ipc_data_value = data.get("ipc_data")
            .ok_or_else(|| CodeUChainError::ValidationError("Missing IPC data".to_string()))?;
        let ipc_data: serde_json::Value = serde_json::from_value(ipc_data_value.clone())
            .map_err(|_| CodeUChainError::ValidationError("Invalid IPC data format".to_string()))?;

        let action = ipc_data.get("ipc_action")
            .and_then(|v| v.as_str())
            .ok_or_else(|| CodeUChainError::ValidationError("Missing IPC action".to_string()))?;

        match action {
            "login" => {
                // Validate login data
                let peer_id = ipc_data.get("peer_id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| CodeUChainError::ValidationError("Missing peer_id in login".to_string()))?;

                if peer_id.trim().is_empty() {
                    return Err(Box::new(CodeUChainError::ValidationError("Empty peer_id".to_string())));
                }

                println!("✅ Login validation passed for peer: {}", peer_id);
            }
            "chat" => {
                // Validate chat message
                let chat_text = ipc_data.get("chat_text")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| CodeUChainError::ValidationError("Missing chat text".to_string()))?;

                if chat_text.len() > 10000 { // Reasonable message limit
                    return Err(Box::new(CodeUChainError::ValidationError("Chat message too long".to_string())));
                }

                println!("✅ Chat message validation passed");
            }
            "close" => {
                println!("✅ Close message validation passed");
            }
            "test" => {
                println!("✅ Test message validation passed");
            }
            _ => {
                return Err(Box::new(CodeUChainError::ValidationError(format!("Unknown message action: {}", action))));
            }
        }

        Ok(ctx)
    }
}

/// IPC Message Processor Link - Processes message requests
pub struct MessageProcessorLink;

impl MessageProcessorLink {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl LegacyLink for MessageProcessorLink {
    async fn call(&self, ctx: Context) -> LinkResult<Context> {
        let data = ctx.data().clone();

        // Extract IPC data
        let ipc_data_value = data.get("ipc_data")
            .ok_or_else(|| CodeUChainError::ValidationError("Missing IPC data".to_string()))?;
        let ipc_data: serde_json::Value = serde_json::from_value(ipc_data_value.clone())
            .map_err(|_| CodeUChainError::ValidationError("Invalid IPC data format".to_string()))?;

        let action = ipc_data.get("ipc_action")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        match action {
            "login" => {
                let login_id = ipc_data.get("login_id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("0");
                let peer_id = ipc_data.get("peer_id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let is_file_transfer = ipc_data.get("is_file_transfer")
                    .and_then(|v| v.as_str())
                    .map(|s| s == "true")
                    .unwrap_or(false);

                println!("🔐 Processing login - ID: {}, Peer: {}, FileTransfer: {}",
                    login_id, peer_id, is_file_transfer);

                // In real implementation, would handle authentication logic
            }
            "chat" => {
                let chat_text = ipc_data.get("chat_text")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");

                println!("💬 Processing chat message: {}", chat_text);

                // In real implementation, would handle chat message routing
            }
            "close" => {
                println!("🔌 Processing close message");

                // In real implementation, would handle connection cleanup
            }
            "test" => {
                println!("🧪 Processing test message");

                // Test message - could trigger diagnostics
            }
            _ => {}
        }

        Ok(ctx)
    }
}

/// IPC System Info Link - Provides system information
pub struct SystemInfoLink;

impl SystemInfoLink {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl LegacyLink for SystemInfoLink {
    async fn call(&self, ctx: Context) -> LinkResult<Context> {
        let data = ctx.data().clone();

        // Extract IPC data
        let ipc_data_value = data.get("ipc_data")
            .ok_or_else(|| CodeUChainError::ValidationError("Missing IPC data".to_string()))?;
        let ipc_data: serde_json::Value = serde_json::from_value(ipc_data_value.clone())
            .map_err(|_| CodeUChainError::ValidationError("Invalid IPC data format".to_string()))?;

        let action = ipc_data.get("ipc_action")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        if action == "system_info" {
            // Gather system information
            let info = format!(
                "log_path: /tmp/rustdesk.log, config: /tmp/config.toml, username: {}",
                whoami::username()
            );

            println!("ℹ️  System info: {}", info);

            let mut new_data = data.clone();
            new_data.insert("system_info_response".to_string(), serde_json::Value::String(info));
            Ok(Context::new(new_data))
        } else {
            Ok(ctx)
        }
    }
}

/// IPC Router Link - Routes IPC requests based on action type
pub struct IPCRouterLink {
    pub config_validator: Option<Box<dyn LegacyLink + Send + Sync>>,
    pub config_processor: Option<Box<dyn LegacyLink + Send + Sync>>,
    pub message_validator: Option<Box<dyn LegacyLink + Send + Sync>>,
    pub message_processor: Option<Box<dyn LegacyLink + Send + Sync>>,
    pub system_info: Option<Box<dyn LegacyLink + Send + Sync>>,
}

impl IPCRouterLink {
    pub fn new() -> Self {
        Self {
            config_validator: None,
            config_processor: None,
            message_validator: None,
            message_processor: None,
            system_info: None,
        }
    }

    pub fn with_config_validator(mut self, link: Box<dyn LegacyLink + Send + Sync>) -> Self {
        self.config_validator = Some(link);
        self
    }

    pub fn with_config_processor(mut self, link: Box<dyn LegacyLink + Send + Sync>) -> Self {
        self.config_processor = Some(link);
        self
    }

    pub fn with_message_validator(mut self, link: Box<dyn LegacyLink + Send + Sync>) -> Self {
        self.message_validator = Some(link);
        self
    }

    pub fn with_message_processor(mut self, link: Box<dyn LegacyLink + Send + Sync>) -> Self {
        self.message_processor = Some(link);
        self
    }

    pub fn with_system_info(mut self, link: Box<dyn LegacyLink + Send + Sync>) -> Self {
        self.system_info = Some(link);
        self
    }
}

#[async_trait]
impl LegacyLink for IPCRouterLink {
    async fn call(&self, ctx: Context) -> LinkResult<Context> {
        let data = ctx.data().clone();

        // Extract IPC data from context
        let ipc_data_value = data.get("ipc_data")
            .ok_or_else(|| CodeUChainError::ValidationError("Missing IPC data".to_string()))?;
        let ipc_data: serde_json::Value = serde_json::from_value(ipc_data_value.clone())
            .map_err(|_| CodeUChainError::ValidationError("Invalid IPC data format".to_string()))?;

        // Extract IPC action
        let action = ipc_data.get("ipc_action")
            .and_then(|v| v.as_str())
            .ok_or_else(|| CodeUChainError::ValidationError("Missing IPC action".to_string()))?;

        // Route based on action type
        match action {
            "config" | "options" => {
                // Route to config validator first, then processor
                let mut current_ctx = ctx;
                if let Some(validator) = &self.config_validator {
                    current_ctx = validator.call(current_ctx).await?;
                }
                if let Some(processor) = &self.config_processor {
                    current_ctx = processor.call(current_ctx).await?;
                }
                Ok(current_ctx)
            }
            "login" | "chat" | "close" | "test" => {
                // Route to message validator first, then processor
                let mut current_ctx = ctx;
                if let Some(validator) = &self.message_validator {
                    current_ctx = validator.call(current_ctx).await?;
                }
                if let Some(processor) = &self.message_processor {
                    current_ctx = processor.call(current_ctx).await?;
                }
                Ok(current_ctx)
            }
            "system_info" => {
                // Route to system info link
                if let Some(link) = &self.system_info {
                    link.call(ctx).await
                } else {
                    Ok(ctx)
                }
            }
            _ => {
                Err(Box::new(CodeUChainError::ValidationError(format!("Unknown IPC action: {}", action))))
            }
        }
    }
}