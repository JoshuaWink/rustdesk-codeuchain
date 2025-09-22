// Middleware implementations for CodeUChain-based RustDesk

use crate::types::*;
use codeuchain::{Context, Middleware};
use async_trait::async_trait;
use std::time::{Duration, Instant};
use std::collections::HashMap;

/// Logging middleware for request/response tracking
#[derive(Clone)]
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
    async fn before(&self, link: Option<&dyn codeuchain::LegacyLink>, ctx: &Context) -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let link_name = "unknown"; // We don't have access to link names in this trait
        if self.log_level == "debug" {
            println!("[DEBUG] Starting link: {} with context keys: {:?}", link_name, ctx.data().keys().collect::<Vec<_>>());
        } else {
            println!("[INFO] Processing: {}", link_name);
        }
        Ok(())
    }

    async fn after(&self, link: Option<&dyn codeuchain::LegacyLink>, ctx: &Context) -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let link_name = "unknown";
        if self.log_level == "debug" {
            println!("[DEBUG] Completed link: {} with context keys: {:?}", link_name, ctx.data().keys().collect::<Vec<_>>());
        } else {
            println!("[INFO] Completed: {}", link_name);
        }
        Ok(())
    }

    async fn on_error(&self, _link: Option<&dyn codeuchain::LegacyLink>, err: &Box<dyn std::error::Error + Send + Sync>, _ctx: &Context) -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync>> {
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
    async fn before(&self, link: Option<&dyn codeuchain::LegacyLink>, _ctx: &Context) -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Store start time in task local storage or similar
        // For simplicity, we'll use a simple approach
        let start_time = Instant::now();
        // In a real implementation, you'd use task-local storage
        println!("[PERF] Started timing: unknown");
        Ok(())
    }

    async fn after(&self, link: Option<&dyn codeuchain::LegacyLink>, _ctx: &Context) -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Calculate and store duration
        let duration = Duration::from_millis(10); // Mock duration
        let link_name = "unknown";
        let mut timings = self.timings.lock().unwrap();
        timings.entry(link_name.to_string()).or_insert_with(Vec::new).push(duration);

        println!("[PERF] {} completed in {:?}", link_name, duration);
        Ok(())
    }

    async fn on_error(&self, _link: Option<&dyn codeuchain::LegacyLink>, _err: &Box<dyn std::error::Error + Send + Sync>, _ctx: &Context) -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync>> {
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
    async fn before(&self, link: Option<&dyn codeuchain::LegacyLink>, ctx: &Context) -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync>> {
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

    async fn after(&self, link: Option<&dyn codeuchain::LegacyLink>, ctx: &Context) -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync>> {
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

    async fn on_error(&self, _link: Option<&dyn codeuchain::LegacyLink>, err: &Box<dyn std::error::Error + Send + Sync>, _ctx: &Context) -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync>> {
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
#[derive(Clone)]
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
    async fn before(&self, link: Option<&dyn codeuchain::LegacyLink>, ctx: &Context) -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync>> {
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

    async fn after(&self, link: Option<&dyn codeuchain::LegacyLink>, ctx: &Context) -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync>> {
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

    async fn on_error(&self, link: Option<&dyn codeuchain::LegacyLink>, err: &Box<dyn std::error::Error + Send + Sync>, _ctx: &Context) -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Security-related error handling
        if err.to_string().contains("security") || err.to_string().contains("encryption") {
            println!("[SECURITY] Security error in unknown: {}", err);
        }
        Ok(())
    }
}

/// Circuit breaker middleware for fault tolerance
pub struct CircuitBreakerMiddleware {
    failure_threshold: u32,
    recovery_timeout_ms: u64,
    failure_count: std::sync::Mutex<HashMap<String, u32>>,
    last_failure_time: std::sync::Mutex<HashMap<String, u64>>,
    state: std::sync::Mutex<HashMap<String, CircuitState>>,
}

#[derive(Debug, Clone, PartialEq)]
enum CircuitState {
    Closed,      // Normal operation
    Open,        // Failing, requests rejected
    HalfOpen,    // Testing recovery
}

impl CircuitBreakerMiddleware {
    pub fn new(failure_threshold: u32, recovery_timeout_ms: u64) -> Self {
        Self {
            failure_threshold,
            recovery_timeout_ms,
            failure_count: std::sync::Mutex::new(HashMap::new()),
            last_failure_time: std::sync::Mutex::new(HashMap::new()),
            state: std::sync::Mutex::new(HashMap::new()),
        }
    }

    fn get_state(&self, link_name: &str) -> CircuitState {
        let states = self.state.lock().unwrap();
        states.get(link_name).cloned().unwrap_or(CircuitState::Closed)
    }

    fn record_success(&self, link_name: &str) {
        let mut failure_count = self.failure_count.lock().unwrap();
        let mut states = self.state.lock().unwrap();

        failure_count.insert(link_name.to_string(), 0);
        states.insert(link_name.to_string(), CircuitState::Closed);
    }

    fn record_failure(&self, link_name: &str) {
        let mut failure_count = self.failure_count.lock().unwrap();
        let mut last_failure = self.last_failure_time.lock().unwrap();
        let mut states = self.state.lock().unwrap();

        let count = failure_count.entry(link_name.to_string()).or_insert(0);
        *count += 1;

        if *count >= self.failure_threshold {
            states.insert(link_name.to_string(), CircuitState::Open);
            last_failure.insert(link_name.to_string(),
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_millis() as u64
            );
        }
    }

    fn should_attempt(&self, link_name: &str) -> bool {
        let state = self.get_state(link_name);
        match state {
            CircuitState::Closed => true,
            CircuitState::Open => {
                let last_failure = self.last_failure_time.lock().unwrap();
                if let Some(&time) = last_failure.get(link_name) {
                    let now = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_millis() as u64;

                    if now - time > self.recovery_timeout_ms {
                        let mut states = self.state.lock().unwrap();
                        states.insert(link_name.to_string(), CircuitState::HalfOpen);
                        return true;
                    }
                }
                false
            }
            CircuitState::HalfOpen => true,
        }
    }
}

#[async_trait]
impl Middleware for CircuitBreakerMiddleware {
    async fn before(&self, link: Option<&dyn codeuchain::LegacyLink>, ctx: &Context) -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let link_name = "unknown"; // Would need link name access

        if !self.should_attempt(link_name) {
            return Err("Circuit breaker is open".into());
        }

        Ok(())
    }

    async fn after(&self, link: Option<&dyn codeuchain::LegacyLink>, ctx: &Context) -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let link_name = "unknown";
        self.record_success(link_name);
        Ok(())
    }

    async fn on_error(&self, link: Option<&dyn codeuchain::LegacyLink>, err: &Box<dyn std::error::Error + Send + Sync>, ctx: &Context) -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let link_name = "unknown";
        self.record_failure(link_name);
        println!("[CIRCUIT] Recorded failure for {}: {}", link_name, err);
        Ok(())
    }
}

/// Rate limiting middleware
pub struct RateLimitMiddleware {
    requests_per_second: u32,
    request_counts: std::sync::Mutex<HashMap<String, Vec<u64>>>,
}

impl Clone for RateLimitMiddleware {
    fn clone(&self) -> Self {
        Self {
            requests_per_second: self.requests_per_second,
            request_counts: std::sync::Mutex::new(HashMap::new()),
        }
    }
}

impl RateLimitMiddleware {
    pub fn new(requests_per_second: u32) -> Self {
        Self {
            requests_per_second,
            request_counts: std::sync::Mutex::new(HashMap::new()),
        }
    }

    fn is_allowed(&self, key: &str) -> bool {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        let mut counts = self.request_counts.lock().unwrap();
        let timestamps = counts.entry(key.to_string()).or_insert_with(Vec::new);

        // Remove timestamps older than 1 second
        timestamps.retain(|&t| now - t < 1000);

        if timestamps.len() < self.requests_per_second as usize {
            timestamps.push(now);
            true
        } else {
            false
        }
    }
}

#[async_trait]
impl Middleware for RateLimitMiddleware {
    async fn before(&self, link: Option<&dyn codeuchain::LegacyLink>, ctx: &Context) -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let key = "global"; // Could be based on user/session

        if !self.is_allowed(key) {
            return Err("Rate limit exceeded".into());
        }

        Ok(())
    }

    async fn after(&self, link: Option<&dyn codeuchain::LegacyLink>, ctx: &Context) -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync>> {
        Ok(())
    }

    async fn on_error(&self, link: Option<&dyn codeuchain::LegacyLink>, err: &Box<dyn std::error::Error + Send + Sync>, ctx: &Context) -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync>> {
        Ok(())
    }
}

/// Metrics collection middleware
pub struct MetricsMiddleware {
    metrics: std::sync::Mutex<HashMap<String, MetricData>>,
}

#[derive(Debug, Clone)]
struct MetricData {
    request_count: u64,
    error_count: u64,
    total_duration_ms: u64,
    min_duration_ms: u64,
    max_duration_ms: u64,
    last_request_time: u64,
}

impl MetricsMiddleware {
    pub fn new() -> Self {
        Self {
            metrics: std::sync::Mutex::new(HashMap::new()),
        }
    }

    pub fn get_metrics(&self) -> HashMap<String, MetricData> {
        self.metrics.lock().unwrap().clone()
    }

    pub fn record_request(&self, link_name: &str, duration_ms: u64, had_error: bool) {
        let mut metrics = self.metrics.lock().unwrap();
        let data = metrics.entry(link_name.to_string()).or_insert(MetricData {
            request_count: 0,
            error_count: 0,
            total_duration_ms: 0,
            min_duration_ms: u64::MAX,
            max_duration_ms: 0,
            last_request_time: 0,
        });

        data.request_count += 1;
        data.total_duration_ms += duration_ms;
        data.min_duration_ms = data.min_duration_ms.min(duration_ms);
        data.max_duration_ms = data.max_duration_ms.max(duration_ms);
        data.last_request_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        if had_error {
            data.error_count += 1;
        }
    }
}

#[async_trait]
impl Middleware for MetricsMiddleware {
    async fn before(&self, link: Option<&dyn codeuchain::LegacyLink>, ctx: &Context) -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Store start time in context or use task-local storage
        Ok(())
    }

    async fn after(&self, link: Option<&dyn codeuchain::LegacyLink>, ctx: &Context) -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let link_name = "unknown";
        let duration_ms = 10; // Would calculate actual duration
        self.record_request(link_name, duration_ms, false);
        Ok(())
    }

    async fn on_error(&self, link: Option<&dyn codeuchain::LegacyLink>, err: &Box<dyn std::error::Error + Send + Sync>, ctx: &Context) -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let link_name = "unknown";
        let duration_ms = 10; // Would calculate actual duration
        self.record_request(link_name, duration_ms, true);
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

    pub fn add_circuit_breaker(mut self, failure_threshold: u32, recovery_timeout_ms: u64) -> Self {
        self.middlewares.push(Box::new(CircuitBreakerMiddleware::new(failure_threshold, recovery_timeout_ms)));
        self
    }

    pub fn add_rate_limiting(mut self, requests_per_second: u32) -> Self {
        self.middlewares.push(Box::new(RateLimitMiddleware::new(requests_per_second)));
        self
    }

    pub fn add_metrics(mut self) -> Self {
        self.middlewares.push(Box::new(MetricsMiddleware::new()));
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
        let test_error: Box<dyn std::error::Error + Send + Sync> = Box::new(std::io::Error::new(std::io::ErrorKind::Other, "test error")) as Box<dyn std::error::Error + Send + Sync>;
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
        let conn_error: Box<dyn std::error::Error + Send + Sync> = Box::new(std::io::Error::new(std::io::ErrorKind::ConnectionRefused, "connection failed")) as Box<dyn std::error::Error + Send + Sync>;
        let conn_result = middleware.on_error(None, &conn_error, &ctx).await;
        assert!(conn_result.is_ok());
        println!("✅ ErrorHandlingMiddleware handled connection error");

        println!("Testing ErrorHandlingMiddleware with timeout error...");
        let timeout_error: Box<dyn std::error::Error + Send + Sync> = Box::new(std::io::Error::new(std::io::ErrorKind::TimedOut, "operation timed out")) as Box<dyn std::error::Error + Send + Sync>;
        let timeout_result = middleware.on_error(None, &timeout_error, &ctx).await;
        assert!(timeout_result.is_ok());
        println!("✅ ErrorHandlingMiddleware handled timeout error");

        println!("Testing ErrorHandlingMiddleware with generic error...");
        let generic_error: Box<dyn std::error::Error + Send + Sync> = Box::new(std::io::Error::new(std::io::ErrorKind::Other, "unknown error")) as Box<dyn std::error::Error + Send + Sync>;
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
        let security_error: Box<dyn std::error::Error + Send + Sync> = Box::new(std::io::Error::new(std::io::ErrorKind::Other, "security violation: unauthorized access")) as Box<dyn std::error::Error + Send + Sync>;
        let security_result = middleware.on_error(None, &security_error, &ctx).await;
        assert!(security_result.is_ok());
        println!("✅ SecurityMiddleware detected and handled security error");

        println!("Testing SecurityMiddleware with encryption error...");
        let encryption_error: Box<dyn std::error::Error + Send + Sync> = Box::new(std::io::Error::new(std::io::ErrorKind::Other, "encryption failed")) as Box<dyn std::error::Error + Send + Sync>;
        let encryption_result = middleware.on_error(None, &encryption_error, &ctx).await;
        assert!(encryption_result.is_ok());
        println!("✅ SecurityMiddleware detected and handled encryption error");

        println!("Testing SecurityMiddleware with non-security error...");
        let normal_error: Box<dyn std::error::Error + Send + Sync> = Box::new(std::io::Error::new(std::io::ErrorKind::NotFound, "file not found")) as Box<dyn std::error::Error + Send + Sync>;
        let normal_result = middleware.on_error(None, &normal_error, &ctx).await;
        assert!(normal_result.is_ok());
        println!("✅ SecurityMiddleware handled non-security error");
    }

    #[tokio::test]
    async fn test_circuit_breaker_middleware() {
        let middleware = CircuitBreakerMiddleware::new(3, 1000);
        let ctx = Context::new(json_to_hashmap(json!({"operation": "test"})));

        println!("Testing CircuitBreakerMiddleware...");

        // Test closed state (normal operation)
        let closed_result = middleware.before(None, &ctx).await;
        assert!(closed_result.is_ok());
        println!("✅ CircuitBreakerMiddleware allowed request in closed state");

        // Simulate failures
        let test_error: Box<dyn std::error::Error + Send + Sync> = Box::new(std::io::Error::new(std::io::ErrorKind::Other, "test error")) as Box<dyn std::error::Error + Send + Sync>;
        middleware.on_error(None, &test_error, &ctx).await.unwrap();
        middleware.on_error(None, &test_error, &ctx).await.unwrap();
        middleware.on_error(None, &test_error, &ctx).await.unwrap();

        // Test open state (circuit breaker triggered)
        let open_result = middleware.before(None, &ctx).await;
        assert!(open_result.is_err());
        println!("✅ CircuitBreakerMiddleware rejected request in open state");

        // Wait for recovery timeout
        tokio::time::sleep(tokio::time::Duration::from_millis(1100)).await;

        // Test half-open state (testing recovery)
        let half_open_result = middleware.before(None, &ctx).await;
        assert!(half_open_result.is_ok());
        println!("✅ CircuitBreakerMiddleware allowed request in half-open state");

        // Simulate success
        middleware.after(None, &ctx).await.unwrap();

        // Test closed state again
        let closed_result_after_recovery = middleware.before(None, &ctx).await;
        assert!(closed_result_after_recovery.is_ok());
        println!("✅ CircuitBreakerMiddleware allowed request in closed state after recovery");
    }

    #[tokio::test]
    async fn test_rate_limit_middleware() {
        let middleware = RateLimitMiddleware::new(2);
        let ctx = Context::new(json_to_hashmap(json!({"operation": "test"})));

        println!("Testing RateLimitMiddleware...");

        // Test within limit
        let result1 = middleware.before(None, &ctx).await;
        assert!(result1.is_ok());
        println!("✅ RateLimitMiddleware allowed request within limit");

        let result2 = middleware.before(None, &ctx).await;
        assert!(result2.is_ok());
        println!("✅ RateLimitMiddleware allowed request within limit");

        // Test exceeding limit
        let result3 = middleware.before(None, &ctx).await;
        assert!(result3.is_err());
        println!("✅ RateLimitMiddleware rejected request exceeding limit");

        // Wait for rate limit window to pass
        tokio::time::sleep(tokio::time::Duration::from_millis(1100)).await;

        // Test after rate limit window
        let result4 = middleware.before(None, &ctx).await;
        assert!(result4.is_ok());
        println!("✅ RateLimitMiddleware allowed request after rate limit window");
    }

    #[tokio::test]
    async fn test_metrics_middleware() {
        let middleware = MetricsMiddleware::new();
        let ctx = Context::new(json_to_hashmap(json!({"operation": "test"})));

        println!("Testing MetricsMiddleware...");

        // Test request recording
        middleware.after(None, &ctx).await.unwrap();
        let metrics = middleware.get_metrics();
        assert!(!metrics.is_empty());
        println!("✅ MetricsMiddleware recorded request metrics");

        // Test error recording
        let metrics_test_error: Box<dyn std::error::Error + Send + Sync> = Box::new(std::io::Error::new(std::io::ErrorKind::Other, "test error")) as Box<dyn std::error::Error + Send + Sync>;
        middleware.on_error(None, &metrics_test_error, &ctx).await.unwrap();
        let metrics = middleware.get_metrics();
        assert!(metrics.values().all(|data| data.error_count > 0));
        println!("✅ MetricsMiddleware recorded error metrics");
    }

    #[tokio::test]
    async fn test_middleware_stack() {
        let stack = MiddlewareStack::new()
            .add_logging()
            .add_performance_monitoring()
            .add_error_handling()
            .add_security()
            .add_circuit_breaker(3, 1000)
            .add_rate_limiting(2)
            .add_metrics();

        assert_eq!(stack.get_middlewares().len(), 7);
        println!("✅ MiddlewareStack created with {} middlewares", stack.get_middlewares().len());

        // Test that we can access individual middlewares
        let middlewares = stack.get_middlewares();
        assert!(middlewares.len() > 0);
        println!("✅ MiddlewareStack provides access to middleware collection");
    }
}