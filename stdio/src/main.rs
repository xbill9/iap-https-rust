use anyhow::Result;
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

fn collect_system_info() -> String {
    let mut sys = System::new_all();
    sys.refresh_all();

    let mut report = String::new();

    let _ = writeln!(report, "System Information Report");
    let _ = writeln!(report, "=========================\n");

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
        collect_system_info()
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
async fn main() -> Result<()> {
    // Check for CLI arguments for direct execution
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 {
        if args[1] == "info" {
            println!("{}", collect_system_info());
            return Ok(());
        } else if args[1] == "disk" {
            println!("{}", collect_disk_usage());
            return Ok(());
        }
    }

    // Initialize tracing subscriber for logging
    // IMPORTANT: Stdio transport uses stdout for JSON-RPC, so logs MUST go to stderr.
    // We use JSON format for structured logging.
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,sysutils_local_rust=debug".into()),
        )
        .with(
            tracing_subscriber::fmt::layer()
                .with_writer(std::io::stderr)
                .with_ansi(false) // Ensure no ANSI codes in JSON
                .json(),
        )
        .init();

    tracing::info!("Starting sysutils-local-rust MCP Stdio server");

    if let Err(e) = run_server().await {
        tracing::error!(error = ?e, "MCP server encountered a fatal error");
        std::process::exit(1);
    }

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
        println!(
            "SCHEMA: {}",
            serde_json::to_string_pretty(&*SYSTEM_INFO_SCHEMA).unwrap()
        );
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
