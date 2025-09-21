// CodeUChain-based RustDesk Components Library

pub mod types;
pub mod contexts;
pub mod links;
pub mod chains;
pub mod middleware;
// pub mod message_links; // Temporarily disabled for testing

pub use types::*;
pub use contexts::*;
pub use links::*;
pub use chains::*;
pub use middleware::*;
// pub use message_links::*;

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