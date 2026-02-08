use anyhow::{Context, Result};
use rmcp::{
    handler::server::{ServerHandler, tool::ToolRouter, wrapper::Parameters},
    model::{ServerCapabilities, ServerInfo},
    schemars::{self, JsonSchema},
    tool, tool_handler, tool_router,
    transport, ServiceExt,
};
use sysinfo::{Disks, Networks, System};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use std::{
    fmt::Write,
    sync::{Arc, LazyLock},
};

// Google Cloud Dependencies
use google_apikeys2::ApiKeysService;
use yup_oauth2::authenticator::ApplicationDefaultCredentialsTypes;
use yup_oauth2::ApplicationDefaultCredentialsAuthenticator;

#[derive(Debug, serde::Deserialize, JsonSchema)]
struct SystemInfoRequest {}

#[derive(Debug, serde::Deserialize, JsonSchema)]
struct DiskUsageRequest {}

fn generate_schema<T: JsonSchema>() -> Arc<serde_json::Map<String, serde_json::Value>> {
    let settings = schemars::generate::SchemaSettings::draft07();
    let generator = settings.into_generator();
    let schema = generator.into_root_schema_for::<T>();
    let mut val = serde_json::to_value(schema).expect("Schema serialization failed");
    if let Some(obj) = val.as_object_mut() {
        obj.remove("$schema");
        Arc::new(obj.clone())
    } else {
        // Fallback for unexpected schema structure
        Arc::new(serde_json::Map::new())
    }
}

static SYSTEM_INFO_SCHEMA: LazyLock<Arc<serde_json::Map<String, serde_json::Value>>> = 
    LazyLock::new(generate_schema::<SystemInfoRequest>);

static DISK_USAGE_SCHEMA: LazyLock<Arc<serde_json::Map<String, serde_json::Value>>> = 
    LazyLock::new(generate_schema::<DiskUsageRequest>);

#[derive(Clone)]
struct SysUtils {
    tool_router: ToolRouter<Self>,
}

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

fn collect_system_info(api_status: Option<&str>) -> String {
    let mut sys = System::new_all();
    sys.refresh_all();

    let mut report = String::new();

    let _ = writeln!(report, "System Information Report");
    let _ = writeln!(report, "=========================\n");

    if let Some(status) = api_status {
        let _ = writeln!(report, "{}", status);
    }

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
    let networks = Networks::new_with_refreshed_list();
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
    let disks = Disks::new_with_refreshed_list();

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
    async fn local_system_info(&self, _params: Parameters<SystemInfoRequest>) -> String {
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

#[tokio::main]
async fn main() {
    // Collect CLI arguments
    let args: Vec<String> = std::env::args().collect();

    // Initialize tracing subscriber for logging
    // IMPORTANT: Stdio transport uses stdout for JSON-RPC, so logs MUST go to stderr.
    // We use JSON format for structured logging.
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,sysutils_stdiokey_rust=debug".into()),
        )
        .with(
            tracing_subscriber::fmt::layer()
                .with_writer(std::io::stderr)
                .with_ansi(false) // Ensure no ANSI codes in JSON
                .json(),
        )
        .init();

    if let Err(e) = handle_main(args).await {
        tracing::error!(error = ?e, "Application failed");
        std::process::exit(1);
    }
}

async fn check_api_key_status(args: &[String]) -> String {
    let mut status = String::new();
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
                }
            }
            Err(e) => {
                let _ = writeln!(status, "Cloud Match:      [ERROR: {:?}]", e);
            }
        }
    } else {
        let _ = writeln!(status, "Provided Key:     [NOT FOUND]");
    }
    status.push('\n');
    status
}

async fn handle_main(args: Vec<String>) -> Result<()> {
    // Check for CLI arguments for direct execution FIRST
    if args.iter().any(|arg| arg == "info") {
        let api_status = check_api_key_status(&args).await;
        println!("{}", collect_system_info(Some(&api_status)));
        return Ok(());
    } else if args.iter().any(|arg| arg == "disk") {
        println!("{}", collect_disk_usage());
        return Ok(());
    }

    // Key Verification Logic (Presence Check)
    let mut provided_key = std::env::var("MCP_API_KEY").ok();

    if provided_key.is_none() {
        for i in 1..args.len() {
            if args[i] == "--key" && i + 1 < args.len() {
                provided_key = Some(args[i + 1].clone());
                break;
            }
        }
    }

    if provided_key.is_none() {
        return Err(anyhow::anyhow!("Authentication Required: Please provide the API Key using --key <KEY> or MCP_API_KEY environment variable"));
    }

    // Fetch MCP API Key and Verify
    // Hardcoded project ID matching the manual variant
    let project_id = "1056842563084";
    let expected_key = fetch_mcp_api_key(project_id).await
        .context("Failed to fetch MCP API Key")?;

    if provided_key.as_ref() != Some(&expected_key) {
        return Err(anyhow::anyhow!("Authentication Failed: Invalid API Key provided"));
    }

    tracing::info!("Authentication Successful");

    tracing::info!("Starting stdiokey MCP Stdio server");

    run_server().await.context("MCP server encountered a fatal error")?;

    Ok(())
}

async fn run_server() -> Result<()> {
    let service = SysUtils::new().serve(transport::stdio()).await?;
    service.waiting().await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_schema_generation() {
        // This test writes to stdout, which might interfere with cargo test output capture 
        // if not handled, but usually cargo test captures stdout.
        let schema = serde_json::to_string_pretty(&*SYSTEM_INFO_SCHEMA).unwrap();
        assert!(schema.len() > 0);
    }

    #[tokio::test]
    async fn test_local_system_info() {
        let sysutils = SysUtils::new();
        let report = sysutils
            .local_system_info(Parameters(SystemInfoRequest {}))
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
}