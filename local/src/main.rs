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

use std::sync::{Arc, LazyLock};

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct IapSystemInfoRequest {}

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

static EXPECTED_API_KEY: LazyLock<Option<String>> =
    LazyLock::new(|| std::env::var("MCP_API_KEY").ok());

#[derive(Clone)]
struct SysUtils {
    tool_router: ToolRouter<Self>,
}

fn collect_system_info() -> String {
    let mut sys = System::new_all();
    sys.refresh_all();

    let mut report = String::new();

    report.push_str("System Information Report\n");
    report.push_str("=========================\n\n");

    // IAP Information
    report.push_str("IAP Context & Identity\n");
    report.push_str("----------------------\n");
    report.push_str("Header Source:    x-goog-iap-jwt-assertion\n");
    let api_key_status = if EXPECTED_API_KEY.is_some() {
        "Enabled (MCP_API_KEY set)"
    } else {
        "Disabled"
    };
    report.push_str(&format!("API Key Security: {}\n", api_key_status));

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
                report.push_str(&format!("{:<18}: {}\n", key, val_str));
            }
        } else {
            report.push_str(&format!("Payload:          {}\n", ctx.payload));
        }
    } else {
        report.push_str(
            "Status:           No IAP JWT found (Expected in production Cloud Run environment)\n",
        );
    }
    report.push('\n');

    // Request Headers
    report.push_str("HTTP Request Headers\n");
    report.push_str("--------------------\n");
    let headers = REQUEST_HEADERS.try_with(|h| h.clone()).ok();
    if let Some(h) = headers {
        for (name, value) in h {
            report.push_str(&format!("{:<18}: {}\n", name, value));
        }
    } else {
        report.push_str(
            "Status:           No request headers captured (CLI mode or capture error)\n",
        );
    }
    report.push('\n');

    report.push_str("IAP Setup Configuration\n");
    report.push_str("-----------------------\n");
    let mut found_config = false;
    for file in &[
        "iap_settings.yaml",
        "iap_service_settings.yaml",
        "iap_programmatic_settings.yaml",
    ] {
        if let Ok(content) = std::fs::read_to_string(file) {
            found_config = true;
            report.push_str(&format!("[{}]\n", file));
            report.push_str(&content);
            if !content.ends_with('\n') {
                report.push('\n');
            }
        }
    }
    if !found_config {
        report
            .push_str("Status:           No IAP configuration files found in current directory.\n");
    }
    report.push('\n');

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
    async fn iap_system_info(&self, _params: Parameters<IapSystemInfoRequest>) -> String {
        collect_system_info()
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

    if let Some(expected_key) = &*EXPECTED_API_KEY {
        let api_key_header = request
            .headers()
            .get("x-goog-api-key")
            .and_then(|h| h.to_str().ok());

        if api_key_header != Some(expected_key) {
            tracing::warn!("Unauthorized request: invalid or missing X-Goog-Api-Key");
            return (axum::http::StatusCode::UNAUTHORIZED, "Unauthorized").into_response();
        }
        tracing::debug!("X-Goog-Api-Key verified successfully");
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
    if args.len() > 1 && args[1] == "info" {
        println!("{}", collect_system_info());
        return Ok(());
    }

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
    async fn test_iap_system_info() {
        let sysutils = SysUtils::new();
        let report = sysutils
            .iap_system_info(Parameters(IapSystemInfoRequest {}))
            .await;
        assert!(report.contains("System Information Report"));
        assert!(report.contains("CPU Information"));
        assert!(!report.contains("Disk Information"));
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
                IAP_CONTEXT.scope(Some(ctx), async { collect_system_info() }),
            )
            .await;

        assert!(report.contains("email             : user@example.com"));
        assert!(report.contains("custom_field      : custom_value"));
        assert!(report.contains("user-agent        : test-agent"));
    }
}
