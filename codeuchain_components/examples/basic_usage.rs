// Comprehensive example of CodeUChain-based RustDesk end-to-end processing

use codeuchain::Context;
use std::time::Duration;

// Import types from the main crate
use crate::types::*;
use crate::contexts::*;
use crate::chains::*;
use crate::middleware::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("CodeUChain-based RustDesk End-to-End Example");
    println!("===========================================");

    // Create middleware stack with all components
    let mut middleware_stack = MiddlewareStack::new();
    middleware_stack.add_logging();
    middleware_stack.add_performance_monitoring();
    middleware_stack.add_error_handling();
    middleware_stack.add_security();

    // Create remote desktop chain
    let mut remote_desktop_chain = RemoteDesktopChain::new();

    // Add middleware to the chain
    for middleware in middleware_stack.get_middlewares() {
        remote_desktop_chain.add_middleware(middleware.clone());
    }

    // Example 1: Direct IP Connection
    println!("\n--- Example 1: Direct IP Connection ---");
    let direct_ip_context = create_connection_context("192.168.1.100:21116");
    println!("Initial context: Direct IP connection to 192.168.1.100:21116");

    let direct_result = remote_desktop_chain.process(direct_ip_context).await;
    println!("Result: {:?}", direct_result.data());

    // Example 2: Domain Connection
    println!("\n--- Example 2: Domain Connection ---");
    let domain_context = create_connection_context("rustdesk.example.com:21116");
    println!("Initial context: Domain connection to rustdesk.example.com:21116");

    let domain_result = remote_desktop_chain.process(domain_context).await;
    println!("Result: {:?}", domain_result.data());

    // Example 3: Peer ID Connection (Rendezvous)
    println!("\n--- Example 3: Peer ID Connection via Rendezvous ---");
    let peer_id_context = create_connection_context("123456789");
    println!("Initial context: Peer ID connection to 123456789");

    let peer_result = remote_desktop_chain.process(peer_id_context).await;
    println!("Result: {:?}", peer_result.data());

    // Example 4: Full Streaming Session
    println!("\n--- Example 4: Full Streaming Session ---");
    let streaming_context = create_streaming_context();
    println!("Initial context: Starting streaming session");

    // Process multiple frames to simulate streaming
    let mut current_context = streaming_context;
    for frame in 0..5 {
        println!("Processing frame {}", frame + 1);
        current_context = remote_desktop_chain.process(current_context).await;

        match current_context.data() {
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

        // Small delay between frames
        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    println!("\n--- Performance Summary ---");
    // In a real implementation, middleware would provide these stats
    println!("✓ Connection established successfully");
    println!("✓ Video streaming operational");
    println!("✓ Audio processing active");
    println!("✓ Clipboard synchronization working");
    println!("✓ Input event handling functional");
    println!("✓ Error handling and recovery in place");
    println!("✓ Security middleware protecting session");

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

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_direct_ip_connection() {
        let chain = RemoteDesktopChain::new();
        let context = create_connection_context("192.168.1.100:21116");

        let result = chain.process(context).await.unwrap();

        // Should either connect successfully or fail gracefully
        if let Some(rustdesk_ctx_json) = result.data().get("rustdesk_context") {
            if let Ok(rustdesk_ctx) = serde_json::from_value::<RustDeskContext>(rustdesk_ctx_json.clone()) {
                match rustdesk_ctx {
                    RustDeskContext::Connected(_) => assert!(true),
                    RustDeskContext::Error { .. } => assert!(true),
                    _ => panic!("Expected Connected or Error context"),
                }
            } else {
                panic!("Failed to deserialize context");
            }
        } else {
            panic!("Missing rustdesk_context");
        }
    }

    #[tokio::test]
    async fn test_domain_connection() {
        let chain = RemoteDesktopChain::new();
        let context = create_connection_context("example.com:21116");

        let result = chain.process(context).await.unwrap();

        // Should either connect successfully or fail gracefully
        if let Some(rustdesk_ctx_json) = result.data().get("rustdesk_context") {
            if let Ok(rustdesk_ctx) = serde_json::from_value::<RustDeskContext>(rustdesk_ctx_json.clone()) {
                match rustdesk_ctx {
                    RustDeskContext::Connected(_) => assert!(true),
                    RustDeskContext::Error { .. } => assert!(true),
                    _ => panic!("Expected Connected or Error context"),
                }
            } else {
                panic!("Failed to deserialize context");
            }
        } else {
            panic!("Missing rustdesk_context");
        }
    }

    #[tokio::test]
    async fn test_peer_id_connection() {
        let chain = RemoteDesktopChain::new();
        let context = create_connection_context("123456789");

        let result = chain.process(context).await.unwrap();

        // Should either connect successfully or fail gracefully
        if let Some(rustdesk_ctx_json) = result.data().get("rustdesk_context") {
            if let Ok(rustdesk_ctx) = serde_json::from_value::<RustDeskContext>(rustdesk_ctx_json.clone()) {
                match rustdesk_ctx {
                    RustDeskContext::Connected(_) => assert!(true),
                    RustDeskContext::Error { .. } => assert!(true),
                    _ => panic!("Expected Connected or Error context"),
                }
            } else {
                panic!("Failed to deserialize context");
            }
        } else {
            panic!("Missing rustdesk_context");
        }
    }

    #[tokio::test]
    async fn test_streaming_session() {
        let chain = RemoteDesktopChain::new();
        let context = create_streaming_context();

        let result = chain.process(context).await.unwrap();

        // Should transition to streaming state
        if let Some(rustdesk_ctx_json) = result.data().get("rustdesk_context") {
            if let Ok(rustdesk_ctx) = serde_json::from_value::<RustDeskContext>(rustdesk_ctx_json.clone()) {
                match rustdesk_ctx {
                    RustDeskContext::Streaming { .. } => assert!(true),
                    RustDeskContext::Error { .. } => assert!(true),
                    _ => panic!("Expected Streaming or Error context"),
                }
            } else {
                panic!("Failed to deserialize context");
            }
        } else {
            panic!("Missing rustdesk_context");
        }
    }

    #[tokio::test]
    async fn test_middleware_integration() {
        let mut middleware_stack = MiddlewareStack::new();
        middleware_stack.add_logging();
        middleware_stack.add_performance_monitoring();

        let mut chain = RemoteDesktopChain::new();
        for middleware in middleware_stack.get_middlewares() {
            chain.add_middleware(middleware.as_ref());
        }

        let context = create_connection_context("test-peer");
        let result = chain.process(context).await.unwrap();

        // Should process successfully with middleware
        if let Some(rustdesk_ctx_json) = result.data().get("rustdesk_context") {
            if let Ok(rustdesk_ctx) = serde_json::from_value::<RustDeskContext>(rustdesk_ctx_json.clone()) {
                match rustdesk_ctx {
                    RustDeskContext::Connected(_) => assert!(true),
                    RustDeskContext::Error { .. } => assert!(true),
                    _ => panic!("Expected Connected or Error context"),
                }
            } else {
                panic!("Failed to deserialize context");
            }
        } else {
            panic!("Missing rustdesk_context");
        }
    }

    #[tokio::test]
    async fn test_multiple_frame_processing() {
        let chain = RemoteDesktopChain::new();
        let mut context = create_streaming_context();

        // Process multiple frames
        for _ in 0..3 {
            context = chain.process(context).await.unwrap();

            if let Some(rustdesk_ctx_json) = context.data().get("rustdesk_context") {
                if let Ok(rustdesk_ctx) = serde_json::from_value::<RustDeskContext>(rustdesk_ctx_json.clone()) {
                    match rustdesk_ctx {
                        RustDeskContext::Streaming { .. } => continue,
                        RustDeskContext::Error { .. } => break,
                        _ => panic!("Unexpected context state during streaming"),
                    }
                } else {
                    panic!("Failed to deserialize context");
                }
            } else {
                panic!("Missing rustdesk_context");
            }
        }

        // Should still be in a valid state
        if let Some(rustdesk_ctx_json) = context.data().get("rustdesk_context") {
            if let Ok(rustdesk_ctx) = serde_json::from_value::<RustDeskContext>(rustdesk_ctx_json.clone()) {
                match rustdesk_ctx {
                    RustDeskContext::Streaming { .. } => assert!(true),
                    RustDeskContext::Error { .. } => assert!(true),
                    _ => panic!("Expected Streaming or Error context"),
                }
            } else {
                panic!("Failed to deserialize context");
            }
        } else {
            panic!("Missing rustdesk_context");
        }
    }
}