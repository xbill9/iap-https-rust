use anyhow::Result;
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use rmcp::{
    handler::server::{ServerHandler, tool::ToolRouter, wrapper::Parameters},
    model::{ServerCapabilities, ServerInfo},
    schemars, tool, tool_handler, tool_router,
    transport::streamable_http_server::{
        StreamableHttpServerConfig, StreamableHttpService, session::local::LocalSessionManager,
    },
};
use serde_json::Value;
use sysinfo::System;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use std::fmt::Write;
use std::sync::{Arc, LazyLock};

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct IapSystemInfoRequest {}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct DiskUsageRequest {}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct ProcessListRequest {}

#[derive(Debug, Clone, serde::Serialize)]
struct IapContext {
    payload: Value,
}

tokio::task_local! {
    static IAP_CONTEXT: Option<IapContext>;
    static REQUEST_HEADERS: Vec<(String, String)>;
}

fn decode_iap_jwt(jwt: &str) -> Option<IapContext> {
    let parts: Vec<&str> = jwt.split('.').collect();
    if parts.len() != 3 {
        return None;
    }
    let payload_b64 = parts[1];
    let decoded = URL_SAFE_NO_PAD.decode(payload_b64).ok()?;
    let payload: Value = serde_json::from_slice(&decoded).ok()?;

    Some(IapContext { payload })
}

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

async fn collect_system_info() -> String {
    let mut sys = System::new_all();
    sys.refresh_all();

    let mut report = String::new();

    let _ = writeln!(report, "System Information Report");
    let _ = writeln!(report, "=========================\n");

    // IAP Information
    let _ = writeln!(report, "IAP Context & Identity");
    let _ = writeln!(report, "----------------------");
    let _ = writeln!(report, "Header Source:    x-goog-iap-jwt-assertion");

    let iap_ctx = IAP_CONTEXT.try_with(|ctx| ctx.clone()).ok().flatten();
    if let Some(ctx) = iap_ctx {
        if let Some(obj) = ctx.payload.as_object() {
            for (key, value) in obj {
                let val_str = match value {
                    Value::String(s) => s.clone(),
                    Value::Number(n) => n.to_string(),
                    Value::Bool(b) => b.to_string(),
                    _ => value.to_string(),
                };
                let _ = writeln!(report, "{:<18}: {}", key, val_str);
            }
        } else {
            let _ = writeln!(report, "Payload:          {}", ctx.payload);
        }
    } else {
        let _ = writeln!(
            report,
            "Status:           No IAP JWT found (Expected in production Cloud Run environment)"
        );
    }
    report.push('\n');

    // Request Headers
    let _ = writeln!(report, "HTTP Request Headers");
    let _ = writeln!(report, "--------------------");
    let headers = REQUEST_HEADERS.try_with(|h| h.clone()).ok();
    if let Some(h) = headers {
        for (name, value) in h {
            let _ = writeln!(report, "{:<18}: {}", name, value);
        }
    } else {
        let _ = writeln!(
            report,
            "Status:           No request headers captured (CLI mode or capture error)"
        );
    }
    report.push('\n');

    let _ = writeln!(report, "IAP Setup Configuration");
    let _ = writeln!(report, "-----------------------");
    let mut found_config = false;
    for file in &[
        "iap_settings.yaml",
        "iap_service_settings.yaml",
        "iap_programmatic_settings.yaml",
    ] {
        if let Ok(content) = std::fs::read_to_string(file) {
            found_config = true;
            let _ = writeln!(report, "[{}]", file);
            report.push_str(&content);
            if !content.ends_with('\n') {
                report.push('\n');
            }
        }
    }
    if !found_config {
        let _ = writeln!(
            report,
            "Status:           No IAP configuration files found in current directory."
        );
    }
    report.push('\n');

    // System name and kernel
    let _ = writeln!(report, "System Information");
    let _ = writeln!(report, "------------------");
    let _ = writeln!(
        report,
        "System Name:      {}",
        System::name().unwrap_or_else(|| "<unknown>".to_string())
    );
    let _ = writeln!(
        report,
        "Kernel Version:   {}",
        System::kernel_version().unwrap_or_else(|| "<unknown>".to_string())
    );
    let _ = writeln!(
        report,
        "OS Version:       {}",
        System::os_version().unwrap_or_else(|| "<unknown>".to_string())
    );
    let _ = writeln!(
        report,
        "Host Name:        {}",
        System::host_name().unwrap_or_else(|| "<unknown>".to_string())
    );

    let _ = writeln!(report, "\nCPU Information");
    let _ = writeln!(report, "---------------");
    let _ = writeln!(report, "Number of Cores:  {}", sys.cpus().len());

    let _ = writeln!(report, "\nMemory Information");
    let _ = writeln!(report, "------------------");
    let _ = writeln!(
        report,
        "Total Memory:     {} MB",
        sys.total_memory() / 1024 / 1024
    );
    let _ = writeln!(
        report,
        "Used Memory:      {} MB",
        sys.used_memory() / 1024 / 1024
    );
    let _ = writeln!(
        report,
        "Total Swap:       {} MB",
        sys.total_swap() / 1024 / 1024
    );
    let _ = writeln!(
        report,
        "Used Swap:        {} MB",
        sys.used_swap() / 1024 / 1024
    );

    let _ = writeln!(report, "\nNetwork Interfaces");
    let _ = writeln!(report, "------------------");
    let networks = sysinfo::Networks::new_with_refreshed_list();
    for (interface_name, network) in &networks {
        let _ = writeln!(
            report,
            "{:<18}: RX: {:>10} bytes, TX: {:>10} bytes (MAC: {})",
            interface_name,
            network.total_received(),
            network.total_transmitted(),
            network.mac_address()
        );
    }

    report
}

fn collect_disk_usage() -> String {
    let disks = sysinfo::Disks::new_with_refreshed_list();

    let mut report = String::new();
    let _ = writeln!(report, "Disk Usage Report");
    let _ = writeln!(report, "=================\n");

    for disk in &disks {
        let total = disk.total_space();
        let available = disk.available_space();
        let used = total - available;
        let usage_pct = if total > 0 {
            (used as f64 / total as f64) * 100.0
        } else {
            0.0
        };

        let _ = writeln!(
            report,
            "{:<20} {:<10} {:>10} / {:>10} MB used ({:.1}%)",
            disk.mount_point().to_string_lossy(),
            disk.file_system().to_string_lossy(),
            used / 1024 / 1024,
            total / 1024 / 1024,
            usage_pct
        );
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
    async fn sysutils_bearer_rust(&self, _params: Parameters<IapSystemInfoRequest>) -> String {
        collect_system_info().await
    }

    #[tool(
        description = "Get disk usage information for all mounted disks.",
        input_schema = "DISK_USAGE_SCHEMA.clone()"
    )]
    async fn disk_usage(&self, _params: Parameters<DiskUsageRequest>) -> String {
        collect_disk_usage()
    }

    #[tool(
        description = "List all running processes and their memory usage.",
        input_schema = "PROCESS_LIST_SCHEMA.clone()"
    )]
    async fn list_processes(&self, _params: Parameters<ProcessListRequest>) -> String {
        let mut sys = System::new_all();
        sys.refresh_processes(sysinfo::ProcessesToUpdate::All, true);

        let mut report = String::new();
        let _ = writeln!(report, "Process List Report");
        let _ = writeln!(report, "===================\n");
        let _ = writeln!(report, "{:<10} {:<20} {:>12}", "PID", "Name", "Memory (KB)");
        let _ = writeln!(report, "------------------------------------------");

        let mut processes: Vec<_> = sys.processes().values().collect();
        processes.sort_by_key(|p| p.memory());
        processes.reverse();

        // Show top 20 processes by memory usage
        for process in processes.iter().take(20) {
            let _ = writeln!(
                report,
                "{:<10} {:<20} {:>12}",
                process.pid().to_string(),
                process.name().to_string_lossy(),
                process.memory() / 1024
            );
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
    request: axum::extract::Request,
    next: axum::middleware::Next,
) -> axum::response::Response {
    // Skip health endpoint
    if request.uri().path() == "/health" {
        return next.run(request).await;
    }

    let mut headers = Vec::new();
    for (name, value) in request.headers() {
        headers.push((
            name.to_string(),
            value.to_str().unwrap_or("<non-utf8>").to_string(),
        ));
    }

    // Debug: Log all request headers
    tracing::debug!("--- Incoming Request Headers ---");
    for (name, value) in &headers {
        tracing::debug!("{}: {}", name, value);
    }

    let iap_header = request.headers().get("x-goog-iap-jwt-assertion");
    let mut iap_context = None;

    if let Some(header_value) = iap_header {
        tracing::debug!("Found x-goog-iap-jwt-assertion header");
        if let Ok(jwt_str) = header_value.to_str() {
            if let Some(ctx) = decode_iap_jwt(jwt_str) {
                tracing::info!("IAP JWT decoded successfully. Claims: {}", ctx.payload);
                iap_context = Some(ctx);
            } else {
                tracing::error!("Failed to decode x-goog-iap-jwt-assertion payload");
            }
        } else {
            tracing::error!("x-goog-iap-jwt-assertion header contains non-UTF8 data");
        }
    } else {
        tracing::debug!("No x-goog-iap-jwt-assertion header found");
    }

    REQUEST_HEADERS
        .scope(headers, IAP_CONTEXT.scope(iap_context, next.run(request)))
        .await
}

#[tokio::main]
async fn main() -> Result<()> {
    // 1. Determine port and bind immediately to satisfy Cloud Run health check
    let port = std::env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let addr = format!("0.0.0.0:{}", port);
    
    println!("DEBUG: Starting bearer-rust version 0.3.0-debug");
    println!("DEBUG: Environment: PORT={}", port);

    println!("DEBUG: Sleeping for 10s before bind to allow environment to settle");
    tokio::time::sleep(std::time::Duration::from_secs(10)).await;

    println!("DEBUG: Attempting to bind to {}", addr);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    println!("DEBUG: Successfully bound to {}", addr);

    // 2. Initialize tracing AFTER binding
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,bearer_rust=debug".into()),
        )
        .with(
            tracing_subscriber::fmt::layer()
                .with_writer(std::io::stderr)
                .json(),
        )
        .init();

    // 3. Handle CLI arguments
    let args: Vec<String> = std::env::args().collect();
    if args.iter().any(|arg| arg == "info") {
        println!("{}", collect_system_info().await);
        return Ok(());
    } else if args.iter().any(|arg| arg == "disk") {
        println!("{}", collect_disk_usage());
        return Ok(());
    } else if args.iter().any(|arg| arg == "processes") {
        let sysutils = SysUtils::new();
        println!(
            "{}",
            sysutils
                .list_processes(Parameters(ProcessListRequest {}))
                .await
        );
        return Ok(());
    }

    // 4. Setup MCP Service
    let service_factory = || Ok(SysUtils::new());
    let session_manager = LocalSessionManager::default();
    let config = StreamableHttpServerConfig::default();
    let service = StreamableHttpService::new(service_factory, session_manager.into(), config);

    let app = axum::Router::new()
        .route("/health", axum::routing::get(|| async { "ok" }))
        .fallback_service(service)
        .layer(axum::middleware::from_fn(iap_middleware));

    tracing::info!("MCP Server starting on http://{}", addr);

    // 5. Serve
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
    async fn test_sysutils_bearer_rust() {
        let sysutils = SysUtils::new();
        let report = sysutils
            .sysutils_bearer_rust(Parameters(IapSystemInfoRequest {}))
            .await;
        assert!(report.contains("System Information Report"));
        assert!(report.contains("CPU Information"));
        assert!(report.contains("Network Interfaces"));
        assert!(!report.contains("Disk Information"));
    }

    #[tokio::test]
    async fn test_disk_usage() {
        let sysutils = SysUtils::new();
        let report = sysutils.disk_usage(Parameters(DiskUsageRequest {})).await;
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

    #[test]
    fn test_decode_iap_jwt() {
        let payload = serde_json::json!({
            "email": "test@example.com",
            "sub": "12345",
            "aud": "iap-audience",
            "iss": "https://cloud.google.com/iap",
            "custom": "value"
        });
        let payload_str = serde_json::to_string(&payload).unwrap();
        let payload_b64 = URL_SAFE_NO_PAD.encode(payload_str);
        let jwt = format!("header.{}.signature", payload_b64);

        let ctx = decode_iap_jwt(&jwt).unwrap();
        assert_eq!(
            ctx.payload.get("email").unwrap().as_str().unwrap(),
            "test@example.com"
        );
        assert_eq!(
            ctx.payload.get("custom").unwrap().as_str().unwrap(),
            "value"
        );
    }

    #[tokio::test]
    async fn test_collect_system_info_with_context() {
        let payload = serde_json::json!({
            "email": "user@example.com",
            "custom_field": "custom_value"
        });
        let ctx = IapContext { payload };
        let headers = vec![("user-agent".to_string(), "test-agent".to_string())];

        let report = REQUEST_HEADERS
            .scope(
                headers,
                IAP_CONTEXT.scope(Some(ctx), async { collect_system_info().await }),
            )
            .await;

        assert!(report.contains("email             : user@example.com"));
        assert!(report.contains("custom_field      : custom_value"));
        assert!(report.contains("user-agent        : test-agent"));
    }

    #[tokio::test]
    async fn test_health_check() {
        use axum::{
            body::Body,
            http::{Request, StatusCode},
        };
        use tower::ServiceExt;

        // Set up the router
        let service_factory = || Ok(SysUtils::new());
        let session_manager = LocalSessionManager::default();
        let config = StreamableHttpServerConfig::default();
        let service = StreamableHttpService::new(service_factory, session_manager.into(), config);

        let app = axum::Router::new()
            .route("/health", axum::routing::get(|| async { "ok" }))
            .fallback_service(service)
            .layer(axum::middleware::from_fn(iap_middleware));

        // Test /health
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }
}
