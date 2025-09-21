// Simple compilation test for CodeUChain components
use codeuchain::core::{Context, Link, LinkResult};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json;

#[derive(Debug, Clone, Serialize, Deserialize)]
enum TestContext {
    Initial(String),
    Processed(String),
}

struct TestLink;

#[async_trait]
impl Link<TestContext, TestContext> for TestLink {
    async fn call(&self, ctx: Context<TestContext>) -> LinkResult<Context<TestContext>> {
        let data = ctx.data().clone();
        match data {
            TestContext::Initial(msg) => {
                let new_data = TestContext::Processed(format!("Processed: {}", msg));
                Ok(ctx.insert("test_context".to_string(), serde_json::to_value(new_data)?))
            }
            _ => Ok(ctx),
        }
    }
}

#[tokio::main]
async fn main() {
    let ctx = Context::new(TestContext::Initial("Hello CodeUChain!".to_string()));
    let link = TestLink;

    let result = link.call(ctx).await.unwrap();
    println!("Test passed: {:?}", result.data());
}