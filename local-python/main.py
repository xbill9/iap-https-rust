import asyncio
import logging
import os
import subprocess
import sys
from typing import Optional

import psutil
from googleapiclient.discovery import build
import google.auth
from mcp.server.fastmcp import FastMCP
from starlette.responses import JSONResponse
from starlette.requests import Request
import uvicorn

# Configure logging to stderr
logging.basicConfig(
    level=logging.INFO,
    format="%(asctime)s - %(name)s - %(levelname)s - %(message)s",
    stream=sys.stderr,
)
logger = logging.getLogger("stdiokey-python")

# Initialize FastMCP
mcp = FastMCP(
    "httpkey-python",
    port=int(os.environ.get("PORT", 8080)),
    host="0.0.0.0",
)


def get_project_id() -> Optional[str]:
    """Get Google Cloud Project ID from environment or credentials."""
    # Check environment variable first
    project_id = os.environ.get("GOOGLE_CLOUD_PROJECT")
    if project_id:
        return project_id

    # Fallback to default credentials
    try:
        _, project_id = google.auth.default()
        return project_id
    except Exception as e:
        logger.debug(f"Failed to get project ID from default credentials: {e}")
        return None


async def fetch_mcp_api_key_gcloud(project_id: str) -> str:
    try:
        # List keys
        result = subprocess.run(
            [
                "gcloud",
                "services",
                "api-keys",
                "list",
                f"--project={project_id}",
                "--filter=displayName='MCP API Key'",
                "--format=value(name)",
            ],
            capture_output=True,
            text=True,
            check=True,
        )
        key_name = result.stdout.strip()
        if not key_name:
            raise Exception("MCP API Key not found via gcloud")

        # Get key string
        result = subprocess.run(
            [
                "gcloud",
                "services",
                "api-keys",
                "get-key-string",
                key_name,
                f"--project={project_id}",
                "--format=value(keyString)",
            ],
            capture_output=True,
            text=True,
            check=True,
        )
        return result.stdout.strip()
    except Exception as e:
        logger.debug(f"gcloud fetch failed: {e}")
        raise


async def fetch_mcp_api_key_library(project_id: str) -> str:
    credentials, _ = google.auth.default()
    service = build("apikeys", "v2", credentials=credentials)

    parent = f"projects/{project_id}/locations/global"
    request = service.projects().locations().keys().list(parent=parent)
    response = request.execute()

    keys = response.get("keys", [])
    target_key = next((k for k in keys if k.get("displayName") == "MCP API Key"), None)

    if not target_key:
        raise Exception("MCP API Key not found")

    key_name = target_key["name"]
    request = service.projects().locations().keys().getKeyString(name=key_name)
    response = request.execute()

    return response.get("keyString", response.get("key_string"))


async def fetch_mcp_api_key(project_id: str) -> str:
    logger.info(f"Fetching MCP API Key for project: {project_id}")
    try:
        key = await fetch_mcp_api_key_gcloud(project_id)
        logger.info("Successfully fetched API key via gcloud")
        return key
    except Exception:
        logger.info("Falling back to library-based API key fetch")
        return await fetch_mcp_api_key_library(project_id)


def collect_system_info(api_status: Optional[str] = None) -> str:
    lines = ["System Information Report", "=========================\n"]

    if api_status:
        lines.append(api_status)

    lines.extend(
        [
            "System Information",
            "------------------",
            f"System Name:      {os.name}",
            f"OS Name:          {sys.platform}",
            f"Host Name:        {subprocess.getoutput('hostname')}",
            "",
        ]
    )

    lines.extend(
        [
            "CPU Information",
            "---------------",
            f"Number of Cores:  {psutil.cpu_count()}",
            "",
        ]
    )

    mem = psutil.virtual_memory()
    swap = psutil.swap_memory()
    lines.extend(
        [
            "Memory Information",
            "------------------",
            f"Total Memory:     {mem.total // (1024 * 1024)} MB",
            f"Used Memory:      {mem.used // (1024 * 1024)} MB",
            f"Total Swap:       {swap.total // (1024 * 1024)} MB",
            f"Used Swap:        {swap.used // (1024 * 1024)} MB",
            "",
        ]
    )

    lines.extend(["Network Interfaces", "------------------"])
    for name, stats in psutil.net_if_stats().items():
        # Get address if possible
        addr_info = psutil.net_if_addrs().get(name, [])
        mac = next(
            (a.address for a in addr_info if a.family == psutil.AF_LINK), "unknown"
        )
        # io stats
        counters = psutil.net_io_counters(pernic=True).get(name)
        if counters:
            lines.append(
                f"{name:<18}: RX: {counters.bytes_recv:>10} bytes, TX: {counters.bytes_sent:>10} bytes (MAC: {mac})"
            )
        else:
            lines.append(f"{name:<18}: (No IO stats) (MAC: {mac})")

    return "\n".join(lines)


def collect_disk_usage() -> str:
    lines = ["Disk Usage Report", "=================\n"]

    for part in psutil.disk_partitions():
        try:
            usage = psutil.disk_usage(part.mountpoint)
            used_mb = usage.used // (1024 * 1024)
            total_mb = usage.total // (1024 * 1024)
            pct = usage.percent
            lines.append(
                f"{part.mountpoint:<20} {part.fstype:<10} {used_mb:>10} / {total_mb:>10} MB used ({pct:.1f}%)"
            )
        except PermissionError:
            continue
        except Exception:
            continue

    return "\n".join(lines)


async def check_api_key_status(args) -> tuple[str, bool]:
    lines = ["MCP API Key Status", "------------------"]
    is_valid = False

    provided_key = os.environ.get("MCP_API_KEY")
    if not provided_key:
        for i, arg in enumerate(args):
            if arg == "--key" and i + 1 < len(args):
                provided_key = args[i + 1]
                break

    if provided_key:
        lines.append("Provided Key:     [FOUND]")
        project_id = get_project_id()
        if project_id:
            try:
                expected_key = await fetch_mcp_api_key(project_id)
                if provided_key == expected_key:
                    lines.append("Cloud Match:      [MATCHED]")
                    is_valid = True
                else:
                    lines.append("Cloud Match:      [MISMATCH]")
            except Exception as e:
                lines.append(f"Cloud Match:      [ERROR: {e}]")
        else:
            lines.append("Cloud Match:      [ERROR: Project ID not found]")
    else:
        lines.append("Provided Key:     [NOT FOUND]")

    lines.append("")
    return "\n".join(lines), is_valid


# MCP Tools using FastMCP decorators
@mcp.tool()
async def local_system_info() -> str:
    """Get a detailed system information report including kernel, cores, and memory usage."""
    return collect_system_info("Authentication:   [VERIFIED] (Running as MCP Server)\n")


@mcp.tool()
async def disk_usage() -> str:
    """Get disk usage information for all mounted disks."""
    return collect_disk_usage()


def main():

    args = sys.argv

    if "info" in args:
        status, is_valid = asyncio.run(check_api_key_status(args))

        if not is_valid:
            print(status, file=sys.stderr)

            print("Authentication Failed: Invalid or missing API Key", file=sys.stderr)

            sys.exit(1)

        print(collect_system_info(status))

        return

    elif "disk" in args:
        print(collect_disk_usage())
        return

    # Determine the expected API Key for server verification
    expected_api_key = os.environ.get("MCP_API_KEY")
    if not expected_api_key:
        for i, arg in enumerate(args):
            if arg == "--key" and i + 1 < len(args):
                expected_api_key = args[i + 1]
                break

    project_id = get_project_id()
    if not expected_api_key and project_id:
        try:
            expected_api_key = asyncio.run(fetch_mcp_api_key(project_id))
            logger.info("Successfully fetched MCP API Key from Google Cloud settings")
        except Exception as e:
            logger.warning(f"Could not fetch MCP API Key from Google Cloud: {e}")
    elif expected_api_key:
        logger.info("Using MCP API Key from environment or command line for verification")

    transport = os.environ.get("MCP_TRANSPORT", "sse")

    logger.info(f"Starting httpkey-python MCP server (Transport: {transport})")

    if transport == "sse":
        app = mcp.sse_app()

        if expected_api_key:

            @app.middleware("http")
            async def api_key_middleware(request: Request, call_next):
                auth_key = request.headers.get("X-Goog-Api-Key")
                if auth_key != expected_api_key:
                    return JSONResponse(
                        {"error": "Unauthorized: Invalid or missing API Key"},
                        status_code=401,
                    )
                return await call_next(request)

            logger.info("API Key verification enabled (X-Goog-Api-Key)")
        else:
            logger.warning("No API Key found. Server will be UNPROTECTED.")

        port = int(os.environ.get("PORT", 8080))
        uvicorn.run(app, host="0.0.0.0", port=port)
    else:
        mcp.run(transport="stdio")


if __name__ == "__main__":
    main()
