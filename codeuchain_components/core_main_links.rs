use crate::types::*;
use codeuchain::{Context, LegacyLink};
use async_trait::async_trait;
use serde_json;
use std::collections::HashMap;

/// Type aliases for compatibility
pub type LinkResult<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

/// Argument processing link for handling command-line arguments
pub struct ArgumentProcessingLink;

impl ArgumentProcessingLink {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl LegacyLink for ArgumentProcessingLink {
    async fn call(&self, ctx: Context) -> LinkResult<Context> {
        let data = ctx.data().clone();

        // Extract command line arguments
        let args: Vec<String> = std::env::args().collect();
        let mut processed_args = Vec::new();
        let mut flutter_args = Vec::new();
        let mut flags = HashMap::new();

        // Skip executable name (args[0])
        for (i, arg) in args.iter().enumerate().skip(1) {
            if arg == "--elevate" {
                flags.insert("elevate".to_string(), serde_json::Value::Bool(true));
            } else if arg == "--run-as-system" {
                flags.insert("run_as_system".to_string(), serde_json::Value::Bool(true));
            } else if arg == "--quick_support" {
                flags.insert("quick_support".to_string(), serde_json::Value::Bool(true));
            } else if arg == "--no-server" {
                flags.insert("no_server".to_string(), serde_json::Value::Bool(true));
            } else if arg.starts_with("--connect") || arg.starts_with("--play") ||
                      arg.starts_with("--file-transfer") || arg.starts_with("--view-camera") ||
                      arg.starts_with("--port-forward") || arg.starts_with("--rdp") {
                flags.insert("flutter_invoke_new_connection".to_string(), serde_json::Value::Bool(true));
                processed_args.push(arg.clone());
            } else {
                processed_args.push(arg.clone());
            }
        }

        // Determine if this is a click setup scenario
        #[cfg(windows)]
        let click_setup = processed_args.is_empty() && crate::common::is_setup(&args[0]);
        #[cfg(not(windows))]
        let click_setup = false;

        if click_setup && !hbb_common::config::is_disable_installation() {
            processed_args.push("--install".to_owned());
            flutter_args.push("--install".to_string());
        }

        let mut new_data = data.clone();
        new_data.insert("processed_args".to_string(), serde_json::Value::Array(
            processed_args.into_iter().map(serde_json::Value::String).collect()
        ));
        new_data.insert("flutter_args".to_string(), serde_json::Value::Array(
            flutter_args.into_iter().map(serde_json::Value::String).collect()
        ));
        new_data.insert("flags".to_string(), serde_json::Value::Object(
            flags.into_iter().collect()
        ));
        new_data.insert("click_setup".to_string(), serde_json::Value::Bool(click_setup));
        new_data.insert("argument_processing_complete".to_string(), serde_json::Value::Bool(true));

        Ok(Context::new(new_data))
    }
}

/// Configuration loading and validation link
pub struct ConfigurationLink;

impl ConfigurationLink {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl LegacyLink for ConfigurationLink {
    async fn call(&self, ctx: Context) -> LinkResult<Context> {
        let data = ctx.data().clone();

        // Check if arguments were processed
        let args_processed = data.get("argument_processing_complete")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        if !args_processed {
            return Err(Box::new(CodeUChainError::ValidationError("Arguments not processed".to_string())));
        }

        // Load configuration
        #[cfg(windows)]
        hbb_common::config::PeerConfig::preload_peers();

        // Set up logging name based on first argument
        let processed_args = data.get("processed_args")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();

        let log_name = if !processed_args.is_empty() {
            let first_arg = processed_args[0].as_str().unwrap_or("");
            if first_arg.starts_with("--") {
                first_arg.replace("--", "")
            } else {
                "".to_string()
            }
        } else {
            "".to_string()
        };

        // Initialize logging
        hbb_common::init_log(false, &log_name);

        let mut new_data = data.clone();
        new_data.insert("log_name".to_string(), serde_json::Value::String(log_name));
        new_data.insert("configuration_loaded".to_string(), serde_json::Value::Bool(true));

        Ok(Context::new(new_data))
    }
}

/// Service initialization link for platform services
pub struct ServiceInitializationLink;

impl ServiceInitializationLink {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl LegacyLink for ServiceInitializationLink {
    async fn call(&self, ctx: Context) -> LinkResult<Context> {
        let data = ctx.data().clone();

        // Check if configuration was loaded
        let config_loaded = data.get("configuration_loaded")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        if !config_loaded {
            return Err(Box::new(CodeUChainError::ValidationError("Configuration not loaded".to_string())));
        }

        let flags = data.get("flags")
            .and_then(|v| v.as_object())
            .cloned()
            .unwrap_or_default();

        // Platform-specific initialization
        #[cfg(not(debug_assertions))]
        #[cfg(not(any(target_os = "android", target_os = "ios")))]
        hbb_common::platform::register_breakdown_handler(crate::platform::breakdown_callback);

        // Linux software rendering configuration
        #[cfg(all(target_os = "linux", feature = "flutter"))]
        {
            let (k, v) = ("LIBGL_ALWAYS_SOFTWARE", "1");
            if hbb_common::config::option2bool(
                "allow-always-software-render",
                &hbb_common::config::Config::get_option("allow-always-software-render"),
            ) {
                std::env::set_var(k, v);
            } else {
                std::env::remove_var(k);
            }
        }

        // Windows CPU performance monitoring
        #[cfg(windows)]
        {
            let processed_args = data.get("processed_args")
                .and_then(|v| v.as_array())
                .cloned()
                .unwrap_or_default();

            let args_strings: Vec<String> = processed_args.iter()
                .filter_map(|v| v.as_str())
                .map(|s| s.to_string())
                .collect();

            if args_strings.contains(&"--connect".to_string()) ||
               args_strings.contains(&"--view-camera".to_string()) {
                hbb_common::platform::windows::start_cpu_performance_monitor();
            }
        }

        // Plugin initialization
        #[cfg(all(feature = "flutter", feature = "plugin_framework"))]
        #[cfg(not(any(target_os = "android", target_os = "ios")))]
        {
            let processed_args = data.get("processed_args")
                .and_then(|v| v.as_array())
                .cloned()
                .unwrap_or_default();

            let args_strings: Vec<String> = processed_args.iter()
                .filter_map(|v| v.as_str())
                .map(|s| s.to_string())
                .collect();

            Self::init_plugins(&args_strings);
        }

        let mut new_data = data.clone();
        new_data.insert("services_initialized".to_string(), serde_json::Value::Bool(true));

        Ok(Context::new(new_data))
    }
}

impl ServiceInitializationLink {
    #[cfg(all(feature = "flutter", feature = "plugin_framework"))]
    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    fn init_plugins(args: &Vec<String>) {
        if args.is_empty() || "--server" == (&args[0] as &str) {
            #[cfg(debug_assertions)]
            let load_plugins = true;
            #[cfg(not(debug_assertions))]
            let load_plugins = crate::platform::is_installed();
            if load_plugins {
                crate::plugin::init();
            }
        } else if "--service" == (&args[0] as &str) {
            hbb_common::allow_err!(crate::plugin::remove_uninstalled());
        }
    }
}

/// Lifecycle management link for application startup
pub struct LifecycleManagementLink;

impl LifecycleManagementLink {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl LegacyLink for LifecycleManagementLink {
    async fn call(&self, ctx: Context) -> LinkResult<Context> {
        let data = ctx.data().clone();

        // Check if services were initialized
        let services_initialized = data.get("services_initialized")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        if !services_initialized {
            return Err(Box::new(CodeUChainError::ValidationError("Services not initialized".to_string())));
        }

        let processed_args = data.get("processed_args")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();

        let flags = data.get("flags")
            .and_then(|v| v.as_object())
            .cloned()
            .unwrap_or_default();

        let args_strings: Vec<String> = processed_args.iter()
            .filter_map(|v| v.as_str())
            .map(|s| s.to_string())
            .collect();

        // Handle different startup scenarios
        if args_strings.is_empty() || crate::common::is_empty_uni_link(&args_strings[0]) {
            // Default server startup
            let no_server = flags.get("no_server")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);

            std::thread::spawn(move || crate::start_server(false, no_server));
        } else {
            // Handle specific commands
            let result = Self::handle_command(&args_strings, &flags).await?;
            if let Some(termination_reason) = result {
                let mut new_data = data.clone();
                new_data.insert("termination_reason".to_string(), serde_json::Value::String(termination_reason));
                new_data.insert("should_terminate".to_string(), serde_json::Value::Bool(true));
                return Ok(Context::new(new_data));
            }
        }

        let mut new_data = data.clone();
        new_data.insert("lifecycle_managed".to_string(), serde_json::Value::Bool(true));
        new_data.insert("should_terminate".to_string(), serde_json::Value::Bool(false));

        Ok(Context::new(new_data))
    }
}

impl LifecycleManagementLink {
    async fn handle_command(args: &[String], flags: &serde_json::Map<String, serde_json::Value>) -> LinkResult<Option<String>> {
        if args.is_empty() {
            return Ok(None);
        }

        let command = &args[0];

        match command.as_str() {
            "--version" => {
                println!("{}", crate::VERSION);
                Ok(Some("version_displayed".to_string()))
            }
            "--build-date" => {
                println!("{}", crate::BUILD_DATE);
                Ok(Some("build_date_displayed".to_string()))
            }
            "--noinstall" => {
                Ok(Some("no_install_requested".to_string()))
            }
            "--tray" => {
                if !crate::check_process("--tray", true) {
                    crate::tray::start_tray();
                }
                Ok(Some("tray_started".to_string()))
            }
            "--server" => {
                Self::handle_server_command(flags).await?;
                Ok(Some("server_started".to_string()))
            }
            "--service" => {
                log::info!("start --service");
                crate::start_os_service();
                Ok(Some("service_started".to_string()))
            }
            "--install-service" => {
                log::info!("start --install-service");
                crate::platform::install_service();
                Ok(Some("service_installed".to_string()))
            }
            "--uninstall-service" => {
                log::info!("start --uninstall-service");
                crate::platform::uninstall_service(false, true);
                Ok(Some("service_uninstalled".to_string()))
            }
            // Add more command handlers as needed
            _ => Ok(None)
        }
    }

    async fn handle_server_command(flags: &serde_json::Map<String, serde_json::Value>) -> LinkResult<()> {
        log::info!("start --server with user {}", crate::username());

        #[cfg(target_os = "linux")]
        {
            hbb_common::allow_err!(crate::platform::check_autostart_config());
            std::process::Command::new("pkill")
                .arg("-f")
                .arg(&format!("{} --tray", crate::get_app_name().to_lowercase()))
                .status()
                .ok();
            hbb_common::allow_err!(crate::run_me(vec!["--tray"]));
        }

        #[cfg(windows)]
        crate::privacy_mode::restore_reg_connectivity(true, false);

        #[cfg(any(target_os = "linux", target_os = "windows"))]
        {
            crate::start_server(true, false);
        }

        #[cfg(target_os = "macos")]
        {
            let handler = std::thread::spawn(move || crate::start_server(true, false));
            crate::tray::start_tray();
            // prevent server exit when encountering errors from tray
            hbb_common::allow_err!(handler.join());
        }

        Ok(())
    }
}