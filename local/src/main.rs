use anyhow::{Context, Result};
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

use std::sync::{Arc, LazyLock, OnceLock};
use std::fmt::Write;

// Google Cloud Dependencies
use google_apikeys2::ApiKeysService;
use yup_oauth2::authenticator::ApplicationDefaultCredentialsTypes;
use yup_oauth2::ApplicationDefaultCredentialsAuthenticator;

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct IapSystemInfoRequest {}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct DiskUsageRequest {}

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

static EXPECTED_API_KEY: OnceLock<Option<String>> = OnceLock::new();

async fn fetch_mcp_api_key(project_id: &str) -> Result<String> {
    tracing::info!("Fetching MCP API Key for project: {}", project_id);

    // Try gcloud first for local development, it's more reliable with User ADC
    match fetch_mcp_api_key_gcloud(project_id).await {
        Ok(key) => {
            tracing::info!("Successfully fetched API key via gcloud");
            return Ok(key);
        }
        Err(e) => {
            tracing::debug!("gcloud fetch failed (expected if gcloud not installed): {}", e);
        }
    }

    // Fallback to library-based approach (works in Cloud Run/GCE with Service Accounts)
    fetch_mcp_api_key_library(project_id).await
}

async fn fetch_mcp_api_key_gcloud(project_id: &str) -> Result<String> {
    let output = tokio::process::Command::new("gcloud")
        .args([
            "services",
            "api-keys",
            "list",
            &format!("--project={}", project_id),
            "--filter=displayName='MCP API Key'",
            "--format=value(name)",
        ])
        .output()
        .await
        .context("Failed to execute gcloud command")?;

    if !output.status.success() {
        return Err(anyhow::anyhow!(
            "gcloud list failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    let key_name = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if key_name.is_empty() {
        return Err(anyhow::anyhow!("MCP API Key not found via gcloud"));
    }

    let output = tokio::process::Command::new("gcloud")
        .args([
            "services",
            "api-keys",
            "get-key-string",
            &key_name,
            &format!("--project={}", project_id),
            "--format=value(keyString)",
        ])
        .output()
        .await
        .context("Failed to execute gcloud get-key-string")?;

    if !output.status.success() {
        return Err(anyhow::anyhow!(
            "gcloud get-key-string failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

async fn fetch_mcp_api_key_library(project_id: &str) -> Result<String> {
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
        ApplicationDefaultCredentialsTypes::InstanceMetadata(builder) => builder
            .build()
            .await
            .context("Failed to build InstanceMetadata authenticator")?,
        ApplicationDefaultCredentialsTypes::ServiceAccount(builder) => builder
            .build()
            .await
            .context("Failed to build ServiceAccount authenticator")?,
    };

    let hub = ApiKeysService::new(client, auth);

    // 3. List keys to find the one named "MCP API Key"
    let parent = format!("projects/{}/locations/global", project_id);

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

    let key_string = response
        .1
        .key_string
        .context("Response contained no key string")?;

    Ok(key_string)
}

#[derive(Clone)]
struct SysUtils {
    tool_router: ToolRouter<Self>,
}

fn collect_system_info(api_status: Option<&str>) -> String {
    let mut sys = System::new_all();
    sys.refresh_all();

    let mut report = String::new();

    let _ = writeln!(report, "System Information Report");
    let _ = writeln!(report, "=========================\n");

    if let Some(status) = api_status {
        let _ = writeln!(report, "{}", status);
    }

    // IAP Information
    let _ = writeln!(report, "IAP Context & Identity");
    let _ = writeln!(report, "----------------------");
    let _ = writeln!(report, "Header Source:    x-goog-iap-jwt-assertion");
    let api_key_presence = if EXPECTED_API_KEY.get().and_then(|k| k.as_ref()).is_some() {
        "Enabled (MCP_API_KEY set)"
    } else {
        "Disabled"
    };
    let _ = writeln!(report, "API Key Security: {}", api_key_presence);

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

async fn check_api_key_status(args: &[String]) -> (String, bool) {
    let mut status = String::new();
    let mut success = true;
    let _ = writeln!(status, "MCP API Key Status");
    let _ = writeln!(status, "------------------");

    let mut provided_key = std::env::var("MCP_API_KEY").ok();
    if provided_key.is_none() {
        for i in 1..args.len() {
            if args[i] == "--key" && i + 1 < args.len() {
                provided_key = Some(args[i + 1].clone());
                break;
            }
        }
    }

    if let Some(key) = provided_key {
        let _ = writeln!(status, "Provided Key:     [FOUND]");
        // Fetch cloud key
        let project_id = "1056842563084";
        match fetch_mcp_api_key(project_id).await {
            Ok(expected_key) => {
                if key == expected_key {
                    let _ = writeln!(status, "Cloud Match:      [MATCHED]");
                } else {
                    let _ = writeln!(status, "Cloud Match:      [MISMATCH]");
                    success = false;
                }
            }
            Err(e) => {
                let _ = writeln!(status, "Cloud Match:      [ERROR: {:?}]", e);
                success = false;
            }
        }
    } else {
        let _ = writeln!(status, "Provided Key:     [NOT FOUND]");
        success = false;
    }
    status.push('\n');
    (status, success)
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
    async fn local_system_info(&self, _params: Parameters<IapSystemInfoRequest>) -> String {
        collect_system_info(Some("Authentication:   [VERIFIED] (Running as MCP Server)\n"))
    }

    #[tool(
        description = "Get disk usage information for all mounted disks.",
        input_schema = "DISK_USAGE_SCHEMA.clone()"
    )]
    async fn disk_usage(&self, _params: Parameters<DiskUsageRequest>) -> String {
        collect_disk_usage()
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
    use axum::response::IntoResponse;

    if let Some(expected_key) = EXPECTED_API_KEY.get().and_then(|k| k.as_ref()) {
        let api_key_header = request
            .headers()
            .get("x-goog-api-key")
            .and_then(|h| h.to_str().ok());

        let api_key_query = request.uri().query().and_then(|q| {
            q.split('&')
                .find(|p| p.starts_with("key="))
                .and_then(|p| p.get(4..))
        });

        if api_key_header != Some(expected_key) && api_key_query != Some(expected_key) {
            tracing::warn!("Unauthorized request: invalid or missing API Key (checked header and ?key=)");
            return (axum::http::StatusCode::UNAUTHORIZED, "Unauthorized").into_response();
        }
        tracing::debug!("API Key verified successfully");
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
    // Check for CLI arguments
    let args: Vec<String> = std::env::args().collect();

    // Initialize tracing subscriber for logging
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,sysutils_local_rust=debug".into()),
        )
        .with(
            tracing_subscriber::fmt::layer()
                .with_writer(std::io::stderr)
                .json(),
        )
        .init();

    if args.iter().any(|arg| arg == "info") {
        let (api_status, success) = check_api_key_status(&args).await;
        if !success {
            eprintln!("{}", api_status);
            eprintln!("Error: MCP_API_KEY is incorrect or missing.");
            std::process::exit(1);
        }
        println!("{}", collect_system_info(Some(&api_status)));
        return Ok(());
    } else if args.iter().any(|arg| arg == "disk") {
        println!("{}", collect_disk_usage());
        return Ok(());
    }

    // Initialize EXPECTED_API_KEY for the server
    let project_id = "1056842563084";
    let cloud_key = match fetch_mcp_api_key(project_id).await {
        Ok(key) => {
            tracing::info!("Successfully fetched MCP API Key from Cloud");
            Some(key)
        }
        Err(e) => {
            tracing::warn!("Failed to fetch MCP API Key from Cloud: {:?}. Checking environment variable.", e);
            std::env::var("MCP_API_KEY").ok()
        }
    };

    let cloud_key = cloud_key.context("MCP_API_KEY not found in Cloud or environment. Server requires an API key.")?;
    EXPECTED_API_KEY.set(Some(cloud_key)).ok();

    let service_factory = || Ok(SysUtils::new());
    let session_manager = LocalSessionManager::default();
    let config = StreamableHttpServerConfig::default();

    let service = StreamableHttpService::new(service_factory, session_manager.into(), config);

    // Add a specific health check route and IAP middleware
    let app = axum::Router::new()
        .route("/health", axum::routing::get(|| async { "ok" }))
        .fallback_service(service)
        .layer(axum::middleware::from_fn(iap_middleware));

    // Determine port from environment variable (Cloud Run standard)
    let port = std::env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let addr = format!("0.0.0.0:{}", port);
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
    async fn test_local_system_info() {
        let sysutils = SysUtils::new();
        let report = sysutils
            .local_system_info(Parameters(IapSystemInfoRequest {}))
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
                IAP_CONTEXT.scope(Some(ctx), async { collect_system_info(None) }),
            )
            .await;

        assert!(report.contains("email             : user@example.com"));
        assert!(report.contains("custom_field      : custom_value"));
        assert!(report.contains("user-agent        : test-agent"));
    }
}
