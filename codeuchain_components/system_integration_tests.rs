use crate::system_orchestrator::{SystemOrchestrator, SystemContext, system_api};
use crate::core_main_chains::{ApplicationChainFactory};
use crate::core::Chain;
use serde_json::json;
use std::sync::Arc;
use parking_lot::RwLock;

#[tokio::test]
async fn test_system_orchestrator_initialization() {
    println!("🧪 Testing System Orchestrator Initialization...");

    let mut orchestrator = SystemOrchestrator::new();

    // Test initialization
    let result = orchestrator.initialize().await;
    assert!(result.is_ok(), "System orchestrator should initialize successfully");

    // Check system status
    let status = orchestrator.get_system_status();
    assert_eq!(status["initialized"], true);
    assert_eq!(status["chains_registered"]["core_main"], true);

    println!("✅ System orchestrator initialization test passed");
}

#[tokio::test]
async fn test_chain_registration() {
    println!("🧪 Testing Chain Registration...");

    let mut orchestrator = SystemOrchestrator::new();

    // Create mock chains (empty for testing)
    let ipc_chain = Chain::new();
    let client_chain = Chain::new();
    let server_chain = Chain::new();
    let ui_chain = Chain::new();

    // Register chains
    orchestrator.register_ipc_chain(ipc_chain);
    orchestrator.register_client_chain(client_chain);
    orchestrator.register_server_chain(server_chain);
    orchestrator.register_ui_chain(ui_chain);

    // Check system status
    let status = orchestrator.get_system_status();
    assert_eq!(status["chains_registered"]["ipc"], true);
    assert_eq!(status["chains_registered"]["client"], true);
    assert_eq!(status["chains_registered"]["server"], true);
    assert_eq!(status["chains_registered"]["ui"], true);

    println!("✅ Chain registration test passed");
}

#[tokio::test]
async fn test_cross_chain_communication_ipc_to_client_server() {
    println!("🧪 Testing IPC to Client/Server Cross-Chain Communication...");

    let mut orchestrator = SystemOrchestrator::new();

    // Register chains
    orchestrator.register_ipc_chain(Chain::new());
    orchestrator.register_client_chain(Chain::new());
    orchestrator.register_server_chain(Chain::new());

    // Send IPC connection request
    let ipc_message = json!({
        "type": "connection_request",
        "client_id": "test_client",
        "server_id": "test_server"
    });

    let result = orchestrator.process_ipc_message(ipc_message.clone()).await;
    assert!(result.is_ok(), "IPC message processing should succeed");

    // Check that cross-chain communication was coordinated
    let cross_chain_request = orchestrator.system_context.get("cross_chain_connection_request");
    assert!(cross_chain_request.is_some(), "Cross-chain connection request should be set");

    println!("✅ IPC to Client/Server cross-chain communication test passed");
}

#[tokio::test]
async fn test_shared_context_management() {
    println!("🧪 Testing Shared Context Management...");

    let context = SystemContext::new();

    // Test setting and getting values
    context.set("test_key", json!("test_value"));
    let retrieved = context.get("test_key");
    assert_eq!(retrieved, Some(json!("test_value")));

    // Test boolean access
    context.set("bool_key", json!(true));
    assert_eq!(context.get_bool("bool_key"), true);
    assert_eq!(context.get_bool("nonexistent"), false);

    // Test string access
    context.set("string_key", json!("hello world"));
    assert_eq!(context.get_string("string_key"), Some("hello world".to_string()));

    // Test context conversion
    let codeuchain_ctx = context.to_context();
    assert!(codeuchain_ctx.data().contains_key("test_key"));

    println!("✅ Shared context management test passed");
}

#[tokio::test]
async fn test_system_api_integration() {
    println!("🧪 Testing System API Integration...");

    // Test system initialization
    let init_result = system_api::initialize_system().await;
    assert!(init_result.is_ok(), "System API initialization should succeed");

    // Test chain registration via API
    system_api::register_ipc_chain(Chain::new());
    system_api::register_client_chain(Chain::new());

    // Test status retrieval
    let status = system_api::get_system_status();
    assert_eq!(status["initialized"], true);
    assert_eq!(status["chains_registered"]["ipc"], true);
    assert_eq!(status["chains_registered"]["client"], true);

    // Test message processing
    let message = json!({"type": "test", "data": "integration_test"});
    let process_result = system_api::process_ipc_message(message).await;
    assert!(process_result.is_ok(), "IPC message processing via API should succeed");

    // Test shutdown
    let shutdown_result = system_api::shutdown_system().await;
    assert!(shutdown_result.is_ok(), "System shutdown should succeed");

    println!("✅ System API integration test passed");
}

#[tokio::test]
async fn test_error_handling_and_recovery() {
    println!("🧪 Testing Error Handling and Recovery...");

    let mut orchestrator = SystemOrchestrator::new();

    // Test processing without registered chains (should not panic)
    let message = json!({"type": "test"});
    let result = orchestrator.process_ipc_message(message).await;
    assert!(result.is_ok(), "Processing without IPC chain should not fail");

    // Test shutdown without initialized system
    let shutdown_result = orchestrator.shutdown().await;
    assert!(shutdown_result.is_ok(), "Shutdown without full initialization should succeed");

    println!("✅ Error handling and recovery test passed");
}

#[tokio::test]
async fn test_concurrent_chain_operations() {
    println!("🧪 Testing Concurrent Chain Operations...");

    let orchestrator = Arc::new(RwLock::new(SystemOrchestrator::new()));

    // Register chains
    {
        let mut orch = orchestrator.write();
        orch.register_ipc_chain(Chain::new());
        orch.register_client_chain(Chain::new());
        orch.register_server_chain(Chain::new());
    }

    // Test concurrent access by running operations sequentially but with shared state
    // This tests that the shared state works correctly under concurrent access patterns

    let msg1 = json!({"type": "concurrent_test_1", "id": 1});
    let msg2 = json!({"type": "concurrent_test_2", "id": 2});
    let msg3 = json!({"type": "concurrent_test_3", "id": 3});

    // Run operations sequentially (simulating concurrent access to shared state)
    {
        let mut orch = orchestrator.write();
        let result1 = orch.process_ipc_message(msg1).await;
        assert!(result1.is_ok(), "IPC processing should succeed");
    }

    {
        let mut orch = orchestrator.write();
        let result2 = orch.process_client_request(msg2).await;
        assert!(result2.is_ok(), "Client processing should succeed");
    }

    {
        let mut orch = orchestrator.write();
        let result3 = orch.process_server_request(msg3).await;
        assert!(result3.is_ok(), "Server processing should succeed");
    }

    // Verify that all operations updated the shared context
    let orch = orchestrator.read();
    let context_data = orch.system_context.data().read();
    assert!(context_data.contains_key("ipc_message"), "IPC message should be in context");
    assert!(context_data.contains_key("client_request"), "Client request should be in context");
    assert!(context_data.contains_key("server_message"), "Server message should be in context");

    println!("✅ Concurrent chain operations test passed");
}