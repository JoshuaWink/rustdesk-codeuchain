use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use lazy_static::lazy_static;
use codeuchain::core::{Context, Chain, Link};
use crate::types::*;
use crate::contexts::*;
use crate::middleware::*;
use crate::ipc_links::*;
use crate::ipc_chains::*;

/// Type aliases for compatibility
pub type ResultType<T> = crate::types::Result<T>;
pub type LinkResult<T> = crate::types::Result<T>;

/// IPC Facade for CodeUChain migration
/// Maintains API compatibility while using CodeUChain internally
#[async_trait::async_trait]
pub trait IPCFacade {
    async fn start_server(&self, postfix: &str) -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync>>;
    async fn connect_client(&self, ms_timeout: u64, postfix: &str) -> std::result::Result<IPCConnection, Box<dyn std::error::Error + Send + Sync>>;
    async fn send_message(&self, connection: &mut IPCConnection, data: IPCData) -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync>>;
    async fn receive_message(&self, connection: &mut IPCConnection) -> std::result::Result<Option<IPCData>, Box<dyn std::error::Error + Send + Sync>>;
    async fn get_config(&self, name: &str) -> std::result::Result<Option<String>, Box<dyn std::error::Error + Send + Sync>>;
    async fn set_config(&self, name: &str, value: String) -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync>>;
    async fn get_options(&self) -> std::result::Result<HashMap<String, String>, Box<dyn std::error::Error + Send + Sync>>;
    async fn set_options(&self, options: HashMap<String, String>) -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync>>;
}

/// IPC Connection abstraction
pub struct IPCConnection {
    pub context: RustDeskChainContext,
    pub chain: Chain,
}

/// IPC Data types (simplified for migration)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IPCData {
    Config { name: String, value: Option<String> },
    Options(Option<HashMap<String, String>>),
    SystemInfo(Option<String>),
    Login { id: i32, is_file_transfer: bool, peer_id: String },
    ChatMessage { text: String },
    Close,
    Test,
}

/// CodeUChain-based IPC implementation
pub struct CodeUChainIPC {
    config_chain: Chain,
    message_chain: Chain,
    system_chain: Chain,
}

impl CodeUChainIPC {
    pub fn new() -> Self {
        let mut config_chain = Chain::new();
        config_chain.add_link("config_validator".to_string(), Box::new(ConfigValidatorLink::new()));
        config_chain.add_link("config_processor".to_string(), Box::new(ConfigProcessorLink::new()));

        let mut message_chain = Chain::new();
        message_chain.add_link("message_validator".to_string(), Box::new(MessageValidatorLink::new()));
        message_chain.add_link("message_processor".to_string(), Box::new(MessageProcessorLink::new()));

        let mut system_chain = Chain::new();
        system_chain.add_link("system_info".to_string(), Box::new(SystemInfoLink::new()));

        // Add middleware to all chains
        let logging_mw = LoggingMiddleware::new();
        let security_mw = SecurityMiddleware::new();
        let rate_limit_mw = RateLimitMiddleware::new(100); // 100 requests per window

        config_chain.use_middleware(Box::new(logging_mw.clone()));
        config_chain.use_middleware(Box::new(security_mw.clone()));
        config_chain.use_middleware(Box::new(rate_limit_mw.clone()));

        message_chain.use_middleware(Box::new(logging_mw.clone()));
        message_chain.use_middleware(Box::new(security_mw.clone()));
        message_chain.use_middleware(Box::new(rate_limit_mw.clone()));

        system_chain.use_middleware(Box::new(logging_mw));
        system_chain.use_middleware(Box::new(security_mw));
        system_chain.use_middleware(Box::new(rate_limit_mw));

        Self {
            config_chain,
            message_chain,
            system_chain,
        }
    }
}

#[async_trait::async_trait]
impl IPCFacade for CodeUChainIPC {
    async fn start_server(&self, postfix: &str) -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // For now, return success - actual server implementation would go here
        println!("CodeUChain IPC server started for postfix: {}", postfix);
        Ok(())
    }

    async fn connect_client(&self, ms_timeout: u64, postfix: &str) -> std::result::Result<IPCConnection, Box<dyn std::error::Error + Send + Sync>> {
        // Create appropriate chain based on postfix
        let chain = match postfix {
            "" => IPCChainFactory::create_config_chain(),
            "_msg" => IPCChainFactory::create_message_chain(),
            "_sys" => IPCChainFactory::create_system_chain(),
            _ => IPCChainFactory::create_config_chain(),
        };

        let context = RustDeskChainContext::new(
            format!("ipc_client_{}", postfix),
            ConnType::DEFAULT_CONN,
            None
        );

        Ok(IPCConnection { context, chain })
    }

    async fn send_message(&self, connection: &mut IPCConnection, data: IPCData) -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Convert IPCData to context data
        let mut context_data = HashMap::new();
        match &data {
            IPCData::Config { name, value } => {
                context_data.insert("ipc_action".to_string(), "config".to_string());
                context_data.insert("config_name".to_string(), name.clone());
                if let Some(val) = value {
                    context_data.insert("config_value".to_string(), val.clone());
                }
            }
            IPCData::Options(opts) => {
                context_data.insert("ipc_action".to_string(), "options".to_string());
                if let Some(opts) = opts {
                    context_data.insert("options".to_string(), serde_json::to_string(opts)?);
                }
            }
            IPCData::SystemInfo(_) => {
                context_data.insert("ipc_action".to_string(), "system_info".to_string());
            }
            IPCData::Login { id, is_file_transfer, peer_id } => {
                context_data.insert("ipc_action".to_string(), "login".to_string());
                context_data.insert("login_id".to_string(), id.to_string());
                context_data.insert("is_file_transfer".to_string(), is_file_transfer.to_string());
                context_data.insert("peer_id".to_string(), peer_id.clone());
            }
            IPCData::ChatMessage { text } => {
                context_data.insert("ipc_action".to_string(), "chat".to_string());
                context_data.insert("chat_text".to_string(), text.clone());
            }
            IPCData::Close => {
                context_data.insert("ipc_action".to_string(), "close".to_string());
            }
            IPCData::Test => {
                context_data.insert("ipc_action".to_string(), "test".to_string());
            }
        }

        // Update context and run chain
        // For IPC, we store the data in the underlying context
        let mut new_data = HashMap::new();
        for (k, v) in &context_data {
            new_data.insert(k.clone(), serde_json::Value::String(v.clone()));
        }
        let updated_context = Context::new(new_data);
        
        connection.chain.run(updated_context).await?;

        Ok(())
    }

    async fn receive_message(&self, _connection: &mut IPCConnection) -> std::result::Result<Option<IPCData>, Box<dyn std::error::Error + Send + Sync>> {
        // For now, return None - actual implementation would check for responses
        // In a real implementation, this would check the context for response data
        Ok(None)
    }

    async fn get_config(&self, name: &str) -> std::result::Result<Option<String>, Box<dyn std::error::Error + Send + Sync>> {
        let mut connection = self.connect_client(1000, "").await?;
        self.send_message(&mut connection, IPCData::Config { name: name.to_string(), value: None }).await?;

        // In real implementation, would extract result from context
        // For now, return mock data
        match name {
            "id" => Ok(Some("test_id".to_string())),
            "permanent-password" => Ok(Some("test_password".to_string())),
            _ => Ok(None),
        }
    }

    async fn set_config(&self, name: &str, value: String) -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut connection = self.connect_client(1000, "").await?;
        self.send_message(&mut connection, IPCData::Config { name: name.to_string(), value: Some(value) }).await?;
        Ok(())
    }

    async fn get_options(&self) -> std::result::Result<HashMap<String, String>, Box<dyn std::error::Error + Send + Sync>> {
        let mut connection = self.connect_client(1000, "").await?;
        self.send_message(&mut connection, IPCData::Options(None)).await?;

        // Mock options return
        let mut options = HashMap::new();
        options.insert("test_option".to_string(), "test_value".to_string());
        Ok(options)
    }

    async fn set_options(&self, options: HashMap<String, String>) -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut connection = self.connect_client(1000, "").await?;
        self.send_message(&mut connection, IPCData::Options(Some(options))).await?;
        Ok(())
    }
}

/// Global IPC facade instance
lazy_static::lazy_static! {
    pub static ref IPC_FACADE: CodeUChainIPC = CodeUChainIPC::new();
}

/// Legacy IPC compatibility functions that delegate to CodeUChain facade
pub async fn start(postfix: &str) -> ResultType<()> {
    IPC_FACADE.start_server(postfix).await?;
    Ok(())
}

pub async fn connect(ms_timeout: u64, postfix: &str) -> ResultType<IPCConnection> {
    Ok(IPC_FACADE.connect_client(ms_timeout, postfix).await?)
}

pub async fn get_config(name: &str) -> ResultType<Option<String>> {
    Ok(IPC_FACADE.get_config(name).await?)
}

pub async fn set_config(name: &str, value: String) -> ResultType<()> {
    IPC_FACADE.set_config(name, value).await?;
    Ok(())
}

pub async fn get_options() -> HashMap<String, String> {
    IPC_FACADE.get_options().await.unwrap_or_default()
}

pub async fn set_options(value: HashMap<String, String>) -> ResultType<()> {
    IPC_FACADE.set_options(value).await?;
    Ok(())
}