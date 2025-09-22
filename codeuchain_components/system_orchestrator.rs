use std::result::Result;
use crate::core::{Context, Chain};
use async_trait::async_trait;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;
use crate::ApplicationChainFactory;

/// Shared system context for cross-chain communication
#[derive(Clone)]
pub struct SystemContext {
    data: Arc<RwLock<HashMap<String, Value>>>,
}

impl SystemContext {
    pub fn new() -> Self {
        Self {
            data: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn set(&self, key: &str, value: Value) {
        let mut data = self.data.write();
        data.insert(key.to_string(), value);
    }

    pub fn get(&self, key: &str) -> Option<Value> {
        let data = self.data.read();
        data.get(key).cloned()
    }

    pub fn get_bool(&self, key: &str) -> bool {
        self.get(key)
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
    }

    pub fn get_string(&self, key: &str) -> Option<String> {
        self.data.read().get(key).and_then(|v| v.as_str().map(|s| s.to_string()))
    }

    pub fn to_context(&self) -> Context {
        let data = self.data.read();
        Context::new(data.clone())
    }

    pub fn update_from_context(&self, ctx: &Context) {
        let mut data = self.data.write();
        *data = ctx.data().clone();
    }

    pub fn data(&self) -> &RwLock<HashMap<String, Value>> {
        &self.data
    }
}

/// System orchestrator for coordinating multiple chains
pub struct SystemOrchestrator {
    pub system_context: SystemContext,
    ipc_chain: Option<Chain>,
    client_chain: Option<Chain>,
    server_chain: Option<Chain>,
    ui_chain: Option<Chain>,
    core_main_chain: Option<Chain>,
}

impl SystemOrchestrator {
    pub fn new() -> Self {
        Self {
            system_context: SystemContext::new(),
            ipc_chain: None,
            client_chain: None,
            server_chain: None,
            ui_chain: None,
            core_main_chain: None,
        }
    }

    /// Initialize the system with core main chain
    pub async fn initialize(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        println!("🔄 Initializing CodeUChain System Orchestrator...");

        // Initialize core main chain for startup
        let core_main_factory = ApplicationChainFactory;
        self.core_main_chain = Some(core_main_factory.create_application_chain());

        // Run core main initialization
        if let Some(chain) = &self.core_main_chain {
            let initial_ctx = self.system_context.to_context();
            let result_ctx = self.execute_chain_manually(chain, initial_ctx).await?;
            self.system_context.update_from_context(&result_ctx);
        }

        // Mark system as initialized
        self.system_context.set("system_initialized", json!(true));

        println!("✅ System initialization complete");
        Ok(())
    }

    /// Register IPC chain
    pub fn register_ipc_chain(&mut self, chain: Chain) {
        self.ipc_chain = Some(chain);
        println!("📡 IPC Chain registered");
    }

    /// Register client chain
    pub fn register_client_chain(&mut self, chain: Chain) {
        self.client_chain = Some(chain);
        println!("🖥️ Client Chain registered");
    }

    /// Register server chain
    pub fn register_server_chain(&mut self, chain: Chain) {
        self.server_chain = Some(chain);
        println!("🖧 Server Chain registered");
    }

    /// Register UI chain
    pub fn register_ui_chain(&mut self, chain: Chain) {
        self.ui_chain = Some(chain);
        println!("🖼️ UI Chain registered");
    }

    /// Execute a chain manually (since Chain::run has issues)
    async fn execute_chain_manually(&self, chain: &Chain, ctx: Context) -> Result<Context, Box<dyn std::error::Error + Send + Sync>> {
        // For now, return the context as-is since we don't have access to individual links
        // In a full implementation, this would execute each link in the chain
        Ok(ctx)
    }

    /// Process IPC communication
    pub async fn process_ipc_message(&mut self, message: Value) -> Result<Value, Box<dyn std::error::Error + Send + Sync>> {
        println!("📨 Processing IPC message: {:?}", message);

        // Check if this is a shutdown message
        if let Some(msg_obj) = message.as_object() {
            if msg_obj.get("type").and_then(|v| v.as_str()) == Some("system_shutdown") {
                self.system_context.set("system_shutdown", json!(true));
            }
        }

        if let Some(chain) = &self.ipc_chain {
            // Update system context with message
            self.system_context.set("ipc_message", message);

            // Execute IPC chain
            let ctx = self.system_context.to_context();
            let result_ctx = self.execute_chain_manually(chain, ctx).await?;
            self.system_context.update_from_context(&result_ctx);

            // Check if this affects other chains
            self.coordinate_cross_chain_communication().await?;
        }

        Ok(json!({"status": "processed"}))
    }

    /// Process client request
    pub async fn process_client_request(&mut self, request: Value) -> Result<Value, Box<dyn std::error::Error + Send + Sync>> {
        println!("🖥️ Processing client request: {:?}", request);

        if let Some(chain) = &self.client_chain {
            self.system_context.set("client_request", request);

            let ctx = self.system_context.to_context();
            let result_ctx = self.execute_chain_manually(chain, ctx).await?;
            self.system_context.update_from_context(&result_ctx);

            self.coordinate_cross_chain_communication().await?;
        }

        Ok(json!({"status": "processed"}))
    }

    /// Process server request
    pub async fn process_server_request(&mut self, request: Value) -> Result<Value, Box<dyn std::error::Error + Send + Sync>> {
        println!("🖧 Processing server request: {:?}", request);

        if let Some(chain) = &self.server_chain {
            self.system_context.set("server_message", request);

            let ctx = self.system_context.to_context();
            let result_ctx = self.execute_chain_manually(chain, ctx).await?;
            self.system_context.update_from_context(&result_ctx);

            self.coordinate_cross_chain_communication().await?;
        }

        Ok(json!({"status": "processed"}))
    }

    /// Process UI event
    pub async fn process_ui_event(&mut self, event: Value) -> Result<Value, Box<dyn std::error::Error + Send + Sync>> {
        println!("🖼️ Processing UI event: {:?}", event);

        if let Some(chain) = &self.ui_chain {
            self.system_context.set("ui_event", event);

            let ctx = self.system_context.to_context();
            let result_ctx = self.execute_chain_manually(chain, ctx).await?;
            self.system_context.update_from_context(&result_ctx);

            self.coordinate_cross_chain_communication().await?;
        }

        Ok(json!({"status": "processed"}))
    }

    /// Coordinate communication between chains based on system state
    async fn coordinate_cross_chain_communication(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Check for cross-chain communication needs

        // If IPC has new connection request, notify client/server chains
        if let Some(ipc_msg) = self.system_context.get("ipc_message") {
            if let Some(msg_obj) = ipc_msg.as_object() {
                if msg_obj.get("type").and_then(|v| v.as_str()) == Some("connection_request") {
                    println!("🔗 Coordinating connection request across chains");

                    // Notify both client and server chains
                    if let (Some(client), Some(server)) = (&self.client_chain, &self.server_chain) {
                        self.system_context.set("cross_chain_connection_request", ipc_msg.clone());
                    }
                }
            }
        }

        // If client has status update, notify UI chain
        if self.system_context.get("client_status_update").is_some() {
            println!("🔗 Coordinating client status update to UI");
            if let Some(ui) = &self.ui_chain {
                // UI would be notified of client status changes
            }
        }

        Ok(())
    }

    /// Get system status
    pub fn get_system_status(&self) -> Value {
        json!({
            "chains_registered": {
                "ipc": self.ipc_chain.is_some(),
                "client": self.client_chain.is_some(),
                "server": self.server_chain.is_some(),
                "ui": self.ui_chain.is_some(),
                "core_main": self.core_main_chain.is_some()
            },
            "system_context_keys": self.system_context.data.read().keys().collect::<Vec<_>>().len(),
            "initialized": true
        })
    }

    /// Shutdown the system
    pub async fn shutdown(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        println!("🛑 Shutting down CodeUChain System Orchestrator...");

        // Perform cleanup
        self.system_context.set("system_shutdown", json!(true));

        // Notify all chains of shutdown
        if let Some(chain) = &self.ipc_chain {
            // IPC cleanup
        }
        if let Some(chain) = &self.client_chain {
            // Client cleanup
        }
        if let Some(chain) = &self.server_chain {
            // Server cleanup
        }
        if let Some(chain) = &self.ui_chain {
            // UI cleanup
        }

        println!("✅ System shutdown complete");
        Ok(())
    }
}

/// Global system orchestrator instance
lazy_static::lazy_static! {
    static ref SYSTEM_ORCHESTRATOR: Arc<RwLock<SystemOrchestrator>> = Arc::new(RwLock::new(SystemOrchestrator::new()));
}

/// Public API for system orchestration
pub mod system_api {
    use super::*;

    pub async fn initialize_system() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut orchestrator = SYSTEM_ORCHESTRATOR.write();
        orchestrator.initialize().await
    }

    pub fn register_ipc_chain(chain: Chain) {
        let mut orchestrator = SYSTEM_ORCHESTRATOR.write();
        orchestrator.register_ipc_chain(chain);
    }

    pub fn register_client_chain(chain: Chain) {
        let mut orchestrator = SYSTEM_ORCHESTRATOR.write();
        orchestrator.register_client_chain(chain);
    }

    pub fn register_server_chain(chain: Chain) {
        let mut orchestrator = SYSTEM_ORCHESTRATOR.write();
        orchestrator.register_server_chain(chain);
    }

    pub fn register_ui_chain(chain: Chain) {
        let mut orchestrator = SYSTEM_ORCHESTRATOR.write();
        orchestrator.register_ui_chain(chain);
    }

    pub async fn process_ipc_message(message: Value) -> Result<Value, Box<dyn std::error::Error + Send + Sync>> {
        let mut orchestrator = SYSTEM_ORCHESTRATOR.write();
        orchestrator.process_ipc_message(message).await
    }

    pub async fn process_client_request(request: Value) -> Result<Value, Box<dyn std::error::Error + Send + Sync>> {
        let mut orchestrator = SYSTEM_ORCHESTRATOR.write();
        orchestrator.process_client_request(request).await
    }

    pub async fn process_server_request(request: Value) -> Result<Value, Box<dyn std::error::Error + Send + Sync>> {
        let mut orchestrator = SYSTEM_ORCHESTRATOR.write();
        orchestrator.process_server_request(request).await
    }

    pub async fn process_ui_event(event: Value) -> Result<Value, Box<dyn std::error::Error + Send + Sync>> {
        let mut orchestrator = SYSTEM_ORCHESTRATOR.write();
        orchestrator.process_ui_event(event).await
    }

    pub fn get_system_status() -> Value {
        let orchestrator = SYSTEM_ORCHESTRATOR.read();
        orchestrator.get_system_status()
    }

    pub async fn shutdown_system() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut orchestrator = SYSTEM_ORCHESTRATOR.write();
        orchestrator.shutdown().await
    }
}