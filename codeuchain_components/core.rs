//! Core CodeUChain abstractions
//!
//! This module provides the fundamental building blocks for CodeUChain:
//! - Context: Immutable data container
//! - Chain: Processing pipeline
//! - Link: Individual processing unit
//! - Middleware: Cross-cutting concerns

use async_trait::async_trait;
use serde_json::{Value, Map};
use std::collections::HashMap;
use std::sync::Arc;

/// Result type for CodeUChain operations
pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

/// Context represents immutable state flowing through processing chains
#[derive(Clone, Debug)]
pub struct Context {
    data: Arc<HashMap<String, Value>>,
}

impl Context {
    /// Create a new context with initial data
    pub fn new(data: HashMap<String, Value>) -> Self {
        Self {
            data: Arc::new(data),
        }
    }

    /// Create an empty context
    pub fn empty() -> Self {
        Self::new(HashMap::new())
    }

    /// Get a value from the context
    pub fn get(&self, key: &str) -> Option<&Value> {
        self.data.get(key)
    }

    /// Insert a new value, returning a new context
    pub fn insert(&self, key: String, value: Value) -> Self {
        let mut new_data = (*self.data).clone();
        new_data.insert(key, value);
        Self::new(new_data)
    }

    /// Get the underlying data
    pub fn data(&self) -> &HashMap<String, Value> {
        &self.data
    }

    /// Convert to a Map for JSON serialization
    pub fn to_map(&self) -> Map<String, Value> {
        self.data.iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect()
    }
}

/// Chain represents a processing pipeline of links
#[derive(Default)]
pub struct Chain {
    links: HashMap<String, Box<dyn Link>>,
    connections: Vec<(String, String, Box<dyn Fn(&Context) -> bool + Send + Sync>)>,
    middleware: Vec<Box<dyn Middleware>>,
}

impl Chain {
    /// Create a new empty chain
    pub fn new() -> Self {
        Self {
            links: HashMap::new(),
            connections: Vec::new(),
            middleware: Vec::new(),
        }
    }

    /// Add a link to the chain
    pub fn add_link(&mut self, name: String, link: Box<dyn Link>) {
        self.links.insert(name, link);
    }

    /// Connect two links with a predicate
    pub fn connect<F>(&mut self, from: String, to: String, predicate: F)
    where
        F: Fn(&Context) -> bool + Send + Sync + 'static,
    {
        self.connections.push((from, to, Box::new(predicate)));
    }

    /// Add middleware to the chain
    pub fn use_middleware(&mut self, middleware: Box<dyn Middleware>) {
        self.middleware.push(middleware);
    }

    /// Run the chain with the given context
    pub async fn run(&self, mut ctx: Context) -> Result<Context> {
        // For now, run all links in sequence
        // TODO: Implement proper graph execution with predicates
        for (name, link) in &self.links {
            // Run middleware before
            for middleware in &self.middleware {
                middleware.before(name, &ctx).await?;
            }

            // Run the link
            ctx = link.call(ctx).await?;

            // Run middleware after
            for middleware in &self.middleware {
                middleware.after(name, &ctx).await?;
            }
        }

        Ok(ctx)
    }
}

/// Link trait for processing units
#[async_trait]
pub trait Link: Send + Sync {
    /// Process the context and return a new context
    async fn call(&self, ctx: Context) -> Result<Context>;
}

/// Legacy link trait for backward compatibility
#[async_trait]
pub trait Link: Send + Sync {
    /// Process the context and return a result
    async fn call(&self, ctx: Context) -> Result<Context>;
}

/// Middleware trait for cross-cutting concerns
#[async_trait]
pub trait Middleware: Send + Sync {
    /// Called before link execution
    async fn before(&self, name: &str, ctx: &Context) -> Result<()> {
        Ok(())
    }

    /// Called after link execution
    async fn after(&self, name: &str, ctx: &Context) -> Result<()> {
        Ok(())
    }

    /// Called when a link errors
    async fn on_error(&self, name: &str, ctx: &Context, err: &Box<dyn std::error::Error + Send + Sync>) -> Result<()> {
        Ok(())
    }
}

/// Simple logging middleware
pub struct LoggingMiddleware;

impl LoggingMiddleware {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Middleware for LoggingMiddleware {
    async fn before(&self, name: &str, _ctx: &Context) -> Result<()> {
        println!("🔗 Executing link: {}", name);
        Ok(())
    }

    async fn after(&self, name: &str, _ctx: &Context) -> Result<()> {
        println!("✅ Completed link: {}", name);
        Ok(())
    }

    async fn on_error(&self, name: &str, _ctx: &Context, err: &Box<dyn std::error::Error + Send + Sync>) -> Result<()> {
        println!("❌ Error in link {}: {}", name, err);
        Ok(())
    }
}

/// Performance monitoring middleware
pub struct PerformanceMiddleware {
    timings: Arc<std::sync::Mutex<HashMap<String, Vec<std::time::Duration>>>>,
}

impl PerformanceMiddleware {
    pub fn new() -> Self {
        Self {
            timings: Arc::new(std::sync::Mutex::new(HashMap::new())),
        }
    }

    pub fn get_stats(&self, name: &str) -> Option<(std::time::Duration, usize)> {
        let timings = self.timings.lock().unwrap();
        timings.get(name).map(|times| {
            let total: std::time::Duration = times.iter().sum();
            let avg = total / times.len() as u32;
            (avg, times.len())
        })
    }
}

#[async_trait]
impl Middleware for PerformanceMiddleware {
    async fn before(&self, name: &str, _ctx: &Context) -> Result<()> {
        // Store start time in task local storage
        // For simplicity, we'll skip detailed timing in this stub
        Ok(())
    }

    async fn after(&self, name: &str, _ctx: &Context) -> Result<()> {
        // Record timing
        // For simplicity, we'll skip detailed timing in this stub
        Ok(())
    }
}