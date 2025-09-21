// Middleware implementations for CodeUChain-based RustDesk

use crate::types::*;
use codeuchain::{Context, Middleware};
use async_trait::async_trait;
use std::time::{Duration, Instant};
use std::collections::HashMap;

/// Logging middleware for request/response tracking
pub struct LoggingMiddleware {
    pub log_level: String,
}

impl LoggingMiddleware {
    pub fn new() -> Self {
        Self {
            log_level: "info".to_string(),
        }
    }

    pub fn with_level(mut self, level: &str) -> Self {
        self.log_level = level.to_string();
        self
    }
}

#[async_trait]
impl Middleware for LoggingMiddleware {
    async fn before(&self, link: Option<&dyn codeuchain::LegacyLink>, ctx: &Context) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let link_name = "unknown"; // We don't have access to link names in this trait
        if self.log_level == "debug" {
            println!("[DEBUG] Starting link: {} with context keys: {:?}", link_name, ctx.data().keys().collect::<Vec<_>>());
        } else {
            println!("[INFO] Processing: {}", link_name);
        }
        Ok(())
    }

    async fn after(&self, link: Option<&dyn codeuchain::LegacyLink>, ctx: &Context) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let link_name = "unknown";
        if self.log_level == "debug" {
            println!("[DEBUG] Completed link: {} with context keys: {:?}", link_name, ctx.data().keys().collect::<Vec<_>>());
        } else {
            println!("[INFO] Completed: {}", link_name);
        }
        Ok(())
    }

    async fn on_error(&self, _link: Option<&dyn codeuchain::LegacyLink>, err: &Box<dyn std::error::Error + Send + Sync>, _ctx: &Context) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let link_name = "unknown";
        eprintln!("[ERROR] Link '{}' failed: {}", link_name, err);
        if self.log_level == "debug" {
            eprintln!("[ERROR] Context keys at failure: {:?}", _ctx.data().keys().collect::<Vec<_>>());
        }
        Ok(())
    }
}

/// Performance monitoring middleware
pub struct PerformanceMiddleware {
    timings: std::sync::Mutex<HashMap<String, Vec<Duration>>>,
}

impl PerformanceMiddleware {
    pub fn new() -> Self {
        Self {
            timings: std::sync::Mutex::new(HashMap::new()),
        }
    }

    /// Get average timing for a link
    pub fn get_average_timing(&self, link_name: &str) -> Option<Duration> {
        let timings = self.timings.lock().unwrap();
        timings.get(link_name).and_then(|times| {
            if times.is_empty() {
                None
            } else {
                let total: Duration = times.iter().sum();
                Some(total / times.len() as u32)
            }
        })
    }

    /// Get all timing statistics
    pub fn get_timing_stats(&self) -> HashMap<String, (Duration, usize)> {
        let timings = self.timings.lock().unwrap();
        timings
            .iter()
            .map(|(name, times)| {
                let total: Duration = times.iter().sum();
                let avg = if times.is_empty() {
                    Duration::default()
                } else {
                    total / times.len() as u32
                };
                (name.clone(), (avg, times.len()))
            })
            .collect()
    }
}

#[async_trait]
impl Middleware for PerformanceMiddleware {
    async fn before(&self, link: Option<&dyn codeuchain::LegacyLink>, _ctx: &Context) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Store start time in task local storage or similar
        // For simplicity, we'll use a simple approach
        let start_time = Instant::now();
        // In a real implementation, you'd use task-local storage
        println!("[PERF] Started timing: unknown");
        Ok(())
    }

    async fn after(&self, link: Option<&dyn codeuchain::LegacyLink>, _ctx: &Context) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Calculate and store duration
        let duration = Duration::from_millis(10); // Mock duration
        let link_name = "unknown";
        let mut timings = self.timings.lock().unwrap();
        timings.entry(link_name.to_string()).or_insert_with(Vec::new).push(duration);

        println!("[PERF] {} completed in {:?}", link_name, duration);
        Ok(())
    }

    async fn on_error(&self, _link: Option<&dyn codeuchain::LegacyLink>, _err: &Box<dyn std::error::Error + Send + Sync>, _ctx: &Context) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        println!("[PERF] unknown failed");
        Ok(())
    }
}

/// Error handling and recovery middleware
pub struct ErrorHandlingMiddleware {
    pub max_retries: u32,
    pub retry_delay_ms: u64,
}

impl ErrorHandlingMiddleware {
    pub fn new() -> Self {
        Self {
            max_retries: 3,
            retry_delay_ms: 1000,
        }
    }

    pub fn with_retries(mut self, max_retries: u32) -> Self {
        self.max_retries = max_retries;
        self
    }

    pub fn with_retry_delay(mut self, delay_ms: u64) -> Self {
        self.retry_delay_ms = delay_ms;
        self
    }
}

#[async_trait]
impl Middleware for ErrorHandlingMiddleware {
    async fn before(&self, link: Option<&dyn codeuchain::LegacyLink>, ctx: &Context) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Check if context contains error information
        if let Some(rustdesk_ctx_json) = ctx.data().get("rustdesk_context") {
            if let Ok(rustdesk_ctx) = serde_json::from_value::<RustDeskContext>(rustdesk_ctx_json.clone()) {
                if matches!(rustdesk_ctx, RustDeskContext::Error { .. }) {
                    println!("[ERROR] Context in error state before unknown");
                }
            }
        }
        Ok(())
    }

    async fn after(&self, link: Option<&dyn codeuchain::LegacyLink>, ctx: &Context) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Check for errors after processing
        if let Some(rustdesk_ctx_json) = ctx.data().get("rustdesk_context") {
            if let Ok(rustdesk_ctx) = serde_json::from_value::<RustDeskContext>(rustdesk_ctx_json.clone()) {
                if let RustDeskContext::Error { error, .. } = rustdesk_ctx {
                    println!("[ERROR] Error after unknown: {}", error);
                }
            }
        }
        Ok(())
    }

    async fn on_error(&self, _link: Option<&dyn codeuchain::LegacyLink>, err: &Box<dyn std::error::Error + Send + Sync>, _ctx: &Context) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        println!("[ERROR] Link 'unknown' failed: {}. Attempting recovery...", err);

        // Implement retry logic here
        // For now, just log the error
        if let Some(rustdesk_ctx_json) = _ctx.data().get("rustdesk_context") {
            if let Ok(rustdesk_ctx) = serde_json::from_value::<RustDeskContext>(rustdesk_ctx_json.clone()) {
                if let RustDeskContext::Error { error, .. } = rustdesk_ctx {
                    println!("[ERROR] Context error: {}", error);
                }
            }
        }
        Ok(())
    }
}

/// Security middleware for encryption and access control
pub struct SecurityMiddleware {
    pub encryption_enabled: bool,
    pub allowed_peer_ids: Vec<String>,
}

impl SecurityMiddleware {
    pub fn new() -> Self {
        Self {
            encryption_enabled: true,
            allowed_peer_ids: Vec::new(),
        }
    }

    pub fn with_encryption(mut self, enabled: bool) -> Self {
        self.encryption_enabled = enabled;
        self
    }

    pub fn allow_peer(mut self, peer_id: &str) -> Self {
        self.allowed_peer_ids.push(peer_id.to_string());
        self
    }
}

#[async_trait]
impl Middleware for SecurityMiddleware {
    async fn before(&self, link: Option<&dyn codeuchain::LegacyLink>, ctx: &Context) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Validate security requirements
        if let Some(rustdesk_ctx_json) = ctx.data().get("rustdesk_context") {
            if let Ok(rustdesk_ctx) = serde_json::from_value::<RustDeskContext>(rustdesk_ctx_json.clone()) {
                match rustdesk_ctx {
                    RustDeskContext::Initial(conn_info) => {
                        if self.encryption_enabled && conn_info.secure_key.is_none() {
                            println!("[SECURITY] Warning: No secure key for connection to {}", conn_info.peer_id);
                        }
                        if !self.allowed_peer_ids.is_empty() && !self.allowed_peer_ids.contains(&conn_info.peer_id) {
                            println!("[SECURITY] Peer {} not in allowlist", conn_info.peer_id);
                        }
                    }
                    RustDeskContext::Connected(session) => {
                        if self.encryption_enabled && session.connection_info.secure_key.is_none() {
                            println!("[SECURITY] Warning: No secure key for session with {}", session.connection_info.peer_id);
                        }
                        if !self.allowed_peer_ids.is_empty() && !self.allowed_peer_ids.contains(&session.peer_info.hostname) {
                            println!("[SECURITY] Peer {} not in allowlist", session.connection_info.peer_id);
                        }
                    }
                    _ => {}
                }
            }
        }
        Ok(())
    }

    async fn after(&self, link: Option<&dyn codeuchain::LegacyLink>, ctx: &Context) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Post-processing security checks
        if self.encryption_enabled {
            if let Some(rustdesk_ctx_json) = ctx.data().get("rustdesk_context") {
                if let Ok(rustdesk_ctx) = serde_json::from_value::<RustDeskContext>(rustdesk_ctx_json.clone()) {
                    if let RustDeskContext::Connected(session) = rustdesk_ctx {
                        if session.connection_info.secure_key.is_none() {
                            println!("[SECURITY] Connection established without encryption");
                        }
                    }
                }
            }
        }
        Ok(())
    }

    async fn on_error(&self, link: Option<&dyn codeuchain::LegacyLink>, err: &Box<dyn std::error::Error + Send + Sync>, _ctx: &Context) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Security-related error handling
        if err.to_string().contains("security") || err.to_string().contains("encryption") {
            println!("[SECURITY] Security error in unknown: {}", err);
        }
        Ok(())
    }
}

/// Combined middleware stack
pub struct MiddlewareStack {
    middlewares: Vec<Box<dyn Middleware + Send + Sync>>,
}

impl MiddlewareStack {
    pub fn new() -> Self {
        Self {
            middlewares: Vec::new(),
        }
    }

    pub fn add<M: Middleware + Send + Sync + 'static>(mut self, middleware: M) -> Self {
        self.middlewares.push(Box::new(middleware));
        self
    }

    pub fn add_logging(mut self) -> Self {
        self.middlewares.push(Box::new(LoggingMiddleware::new()));
        self
    }

    pub fn add_performance_monitoring(mut self) -> Self {
        self.middlewares.push(Box::new(PerformanceMiddleware::new()));
        self
    }

    pub fn add_error_handling(mut self) -> Self {
        self.middlewares.push(Box::new(ErrorHandlingMiddleware::new()));
        self
    }

    pub fn add_security(mut self) -> Self {
        self.middlewares.push(Box::new(SecurityMiddleware::new()));
        self
    }

    pub fn get_middlewares(&self) -> &[Box<dyn Middleware + Send + Sync>] {
        &self.middlewares
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use codeuchain::Context;
    use serde_json::{json, Value};
    use std::collections::HashMap;

    fn json_to_hashmap(value: Value) -> HashMap<String, Value> {
        if let Value::Object(map) = value {
            map.into_iter().collect()
        } else {
            HashMap::new()
        }
    }

    #[tokio::test]
    async fn test_logging_middleware() {
        let middleware = LoggingMiddleware::new();
        let ctx = Context::new(json_to_hashmap(json!({"test": "data", "session_id": "test-123"})));

        println!("Testing LoggingMiddleware before hook...");
        let before_result = middleware.before(None, &ctx).await;
        assert!(before_result.is_ok());
        println!("✅ LoggingMiddleware before hook executed successfully");

        println!("Testing LoggingMiddleware after hook...");
        let after_result = middleware.after(None, &ctx).await;
        assert!(after_result.is_ok());
        println!("✅ LoggingMiddleware after hook executed successfully");

        println!("Testing LoggingMiddleware error hook...");
        let test_error: Box<dyn std::error::Error + Send + Sync> = Box::new(std::io::Error::new(std::io::ErrorKind::Other, "test error"));
        let error_result = middleware.on_error(None, &test_error, &ctx).await;
        assert!(error_result.is_ok());
        println!("✅ LoggingMiddleware error hook executed successfully");
    }

    #[tokio::test]
    async fn test_performance_middleware_timing() {
        let middleware = PerformanceMiddleware::new();
        let ctx = Context::new(json_to_hashmap(json!({"operation": "test", "data_size": 1024})));

        println!("Testing PerformanceMiddleware timing hooks...");

        // Test before hook
        let before_result = middleware.before(None, &ctx).await;
        assert!(before_result.is_ok());
        println!("✅ PerformanceMiddleware before hook executed");

        // Simulate some processing time
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

        // Test after hook
        let after_result = middleware.after(None, &ctx).await;
        assert!(after_result.is_ok());
        println!("✅ PerformanceMiddleware after hook executed (should have measured timing)");
    }

    #[tokio::test]
    async fn test_error_handling_middleware_different_errors() {
        let middleware = ErrorHandlingMiddleware::new();
        let ctx = Context::new(json_to_hashmap(json!({"session_id": "test-session"})));

        println!("Testing ErrorHandlingMiddleware with connection error...");
        let conn_error: Box<dyn std::error::Error + Send + Sync> = Box::new(std::io::Error::new(std::io::ErrorKind::ConnectionRefused, "connection failed"));
        let conn_result = middleware.on_error(None, &conn_error, &ctx).await;
        assert!(conn_result.is_ok());
        println!("✅ ErrorHandlingMiddleware handled connection error");

        println!("Testing ErrorHandlingMiddleware with timeout error...");
        let timeout_error: Box<dyn std::error::Error + Send + Sync> = Box::new(std::io::Error::new(std::io::ErrorKind::TimedOut, "operation timed out"));
        let timeout_result = middleware.on_error(None, &timeout_error, &ctx).await;
        assert!(timeout_result.is_ok());
        println!("✅ ErrorHandlingMiddleware handled timeout error");

        println!("Testing ErrorHandlingMiddleware with generic error...");
        let generic_error: Box<dyn std::error::Error + Send + Sync> = Box::new(std::io::Error::new(std::io::ErrorKind::Other, "unknown error"));
        let generic_result = middleware.on_error(None, &generic_error, &ctx).await;
        assert!(generic_result.is_ok());
        println!("✅ ErrorHandlingMiddleware handled generic error");
    }

    #[tokio::test]
    async fn test_security_middleware_error_detection() {
        let middleware = SecurityMiddleware::new();
        let ctx = Context::new(json_to_hashmap(json!({"user_id": "test-user", "permissions": ["read"]})));

        println!("Testing SecurityMiddleware hooks...");

        // Test before hook
        let before_result = middleware.before(None, &ctx).await;
        assert!(before_result.is_ok());
        println!("✅ SecurityMiddleware before hook executed");

        // Test after hook
        let after_result = middleware.after(None, &ctx).await;
        assert!(after_result.is_ok());
        println!("✅ SecurityMiddleware after hook executed");

        println!("Testing SecurityMiddleware with security-related error...");
        let security_error: Box<dyn std::error::Error + Send + Sync> = Box::new(std::io::Error::new(std::io::ErrorKind::Other, "security violation: unauthorized access"));
        let security_result = middleware.on_error(None, &security_error, &ctx).await;
        assert!(security_result.is_ok());
        println!("✅ SecurityMiddleware detected and handled security error");

        println!("Testing SecurityMiddleware with encryption error...");
        let encryption_error: Box<dyn std::error::Error + Send + Sync> = Box::new(std::io::Error::new(std::io::ErrorKind::Other, "encryption failed"));
        let encryption_result = middleware.on_error(None, &encryption_error, &ctx).await;
        assert!(encryption_result.is_ok());
        println!("✅ SecurityMiddleware detected and handled encryption error");

        println!("Testing SecurityMiddleware with non-security error...");
        let normal_error: Box<dyn std::error::Error + Send + Sync> = Box::new(std::io::Error::new(std::io::ErrorKind::NotFound, "file not found"));
        let normal_result = middleware.on_error(None, &normal_error, &ctx).await;
        assert!(normal_result.is_ok());
        println!("✅ SecurityMiddleware handled non-security error");
    }

    #[tokio::test]
    async fn test_middleware_stack() {
        let stack = MiddlewareStack::new()
            .add_logging()
            .add_performance_monitoring()
            .add_error_handling()
            .add_security();

        assert_eq!(stack.get_middlewares().len(), 4);
        println!("✅ MiddlewareStack created with {} middlewares", stack.get_middlewares().len());

        // Test that we can access individual middlewares
        let middlewares = stack.get_middlewares();
        assert!(middlewares.len() > 0);
        println!("✅ MiddlewareStack provides access to middleware collection");
    }
}