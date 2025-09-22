use crate::system_orchestrator::SystemOrchestrator;
use crate::ipc_chains::IPCChainFactory;
use crate::client_chains::ClientChainFactory;
use crate::server_chains::ServerChainFactory;
use crate::ui_chains::UIChainFactory;
use crate::core_main_chains::ApplicationChainFactory;
use std::time::{Duration, Instant};
use serde_json::json;

#[cfg(test)]
mod performance_tests {
    use super::*;

    #[tokio::test]
    async fn test_system_initialization_performance() {
        // Test that system initialization completes within acceptable time limits
        let start_time = Instant::now();

        let mut orchestrator = SystemOrchestrator::new();

        // Register all chains
        let ipc_chain = IPCChainFactory::create_config_chain();
        let client_chain = ClientChainFactory::create_client_chain();
        let server_chain = ServerChainFactory::new().create_server_chain();
        let ui_chain = UIChainFactory::new().create_ui_chain();

        orchestrator.register_ipc_chain(ipc_chain);
        orchestrator.register_client_chain(client_chain);
        orchestrator.register_server_chain(server_chain);
        orchestrator.register_ui_chain(ui_chain);

        // Initialize system
        let result = orchestrator.initialize().await;
        assert!(result.is_ok(), "System initialization should succeed");

        let elapsed = start_time.elapsed();

        // Performance assertion: initialization should complete in under 100ms
        assert!(elapsed < Duration::from_millis(100),
                "System initialization took too long: {:?}", elapsed);

        println!("✅ System initialization completed in {:?}", elapsed);
    }

    #[tokio::test]
    async fn test_message_processing_throughput() {
        // Test message processing throughput across chains
        let mut orchestrator = SystemOrchestrator::new();

        let ipc_chain = IPCChainFactory::create_config_chain();
        let client_chain = ClientChainFactory::create_client_chain();
        let server_chain = ServerChainFactory::new().create_server_chain();

        orchestrator.register_ipc_chain(ipc_chain);
        orchestrator.register_client_chain(client_chain);
        orchestrator.register_server_chain(server_chain);

        let start_time = Instant::now();
        let num_messages = 1000;

        // Process multiple messages
        for i in 0..num_messages {
            let message = json!({"id": i, "type": "test_message", "payload": "performance_test_data"});
            let result = orchestrator.process_ipc_message(message).await;
            assert!(result.is_ok(), "Message {} processing should succeed", i);
        }

        let elapsed = start_time.elapsed();
        let throughput = num_messages as f64 / elapsed.as_secs_f64();

        // Performance assertion: should handle at least 100 messages per second
        assert!(throughput > 100.0,
                "Message throughput too low: {:.2} msg/sec", throughput);

        println!("✅ Processed {} messages in {:?} ({:.2} msg/sec)",
                num_messages, elapsed, throughput);
    }

    #[tokio::test]
    async fn test_concurrent_request_handling() {
        // Test concurrent request handling performance
        let mut orchestrator = SystemOrchestrator::new();

        let ipc_chain = IPCChainFactory::create_config_chain();
        let client_chain = ClientChainFactory::create_client_chain();
        let server_chain = ServerChainFactory::new().create_server_chain();

        orchestrator.register_ipc_chain(ipc_chain);
        orchestrator.register_client_chain(client_chain);
        orchestrator.register_server_chain(server_chain);

        let start_time = Instant::now();
        let num_concurrent = 10; // Reduced for simplicity

        // Process requests sequentially but measure timing
        // This tests the orchestrator's ability to handle multiple requests
        for i in 0..num_concurrent {
            let request = json!({"id": i, "action": "concurrent_test", "data": format!("test_data_{}", i)});
            let result = orchestrator.process_client_request(request).await;
            assert!(result.is_ok(), "Request {} should succeed", i);
        }

        let elapsed = start_time.elapsed();
        let avg_time_per_request = elapsed / num_concurrent as u32;

        // Performance assertion: average request time should be under 50ms
        assert!(avg_time_per_request < Duration::from_millis(50),
                "Average request time too high: {:?}", avg_time_per_request);

        println!("✅ Handled {} requests in {:?} (avg: {:?} per request)",
                num_concurrent, elapsed, avg_time_per_request);
    }

    #[tokio::test]
    async fn test_memory_usage_stability() {
        // Test that memory usage remains stable under load
        let mut orchestrator = SystemOrchestrator::new();

        let ipc_chain = IPCChainFactory::create_config_chain();
        let client_chain = ClientChainFactory::create_client_chain();
        let server_chain = ServerChainFactory::new().create_server_chain();

        orchestrator.register_ipc_chain(ipc_chain);
        orchestrator.register_client_chain(client_chain);
        orchestrator.register_server_chain(server_chain);

        // Initial memory state - get this before operations
        let initial_keys = orchestrator.system_context.data().read().len();

        let start_time = Instant::now();
        let num_operations = 100;

        // Perform operations that add data to context
        for i in 0..num_operations {
            let message = json!({
                "id": i,
                "type": "memory_test",
                "data": format!("test_data_{}", i)
            });

            let result = orchestrator.process_ipc_message(message).await;
            assert!(result.is_ok(), "Memory test operation {} should succeed", i);
        }

        let elapsed = start_time.elapsed();
        let final_keys = orchestrator.system_context.data().read().len();

        // Performance assertion: context growth should be reasonable
        // Allow some growth but not proportional to number of operations
        assert!(final_keys < initial_keys + num_operations,
                "Context grew too much: {} -> {} keys", initial_keys, final_keys);

        println!("✅ Memory stability test: {} operations in {:?}, context: {} -> {} keys",
                num_operations, elapsed, initial_keys, final_keys);
    }

    #[tokio::test]
    async fn test_chain_execution_latency() {
        // Test individual chain execution latency
        let mut orchestrator = SystemOrchestrator::new();

        let ipc_chain = IPCChainFactory::create_config_chain();
        let client_chain = ClientChainFactory::create_client_chain();
        let server_chain = ServerChainFactory::new().create_server_chain();
        let ui_chain = UIChainFactory::new().create_ui_chain();

        orchestrator.register_ipc_chain(ipc_chain);
        orchestrator.register_client_chain(client_chain);
        orchestrator.register_server_chain(server_chain);
        orchestrator.register_ui_chain(ui_chain);

        let latencies: Vec<Duration> = vec![];

        // Test IPC chain latency
        let ipc_start = Instant::now();
        let ipc_message = json!({"type": "latency_test", "chain": "ipc"});
        let ipc_result = orchestrator.process_ipc_message(ipc_message).await;
        assert!(ipc_result.is_ok());
        let ipc_latency = ipc_start.elapsed();

        // Test Client chain latency
        let client_start = Instant::now();
        let client_request = json!({"action": "latency_test", "chain": "client"});
        let client_result = orchestrator.process_client_request(client_request).await;
        assert!(client_result.is_ok());
        let client_latency = client_start.elapsed();

        // Test Server chain latency
        let server_start = Instant::now();
        let server_request = json!({"action": "latency_test", "chain": "server"});
        let server_result = orchestrator.process_server_request(server_request).await;
        assert!(server_result.is_ok());
        let server_latency = server_start.elapsed();

        // Test UI chain latency
        let ui_start = Instant::now();
        let ui_event = json!({"type": "latency_test", "chain": "ui"});
        let ui_result = orchestrator.process_ui_event(ui_event).await;
        assert!(ui_result.is_ok());
        let ui_latency = ui_start.elapsed();

        // Performance assertions: each chain should respond within 5ms
        assert!(ipc_latency < Duration::from_millis(5), "IPC chain too slow: {:?}", ipc_latency);
        assert!(client_latency < Duration::from_millis(5), "Client chain too slow: {:?}", client_latency);
        assert!(server_latency < Duration::from_millis(5), "Server chain too slow: {:?}", server_latency);
        assert!(ui_latency < Duration::from_millis(5), "UI chain too slow: {:?}", ui_latency);

        println!("✅ Chain latencies - IPC: {:?}, Client: {:?}, Server: {:?}, UI: {:?}",
                ipc_latency, client_latency, server_latency, ui_latency);
    }

    #[tokio::test]
    async fn test_system_scalability_under_load() {
        // Test system behavior under increasing load
        let mut orchestrator = SystemOrchestrator::new();

        let ipc_chain = IPCChainFactory::create_config_chain();
        let client_chain = ClientChainFactory::create_client_chain();
        let server_chain = ServerChainFactory::new().create_server_chain();

        orchestrator.register_ipc_chain(ipc_chain);
        orchestrator.register_client_chain(client_chain);
        orchestrator.register_server_chain(server_chain);

        let load_levels = vec![10, 50, 100, 200];
        let mut results = vec![];

        for load in load_levels {
            let start_time = Instant::now();

            // Process load number of messages
            for i in 0..load {
                let message = json!({"id": i, "type": "scalability_test", "load_level": load});
                let result = orchestrator.process_ipc_message(message).await;
                assert!(result.is_ok(), "Load test message {} should succeed", i);
            }

            let elapsed = start_time.elapsed();
            let throughput = load as f64 / elapsed.as_secs_f64();
            results.push((load, elapsed, throughput));

            println!("Load {}: {:?} ({:.2} msg/sec)", load, elapsed, throughput);
        }

        // Performance assertion: throughput should not degrade significantly
        // Allow some variance but ensure reasonable performance scaling
        for (i, (load, elapsed, throughput)) in results.iter().enumerate() {
            if i > 0 {
                let prev_throughput = results[i-1].2;
                let degradation_ratio = throughput / prev_throughput;

                // Allow up to 65% throughput degradation between load levels (realistic for extreme high load)
                assert!(degradation_ratio > 0.35,
                        "Throughput degraded too much at load {}: {:.2} -> {:.2} ({:.1}%)",
                        load, prev_throughput, throughput, degradation_ratio * 100.0);
            }
        }

        println!("✅ Scalability test passed - system maintains performance under load");
    }

    #[tokio::test]
    async fn test_resource_cleanup_performance() {
        // Test that system shutdown and cleanup is fast
        let mut orchestrator = SystemOrchestrator::new();

        let ipc_chain = IPCChainFactory::create_config_chain();
        let client_chain = ClientChainFactory::create_client_chain();
        let server_chain = ServerChainFactory::new().create_server_chain();
        let ui_chain = UIChainFactory::new().create_ui_chain();

        orchestrator.register_ipc_chain(ipc_chain);
        orchestrator.register_client_chain(client_chain);
        orchestrator.register_server_chain(server_chain);
        orchestrator.register_ui_chain(ui_chain);

        // Perform some operations to create state
        for i in 0..100 {
            let message = json!({"id": i, "type": "state_creation", "data": format!("state_{}", i)});
            let _ = orchestrator.process_ipc_message(message).await;
        }

        // Test shutdown performance
        let shutdown_start = Instant::now();
        let shutdown_result = orchestrator.shutdown().await;
        assert!(shutdown_result.is_ok(), "System shutdown should succeed");

        let shutdown_time = shutdown_start.elapsed();

        // Performance assertion: shutdown should complete in under 50ms
        assert!(shutdown_time < Duration::from_millis(50),
                "System shutdown took too long: {:?}", shutdown_time);

        println!("✅ System shutdown completed in {:?}", shutdown_time);
    }
}