import logging
import os
import subprocess
import sys

import psutil
from mcp.server.fastmcp import FastMCP
import uvicorn

# Configure logging to stderr
logging.basicConfig(
    level=logging.INFO,
    format="%(asctime)s - %(name)s - %(levelname)s - %(message)s",
    stream=sys.stderr,
)
logger = logging.getLogger("proxy-python")

# Initialize FastMCP
mcp = FastMCP(
    "proxy-python",
    port=int(os.environ.get("PORT", 8080)),
    host="0.0.0.0",
)


def collect_system_info() -> str:
    lines = ["System Information Report", "=========================\n"]

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


# MCP Tools using FastMCP decorators
@mcp.tool()
async def local_system_info() -> str:
    """Get a detailed system information report including kernel, cores, and memory usage."""
    return collect_system_info()


@mcp.tool()
async def disk_usage() -> str:
    """Get disk usage information for all mounted disks."""
    return collect_disk_usage()


def main():

    args = sys.argv

    if "info" in args:
        print(collect_system_info())
        return

    elif "disk" in args:
        print(collect_disk_usage())
        return

    transport = os.environ.get("MCP_TRANSPORT", "http")

    logger.info(f"Starting proxy-python MCP server (Transport: {transport})")

    if transport == "http":
        app = mcp.streamable_http_app()
        port = int(os.environ.get("PORT", 8080))
        uvicorn.run(app, host="0.0.0.0", port=port)
    elif transport == "sse":
        app = mcp.sse_app()
        port = int(os.environ.get("PORT", 8080))
        uvicorn.run(app, host="0.0.0.0", port=port)
    else:
        mcp.run(transport="stdio")


if __name__ == "__main__":
    main()
