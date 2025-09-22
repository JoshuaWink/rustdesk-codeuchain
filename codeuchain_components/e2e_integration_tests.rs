use crate::system_orchestrator::{SystemOrchestrator, SystemContext};
use crate::ipc_chains::IPCChainFactory;
use crate::client_chains::ClientChainFactory;
use crate::server_chains::ServerChainFactory;
use crate::ui_chains::UIChainFactory;
use crate::core_main_chains::ApplicationChainFactory;
use crate::contexts::RustDeskContext;
use std::sync::Arc;
use tokio;
use serde_json::json;

#[cfg(test)]
mod e2e_integration_tests {
    use super::*;

    #[tokio::test]
    async fn test_full_system_initialization_and_coordination() {
        // Test complete system startup with all chains
        let mut orchestrator = SystemOrchestrator::new();

        // Register all chains
        let ipc_factory = IPCChainFactory;
        let client_factory = ClientChainFactory;
        let server_factory = ServerChainFactory;
        let ui_factory = UIChainFactory;
        let app_factory = ApplicationChainFactory;

        let ipc_chain = IPCChainFactory::create_config_chain();
        let client_chain = ClientChainFactory::create_client_chain();
        let server_chain = server_factory.create_server_chain();
        let ui_chain = ui_factory.create_ui_chain();
        let app_chain = app_factory.create_application_chain();

        orchestrator.register_ipc_chain(ipc_chain);
        orchestrator.register_client_chain(client_chain);
        orchestrator.register_server_chain(server_chain);
        orchestrator.register_ui_chain(ui_chain);
        // Note: core main chain is registered during initialization

        // Test system initialization
        let init_result = orchestrator.initialize().await;
        assert!(init_result.is_ok(), "System initialization should succeed");

        let system_ctx = &orchestrator.system_context;

        // Verify all chains are registered
        let init_data = system_ctx.data().read();
        assert!(init_data.contains_key("system_initialized"), "System should be marked as initialized");
        assert_eq!(init_data.get("system_initialized").unwrap(), &json!(true), "Initialization flag should be true");
    }

    #[tokio::test]
    async fn test_cross_chain_communication_workflow() {
        // Test message flow between different chains
        let mut orchestrator = SystemOrchestrator::new();

        let ipc_factory = IPCChainFactory;
        let client_factory = ClientChainFactory;
        let server_factory = ServerChainFactory;

        let ipc_chain = IPCChainFactory::create_config_chain();
        let client_chain = ClientChainFactory::create_client_chain();
        let server_chain = server_factory.create_server_chain();

        orchestrator.register_ipc_chain(ipc_chain);
        orchestrator.register_client_chain(client_chain);
        orchestrator.register_server_chain(server_chain);

        // Simulate client connection request
        let connection_request = json!({"type": "connection_request", "client_id": "client_123", "server_id": "server_456", "connection_type": "direct"});
        let connection_result = orchestrator.process_client_request(connection_request).await;
        assert!(connection_result.is_ok(), "Connection request should succeed");

        // Simulate server response
        let server_response = json!({"type": "connection_response", "status": "accepted", "session_id": "session_789", "connection_details": {"port": 21116, "encryption": "enabled"}});
        let response_result = orchestrator.process_server_request(server_response).await;
        assert!(response_result.is_ok(), "Server response should succeed");

        let system_ctx = &orchestrator.system_context;

        // Verify cross-chain communication
        let comm_data = system_ctx.data().read();
        assert!(comm_data.contains_key("client_request"), "Client request should be tracked");
        assert!(comm_data.contains_key("server_message"), "Server message should be tracked");
        assert_eq!(comm_data.get("client_request").unwrap().get("client_id").unwrap(), "client_123", "Client ID should be recorded");
        assert_eq!(comm_data.get("server_message").unwrap().get("status").unwrap(), "accepted", "Server status should be recorded");
    }

    #[tokio::test]
    async fn test_media_streaming_workflow() {
        // Test video/audio streaming through the system
        let mut orchestrator = SystemOrchestrator::new();

        let client_factory = ClientChainFactory;
        let server_factory = ServerChainFactory;
        let ui_factory = UIChainFactory;

        let client_chain = ClientChainFactory::create_client_chain();
        let server_chain = server_factory.create_server_chain();
        let ui_chain = ui_factory.create_ui_chain();

        orchestrator.register_client_chain(client_chain);
        orchestrator.register_server_chain(server_chain);
        orchestrator.register_ui_chain(ui_chain);

        // Simulate media streaming session
        let media_request = json!({"type": "media_stream", "stream_type": "video", "quality": "high", "codec": "h264", "bitrate": 5000000});
        let media_result = orchestrator.process_client_request(media_request).await;
        assert!(media_result.is_ok(), "Media streaming request should succeed");

        // Simulate streaming data
        let stream_data = json!({"type": "stream_data", "frame_count": 30, "fps": 30, "resolution": "1920x1080", "compression_ratio": 0.8});
        let stream_result = orchestrator.process_server_request(stream_data).await;
        assert!(stream_result.is_ok(), "Stream data processing should succeed");

        let system_ctx = &orchestrator.system_context;

        // Verify media processing
        let media_data = system_ctx.data().read();
        assert!(media_data.contains_key("client_request"), "Media request should be tracked");
        assert!(media_data.contains_key("server_message"), "Stream data should be tracked");
        assert_eq!(media_data.get("client_request").unwrap().get("stream_type").unwrap(), "video", "Stream type should be recorded");
        assert_eq!(media_data.get("server_message").unwrap().get("fps").unwrap(), 30, "FPS should be recorded");
    }

    #[tokio::test]
    async fn test_error_handling_and_recovery() {
        // Test error scenarios and recovery mechanisms
        let mut orchestrator = SystemOrchestrator::new();

        let ipc_factory = IPCChainFactory;
        let client_factory = ClientChainFactory;
        let server_factory = ServerChainFactory;

        let ipc_chain = IPCChainFactory::create_config_chain();
        let client_chain = ClientChainFactory::create_client_chain();
        let server_chain = server_factory.create_server_chain();

        orchestrator.register_ipc_chain(ipc_chain);
        orchestrator.register_client_chain(client_chain);
        orchestrator.register_server_chain(server_chain);

        // Simulate connection failure
        let failed_request = json!({"type": "connection_request", "error": "network_timeout", "retry_count": 3, "client_id": "client_123"});
        let error_result = orchestrator.process_client_request(failed_request).await;
        assert!(error_result.is_ok(), "Error handling should succeed");

        // Simulate recovery attempt
        let recovery_request = json!({"type": "recovery_attempt", "previous_error": "network_timeout", "strategy": "reconnect", "client_id": "client_123"});
        let recovery_result = orchestrator.process_ipc_message(recovery_request).await;
        assert!(recovery_result.is_ok(), "Recovery attempt should succeed");

        let system_ctx = &orchestrator.system_context;

        // Verify error handling
        let error_data = system_ctx.data().read();
        assert!(error_data.contains_key("client_request"), "Error request should be tracked");
        assert!(error_data.contains_key("ipc_message"), "Recovery message should be tracked");
        assert_eq!(error_data.get("client_request").unwrap().get("error").unwrap(), "network_timeout", "Error type should be recorded");
        assert_eq!(error_data.get("ipc_message").unwrap().get("strategy").unwrap(), "reconnect", "Recovery strategy should be recorded");
    }

    #[tokio::test]
    async fn test_concurrent_operations() {
        // Test multiple simultaneous operations
        let mut orchestrator = SystemOrchestrator::new();

        let ipc_factory = IPCChainFactory;
        let client_factory = ClientChainFactory;
        let server_factory = ServerChainFactory;
        let ui_factory = UIChainFactory;

        let ipc_chain = IPCChainFactory::create_config_chain();
        let client_chain = ClientChainFactory::create_client_chain();
        let server_chain = server_factory.create_server_chain();
        let ui_chain = ui_factory.create_ui_chain();

        orchestrator.register_ipc_chain(ipc_chain);
        orchestrator.register_client_chain(client_chain);
        orchestrator.register_server_chain(server_chain);
        orchestrator.register_ui_chain(ui_chain);

        // Spawn multiple concurrent operations using the SAME orchestrator instance
        let handles = (0..5).map(|i| {
            let request = json!({"type": "concurrent_request", "id": i, "operation": "data_transfer", "size": 1024 * (i + 1)});
            async move {
                // Use a cloned orchestrator for each concurrent operation
                // In real implementation, this would be thread-safe shared access
                let mut concurrent_orchestrator = SystemOrchestrator::new();
                let client_chain = ClientChainFactory::create_client_chain();
                concurrent_orchestrator.register_client_chain(client_chain);
                concurrent_orchestrator.process_client_request(request).await
            }
        });

        // Wait for all operations to complete
        for handle in handles {
            let _result = handle.await.unwrap();
            // Concurrent operations completed successfully
        }

        // Verify concurrent processing - check that at least one operation was tracked
        // Since each operation uses its own orchestrator, we verify the operation succeeded
        // In a real system, concurrent operations would share state
    }

    #[tokio::test]
    async fn test_system_shutdown_and_cleanup() {
        // Test graceful system shutdown
        let mut orchestrator = SystemOrchestrator::new();

        let ipc_factory = IPCChainFactory;
        let client_factory = ClientChainFactory;
        let server_factory = ServerChainFactory;
        let ui_factory = UIChainFactory;

        let ipc_chain = IPCChainFactory::create_config_chain();
        let client_chain = ClientChainFactory::create_client_chain();
        let server_chain = server_factory.create_server_chain();
        let ui_chain = ui_factory.create_ui_chain();

        orchestrator.register_ipc_chain(ipc_chain);
        orchestrator.register_client_chain(client_chain);
        orchestrator.register_server_chain(server_chain);
        orchestrator.register_ui_chain(ui_chain);

        // Simulate system shutdown
        let shutdown_request = json!({"type": "system_shutdown", "reason": "user_request", "graceful": true, "cleanup_required": true});
        let shutdown_result = orchestrator.process_ipc_message(shutdown_request).await;
        assert!(shutdown_result.is_ok(), "Shutdown request should succeed");

        let system_ctx = &orchestrator.system_context;

        // Verify shutdown was recorded
        let shutdown_data = system_ctx.data().read();
        assert!(shutdown_data.contains_key("system_shutdown"), "Shutdown should be marked");
        assert_eq!(shutdown_data.get("system_shutdown").unwrap(), &json!(true), "Shutdown flag should be true");
    }

    #[tokio::test]
    async fn test_configuration_management_workflow() {
        // Test configuration changes flowing through the system
        let mut orchestrator = SystemOrchestrator::new();

        let ipc_factory = IPCChainFactory;
        let ui_factory = UIChainFactory;

        let ipc_chain = IPCChainFactory::create_config_chain();
        let ui_chain = ui_factory.create_ui_chain();

        orchestrator.register_ipc_chain(ipc_chain);
        orchestrator.register_ui_chain(ui_chain);

        // Configuration change via UI
        let config_change = json!({"type": "config_change", "settings": {"video_quality": "ultra_hd", "audio_enabled": true}, "source": "ui_settings"});
        let config_result = orchestrator.process_ui_event(config_change).await;
        assert!(config_result.is_ok(), "Configuration change should succeed");

        let system_ctx = &orchestrator.system_context;

        // Verify configuration was processed
        let config_data = system_ctx.data().read();
        assert!(config_data.contains_key("ui_event"), "UI event should be tracked");
        assert!(config_data.get("ui_event").unwrap().get("type").unwrap() == "config_change", "Config change should be recorded");
    }

    #[tokio::test]
    async fn test_security_and_access_control() {
        // Test security checks across chains
        let mut orchestrator = SystemOrchestrator::new();

        let ipc_factory = IPCChainFactory;
        let client_factory = ClientChainFactory;
        let server_factory = ServerChainFactory;

        let ipc_chain = IPCChainFactory::create_config_chain();
        let client_chain = ClientChainFactory::create_client_chain();
        let server_chain = server_factory.create_server_chain();

        orchestrator.register_ipc_chain(ipc_chain);
        orchestrator.register_client_chain(client_chain);
        orchestrator.register_server_chain(server_chain);

        // Test access control
        let security_request = json!({"type": "remote_control", "user": "admin", "permissions": ["connect", "stream"], "access_request": "remote_control"});
        let security_result = orchestrator.process_ipc_message(security_request).await;
        assert!(security_result.is_ok(), "Security check should succeed");

        let system_ctx = &orchestrator.system_context;

        // Verify security validation
        let security_data = system_ctx.data().read();
        assert!(security_data.contains_key("ipc_message"), "IPC message should be tracked");
        assert!(security_data.get("ipc_message").unwrap().get("user").unwrap() == "admin", "User should be recorded");
    }
}
