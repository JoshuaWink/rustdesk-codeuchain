// CodeUChain-based modular components for RustDesk

pub mod types;
pub mod contexts;
pub mod links;
pub mod chains;
pub mod middleware;
pub mod migration; // Migration infrastructure
pub mod ipc_facade; // IPC facade for API compatibility
pub mod ipc_links; // IPC processing links
pub mod ipc_chains; // IPC processing chains
pub mod client_links; // Client system processing links
pub mod client_chains; // Client system processing chains
pub mod server_links; // Server system processing links
pub mod server_chains; // Server system processing chains
pub mod ui_links; // UI interface processing links
pub mod ui_chains; // UI interface processing chains

pub use types::*;
pub use contexts::*;
pub use links::*;
pub use chains::*;
pub use middleware::*;
pub use migration::*;
pub use ipc_facade::*;
pub use ipc_links::*;
pub use ipc_chains::*;
pub use client_links::*;
pub use client_chains::*;
pub use server_links::*;
pub use server_chains::*;
pub use ui_links::*;
pub use ui_chains::*;

// Include E2E tests
#[cfg(test)]
mod e2e_tests;

#[cfg(test)]
mod tests {
    use super::*;
    use codeuchain::core::{Chain, Context};

    #[tokio::test]
    async fn test_codeuchain_basic_integration() {
        // Create a CodeUChain to demonstrate basic functionality
        let mut chain = Chain::new();

        // Create initial context with test data
        let mut initial_data = std::collections::HashMap::new();
        initial_data.insert("session_id".to_string(), serde_json::Value::String("test-session".to_string()));
        initial_data.insert("message_count".to_string(), serde_json::Value::Number(0.into()));

        let ctx = Context::new(initial_data);

        // Run the chain (empty chain just passes through context)
        let result_ctx = chain.run(ctx).await.unwrap();

        // Verify the context was processed
        assert!(result_ctx.get("session_id").is_some());
        assert_eq!(result_ctx.get("session_id").unwrap().as_str().unwrap(), "test-session");
        assert!(result_ctx.get("message_count").is_some());

        println!("✅ CodeUChain basic integration test passed!");
        println!("📊 Context keys: {:?}", result_ctx.data().keys().collect::<Vec<_>>());
    }
}