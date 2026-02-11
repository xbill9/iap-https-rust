package main

import (
	"context"
	"fmt"
	"log/slog"
	"os"
	"runtime"
	"strings"

	"github.com/mark3labs/mcp-go/mcp"
	"github.com/mark3labs/mcp-go/server"
	"github.com/shirou/gopsutil/v3/cpu"
	"github.com/shirou/gopsutil/v3/disk"
	"github.com/shirou/gopsutil/v3/host"
	"github.com/shirou/gopsutil/v3/mem"
	"github.com/shirou/gopsutil/v3/net"
)

func collectSystemInfo(apiStatus string) string {
	var sb strings.Builder
	sb.WriteString("System Information Report\n")
	sb.WriteString("=========================\n\n")

	if apiStatus != "" {
		sb.WriteString(apiStatus + "\n")
	}

	hInfo, err := host.Info()
	sb.WriteString("System Information\n")
	sb.WriteString("------------------\n")
	if err != nil {
		sb.WriteString(fmt.Sprintf("Error retrieving host info: %v\n", err))
	} else {
		sb.WriteString(fmt.Sprintf("System Name:      %s\n", runtime.GOOS))
		sb.WriteString(fmt.Sprintf("OS Name:          %s\n", hInfo.OS))
		sb.WriteString(fmt.Sprintf("Host Name:        %s\n", hInfo.Hostname))
		sb.WriteString(fmt.Sprintf("Uptime:           %d seconds\n", hInfo.Uptime))
	}
	sb.WriteString("\n")

	cpuCount, err := cpu.Counts(true)
	sb.WriteString("CPU Information\n")
	sb.WriteString("---------------\n")
	if err != nil {
		sb.WriteString(fmt.Sprintf("Error retrieving CPU counts: %v\n", err))
	} else {
		sb.WriteString(fmt.Sprintf("Number of Cores:  %d\n", cpuCount))
	}
	sb.WriteString("\n")

	vMem, errV := mem.VirtualMemory()
	sMem, errS := mem.SwapMemory()
	sb.WriteString("Memory Information\n")
	sb.WriteString("------------------\n")
	if errV != nil {
		sb.WriteString(fmt.Sprintf("Error retrieving virtual memory: %v\n", errV))
	} else {
		sb.WriteString(fmt.Sprintf("Total Memory:     %d MB\n", vMem.Total/(1024*1024)))
		sb.WriteString(fmt.Sprintf("Used Memory:      %d MB\n", vMem.Used/(1024*1024)))
	}
	if errS != nil {
		sb.WriteString(fmt.Sprintf("Error retrieving swap memory: %v\n", errS))
	} else {
		sb.WriteString(fmt.Sprintf("Total Swap:       %d MB\n", sMem.Total/(1024*1024)))
		sb.WriteString(fmt.Sprintf("Used Swap:        %d MB\n", sMem.Used/(1024*1024)))
	}
	sb.WriteString("\n")

	sb.WriteString("Network Interfaces\n")
	sb.WriteString("------------------\n")
	interfaces, errI := net.Interfaces()
	if errI != nil {
		sb.WriteString(fmt.Sprintf("Error retrieving network interfaces: %v\n", errI))
	} else {
		ioCounters, errIO := net.IOCounters(true)
		for _, iface := range interfaces {
			mac := iface.HardwareAddr
			if mac == "" {
				mac = "unknown"
			}

			var rx, tx uint64
			found := false
			if errIO == nil {
				for _, io := range ioCounters {
					if io.Name == iface.Name {
						rx = io.BytesRecv
						tx = io.BytesSent
						found = true
						break
					}
				}
			}

			if found {
				sb.WriteString(fmt.Sprintf("%-18s: RX: %10d bytes, TX: %10d bytes (MAC: %s)\n", iface.Name, rx, tx, mac))
			} else {
				sb.WriteString(fmt.Sprintf("%-18s: (No IO stats) (MAC: %s)\n", iface.Name, mac))
			}
		}
	}

	return sb.String()
}

func collectDiskUsage() string {
	var sb strings.Builder
	sb.WriteString("Disk Usage Report\n")
	sb.WriteString("=================\n\n")

	parts, err := disk.Partitions(false)
	if err != nil {
		sb.WriteString(fmt.Sprintf("Error retrieving disk partitions: %v\n", err))
		return sb.String()
	}

	for _, part := range parts {
		usage, err := disk.Usage(part.Mountpoint)
		if err != nil {
			sb.WriteString(fmt.Sprintf("%-20s %-10s Error: %v\n", part.Mountpoint, part.Fstype, err))
			continue
		}
		usedMB := usage.Used / (1024 * 1024)
		totalMB := usage.Total / (1024 * 1024)
		sb.WriteString(fmt.Sprintf("%-20s %-10s %10d / %10d MB used (%.1f%%)\n",
			part.Mountpoint, part.Fstype, usedMB, totalMB, usage.UsedPercent))
	}

	return sb.String()
}

func main() {
	slog.SetDefault(slog.New(slog.NewJSONHandler(os.Stderr, nil)))
	args := os.Args[1:]

	hasInfo := false
	hasDisk := false

	for _, arg := range args {
		if arg == "info" {
			hasInfo = true
		} else if arg == "disk" {
			hasDisk = true
		}
	}

	if hasInfo {
		fmt.Print(collectSystemInfo(""))
		return
	}

	if hasDisk {
		fmt.Print(collectDiskUsage())
		return
	}

	// Server mode
	s := server.NewMCPServer(
		"stdio-go",
		"1.0.0",
	)

	s.AddTool(mcp.NewTool("local_system_info",
		mcp.WithDescription("Get a detailed system information report including kernel, cores, and memory usage."),
	), func(ctx context.Context, request mcp.CallToolRequest) (*mcp.CallToolResult, error) {
		return mcp.NewToolResultText(collectSystemInfo("")), nil
	})

	s.AddTool(mcp.NewTool("disk_usage",
		mcp.WithDescription("Get disk usage information for all mounted disks."),
	), func(ctx context.Context, request mcp.CallToolRequest) (*mcp.CallToolResult, error) {
		return mcp.NewToolResultText(collectDiskUsage()), nil
	})

	slog.Info("Starting stdio-go MCP server", "transport", "stdio")

	if err := server.ServeStdio(s); err != nil {
		slog.Error("Failed to serve stdio", "error", err)
		os.Exit(1)
	}
}
