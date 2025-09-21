// Test CodeUChain Link implementations
use codeuchain::{Context, LegacyLink};
use async_trait::async_trait;
use serde_json::json;
use std::collections::HashMap;

struct TestLink;

#[async_trait]
impl LegacyLink for TestLink {
    async fn call(&self, ctx: Context) -> Result<Context, Box<dyn std::error::Error + Send + Sync>> {
        let data = ctx.data().clone();
        let status = data.get("status").and_then(|v| v.as_str()).unwrap_or("");

        match status {
            "initial" => {
                let mut new_data = data.clone();
                new_data.insert("status".to_string(), json!("processed"));
                new_data.insert("message".to_string(), json!("Processed: Hello CodeUChain!"));
                Ok(Context::new(new_data))
            }
            _ => Ok(ctx),
        }
    }
}

#[tokio::main]
async fn main() {
    println!("Testing CodeUChain Link implementation...");

    let mut initial_data = HashMap::new();
    initial_data.insert("status".to_string(), json!("initial"));
    initial_data.insert("message".to_string(), json!("Hello CodeUChain!"));

    let ctx = Context::new(initial_data);
    let link = TestLink;

    let result = link.call(ctx).await.unwrap();
    println!("✓ Link call successful");

    let result_data = result.data();
    if let (Some(status), Some(message)) = (
        result_data.get("status").and_then(|v| v.as_str()),
        result_data.get("message").and_then(|v| v.as_str())
    ) {
        if status == "processed" && message == "Processed: Hello CodeUChain!" {
            println!("✓ Context transformation correct");
        } else {
            println!("✗ Context transformation incorrect: status={}, message={}", status, message);
        }
    } else {
        println!("✗ Missing expected fields in result");
    }

    println!("CodeUChain API test completed successfully!");
}
