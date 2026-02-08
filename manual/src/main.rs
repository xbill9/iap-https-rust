use anyhow::{Context, Result};
use axum::{
    body::Body,
    extract::Request,
    http::StatusCode,
    middleware::{self, Next},
    response::Response,
    Extension,
};
use rmcp::{
    handler::server::{ServerHandler, tool::ToolRouter, wrapper::Parameters},
    model::{ServerCapabilities, ServerInfo},
    schemars, tool, tool_handler, tool_router,
    transport::streamable_http_server::{
        StreamableHttpServerConfig, StreamableHttpService, session::local::LocalSessionManager,
    },
};
use sysinfo::System;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use std::sync::{Arc, LazyLock};

// Google Cloud Dependencies
use google_apikeys2::ApiKeysService;
use yup_oauth2::authenticator::ApplicationDefaultCredentialsTypes;
use yup_oauth2::ApplicationDefaultCredentialsAuthenticator;

#[derive(Clone)]
struct ApiKey(Arc<Option<String>>);

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct IapSystemInfoRequest {}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct DiskUsageRequest {}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct ProcessListRequest {}

static SYSTEM_INFO_SCHEMA: LazyLock<Arc<serde_json::Map<String, serde_json::Value>>> =
    LazyLock::new(|| {
        let settings = schemars::generate::SchemaSettings::draft07();
        let generator = settings.into_generator();
        let schema = generator.into_root_schema_for::<IapSystemInfoRequest>();
        let mut val = serde_json::to_value(schema).unwrap();
        let obj = val.as_object_mut().unwrap();
        obj.remove("$schema");
        Arc::new(obj.clone())
    });

static DISK_USAGE_SCHEMA: LazyLock<Arc<serde_json::Map<String, serde_json::Value>>> =
    LazyLock::new(|| {
        let settings = schemars::generate::SchemaSettings::draft07();
        let generator = settings.into_generator();
        let schema = generator.into_root_schema_for::<DiskUsageRequest>();
        let mut val = serde_json::to_value(schema).unwrap();
        let obj = val.as_object_mut().unwrap();
        obj.remove("$schema");
        Arc::new(obj.clone())
    });

static PROCESS_LIST_SCHEMA: LazyLock<Arc<serde_json::Map<String, serde_json::Value>>> =
    LazyLock::new(|| {
        let settings = schemars::generate::SchemaSettings::draft07();
        let generator = settings.into_generator();
        let schema = generator.into_root_schema_for::<ProcessListRequest>();
        let mut val = serde_json::to_value(schema).unwrap();
        let obj = val.as_object_mut().unwrap();
        obj.remove("$schema");
        Arc::new(obj.clone())
    });

#[derive(Clone)]
struct SysUtils {
    tool_router: ToolRouter<Self>,
}

async fn fetch_mcp_api_key(project_id: &str) -> Result<String> {
    tracing::info!("Fetching MCP API Key for project: {}", project_id);

    // 1. Create the API Client first (so we can use it for auth)
    let client = hyper_util::client::legacy::Client::builder(hyper_util::rt::TokioExecutor::new())
        .build(
            hyper_rustls::HttpsConnectorBuilder::new()
                .with_native_roots()
                .unwrap()
                .https_or_http()
                .enable_http1()
                .build(),
        );

    // 2. Authenticate using Application Default Credentials
    let opts = yup_oauth2::ApplicationDefaultCredentialsFlowOpts::default();
    let auth_builder = ApplicationDefaultCredentialsAuthenticator::builder(opts).await;
    
    let auth: yup_oauth2::authenticator::Authenticator<_> = match auth_builder {
        ApplicationDefaultCredentialsTypes::InstanceMetadata(builder) => builder.build().await.context("Failed to build InstanceMetadata authenticator")?,
        ApplicationDefaultCredentialsTypes::ServiceAccount(builder) => builder.build().await.context("Failed to build ServiceAccount authenticator")?,
    };

    let hub = ApiKeysService::new(client, auth);

    // 3. List keys to find the one named "MCP API Key"
    // The parent should be "projects/{project_id}/locations/global"
    let parent = format!("projects/{}/locations/global", project_id);
    
    // Attempting flat structure first as it's common in some generated libs
    // If this fails, we will try nested: hub.projects().locations().keys().list(...)
    // Checking previous error suggests trying one way.
    // The crate google-apikeys2 usually has `projects()` returning a struct with methods.
    
    let response = hub
        .projects()
        .locations_keys_list(&parent)
        .doit()
        .await
        .context("Failed to list API keys")?;

    let keys = response.1.keys.context("No keys found in project")?;
    
    let target_key = keys
        .into_iter()
        .find(|k| k.display_name.as_deref() == Some("MCP API Key"))
        .context("MCP API Key not found")?;

    let key_name = target_key.name.context("Key has no name")?;
    tracing::info!("Found MCP API Key resource: {}", key_name);

    // 4. Get the key string (the secret)
    let response = hub
        .projects()
        .locations_keys_get_key_string(&key_name)
        .doit()
        .await
        .context("Failed to get key string")?;

    let key_string = response.1.key_string.context("Response contained no key string")?;
    
    Ok(key_string)
}

fn collect_system_info() -> String {
    let mut sys = System::new_all();
    sys.refresh_all();

    let mut report = String::new();

    report.push_str("System Information Report\n");
    report.push_str("=========================\n\n");

    // System name and kernel
    report.push_str("System Information\n");
    report.push_str("------------------\n");
    report.push_str(&format!(
        "System Name:      {}\n",
        System::name().unwrap_or_else(|| "<unknown>".to_string())
    ));
    report.push_str(&format!(
        "Kernel Version:   {}\n",
        System::kernel_version().unwrap_or_else(|| "<unknown>".to_string())
    ));
    report.push_str(&format!(
        "OS Version:       {}\n",
        System::os_version().unwrap_or_else(|| "<unknown>".to_string())
    ));
    report.push_str(&format!(
        "Host Name:        {}\n",
        System::host_name().unwrap_or_else(|| "<unknown>".to_string())
    ));

    report.push_str("\nCPU Information\n");
    report.push_str("---------------\n");
    report.push_str(&format!("Number of Cores:  {}\n", sys.cpus().len()));

    report.push_str("\nMemory Information\n");
    report.push_str("------------------\n");
    report.push_str(&format!(
        "Total Memory:     {} MB\n",
        sys.total_memory() / 1024 / 1024
    ));
    report.push_str(&format!(
        "Used Memory:      {} MB\n",
        sys.used_memory() / 1024 / 1024
    ));
    report.push_str(&format!(
        "Total Swap:       {} MB\n",
        sys.total_swap() / 1024 / 1024
    ));
    report.push_str(&format!(
        "Used Swap:        {} MB\n",
        sys.used_swap() / 1024 / 1024
    ));

    report.push_str("\nNetwork Interfaces\n");
    report.push_str("------------------\n");
    let networks = sysinfo::Networks::new_with_refreshed_list();
    for (interface_name, network) in &networks {
        report.push_str(&format!(
            "{:<18}: RX: {:>10} bytes, TX: {:>10} bytes (MAC: {})\n",
            interface_name,
            network.total_received(),
            network.total_transmitted(),
            network.mac_address()
        ));
    }

    report
}

#[tool_router]
impl SysUtils {
    fn new() -> Self {
        Self {
            tool_router: Self::tool_router(),
        }
    }

    #[tool(
        description = "Get a detailed system information report including kernel, cores, and memory usage.",
        input_schema = "SYSTEM_INFO_SCHEMA.clone()"
    )]
    async fn sysutils_manual_rust(&self, _params: Parameters<IapSystemInfoRequest>) -> String {
        collect_system_info()
    }

    #[tool(
        description = "Get disk usage information for all mounted disks.",
        input_schema = "DISK_USAGE_SCHEMA.clone()"
    )]
    async fn disk_usage(&self, _params: Parameters<DiskUsageRequest>) -> String {
        let disks = sysinfo::Disks::new_with_refreshed_list();

        let mut report = String::new();
        report.push_str("Disk Usage Report\n");
        report.push_str("=================\n\n");

        for disk in &disks {
            let total = disk.total_space();
            let available = disk.available_space();
            let used = total - available;
            let usage_pct = if total > 0 {
                (used as f64 / total as f64) * 100.0
            } else {
                0.0
            };

            report.push_str(&format!(
                "{:<20} {:<10} {:>10} / {:>10} MB used ({:.1}%)\n",
                disk.mount_point().to_string_lossy(),
                disk.file_system().to_string_lossy(),
                used / 1024 / 1024,
                total / 1024 / 1024,
                usage_pct
            ));
        }

        report
    }

    #[tool(
        description = "List all running processes and their memory usage.",
        input_schema = "PROCESS_LIST_SCHEMA.clone()"
    )]
    async fn list_processes(&self, _params: Parameters<ProcessListRequest>) -> String {
        let mut sys = System::new_all();
        sys.refresh_processes(sysinfo::ProcessesToUpdate::All, true);

        let mut report = String::new();
        report.push_str("Process List Report\n");
        report.push_str("===================\n\n");
        report.push_str(&format!(
            "{:<10} {:<20} {:>12}\n",
            "PID", "Name", "Memory (KB)"
        ));
        report.push_str("------------------------------------------\n");

        let mut processes: Vec<_> = sys.processes().values().collect();
        processes.sort_by_key(|p| p.memory());
        processes.reverse();

        // Show top 20 processes by memory usage
        for process in processes.iter().take(20) {
            report.push_str(&format!(
                "{:<10} {:<20} {:>12}\n",
                process.pid().to_string(),
                process.name().to_string_lossy(),
                process.memory() / 1024
            ));
        }

        report
    }
}

#[tool_handler]
impl ServerHandler for SysUtils {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            instructions: Some(
                "A system utilities MCP that provides detailed system information.".into(),
            ),
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            ..Default::default()
        }
    }
}

async fn iap_middleware(
    Extension(expected_key): Extension<ApiKey>,
    req: Request<Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    // 1. Capture IAP JWT (optional but good for logging/context)
    if let Some(jwt) = req.headers().get("x-goog-iap-jwt-assertion") {
        if let Ok(jwt_str) = jwt.to_str() {
            tracing::debug!("IAP JWT found: {}", jwt_str);
        }
    }

    // 2. Validate API Key if set
    if let Some(expected_key) = expected_key.0.as_ref() {
        let provided_key = req
            .headers()
            .get("x-goog-api-key")
            .and_then(|h| h.to_str().ok());

        if provided_key != Some(expected_key) {
            tracing::warn!("Unauthorized: Invalid or missing x-goog-api-key");
            return Err(StatusCode::UNAUTHORIZED);
        }
    }

    Ok(next.run(req).await)
}

#[tokio::main]
async fn main() -> Result<()> {
    // Check for CLI arguments
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 {
        if args[1] == "info" {
            println!("{}", collect_system_info());
            return Ok(());
        } else if args[1] == "disk" {
            let sysutils = SysUtils::new();
            println!(
                "{}",
                sysutils.disk_usage(Parameters(DiskUsageRequest {})).await
            );
            return Ok(());
        } else if args[1] == "processes" {
            let sysutils = SysUtils::new();
            println!(
                "{}",
                sysutils.list_processes(Parameters(ProcessListRequest {})).await
            );
            return Ok(());
        }
    }

    // Initialize tracing subscriber for logging
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,manual_https_rust=debug".into()),
        )
        .with(
            tracing_subscriber::fmt::layer()
                .with_writer(std::io::stderr)
                .json(),
        )
        .init();

    // Fetch MCP API Key
    // Hardcoded project ID for demonstration; in production this should be from env or metadata
    let project_id = "1056842563084";
    let fetched_key = match fetch_mcp_api_key(project_id).await {
        Ok(key) => {
            tracing::info!("Successfully fetched MCP API Key from Cloud API Keys");
            Some(key)
        }
        Err(e) => {
            tracing::error!("Failed to fetch MCP API Key: {:?}", e);
            None
        }
    };

    // Prefer environment variable if set, otherwise use fetched key
    let mcp_api_key = std::env::var("MCP_API_KEY").ok().or(fetched_key);
    let api_key_state = ApiKey(Arc::new(mcp_api_key));

    let service_factory = || Ok(SysUtils::new());
    let session_manager = LocalSessionManager::default();
    let config = StreamableHttpServerConfig::default();

    let service = StreamableHttpService::new(service_factory, session_manager.into(), config);

    // Add a specific health check route and apply IAP middleware
    let app = axum::Router::new()
        .fallback_service(service)
        .route("/health", axum::routing::get(|| async { "ok" }))
        .layer(middleware::from_fn(iap_middleware))
        .layer(Extension(api_key_state));

    // Determine port from environment variable (Cloud Run standard)
    let port = std::env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let addr = format!("0.0.0.0:{}", port);
    
    tracing::info!("Starting MCP server on {}", addr);
    let listener = tokio::net::TcpListener::bind(&addr).await?;

    tracing::info!("MCP Server listening on http://{}", addr);

    // Run with graceful shutdown
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}

/// Handles graceful shutdown for SIGINT and SIGTERM
async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    tracing::info!("Signal received, starting graceful shutdown...");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_schema_generation() {
        println!(
            "SCHEMA: {}",
            serde_json::to_string_pretty(&*SYSTEM_INFO_SCHEMA).unwrap()
        );
    }

    #[tokio::test]
    async fn test_sysutils_manual_rust() {
        let sysutils = SysUtils::new();
        let report = sysutils
            .sysutils_manual_rust(Parameters(IapSystemInfoRequest {}))
            .await;
        assert!(report.contains("System Information Report"));
        assert!(report.contains("CPU Information"));
        assert!(report.contains("Network Interfaces"));
        assert!(!report.contains("Disk Information"));
    }

    #[tokio::test]
    async fn test_disk_usage() {
        let sysutils = SysUtils::new();
        let report = sysutils
            .disk_usage(Parameters(DiskUsageRequest {}))
            .await;
        assert!(report.contains("Disk Usage Report"));
    }

    #[tokio::test]
    async fn test_list_processes() {
        let sysutils = SysUtils::new();
        let report = sysutils
            .list_processes(Parameters(ProcessListRequest {}))
            .await;
        assert!(report.contains("Process List Report"));
        assert!(report.contains("PID"));
    }
}
