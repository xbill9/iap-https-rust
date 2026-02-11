package main

import (
	"context"
	"fmt"
	"log/slog"
	"os"
	"os/exec"
	"runtime"
	"strings"

	"github.com/mark3labs/mcp-go/mcp"
	"github.com/mark3labs/mcp-go/server"
	"github.com/shirou/gopsutil/v3/cpu"
	"github.com/shirou/gopsutil/v3/disk"
	"github.com/shirou/gopsutil/v3/host"
	"github.com/shirou/gopsutil/v3/mem"
	"github.com/shirou/gopsutil/v3/net"
	"google.golang.org/api/apikeys/v2"
	"google.golang.org/api/option"
)

func getProjectID() string {
	if projectID := os.Getenv("GOOGLE_CLOUD_PROJECT"); projectID != "" {
		return projectID
	}

	// Try gcloud config
	out, err := exec.Command("gcloud", "config", "get-value", "project").Output()
	if err == nil {
		return strings.TrimSpace(string(out))
	}

	return ""
}

func fetchMCPAPIKeyGcloud(projectID string) (string, error) {
	out, err := exec.Command("gcloud", "services", "api-keys", "list",
		"--project="+projectID,
		"--filter=displayName='MCP API Key'",
		"--format=value(name)").Output()
	if err != nil {
		return "", err
	}
	keyName := strings.TrimSpace(string(out))
	if keyName == "" {
		return "", fmt.Errorf("MCP API Key not found via gcloud")
	}

	out, err = exec.Command("gcloud", "services", "api-keys", "get-key-string",
		keyName,
		"--project="+projectID,
		"--format=value(keyString)").Output()
	if err != nil {
		return "", err
	}
	return strings.TrimSpace(string(out)), nil
}

func fetchMCPAPIKeyLibrary(ctx context.Context, projectID string) (string, error) {
	service, err := apikeys.NewService(ctx, option.WithScopes(apikeys.CloudPlatformScope))
	if err != nil {
		return "", err
	}

	parent := fmt.Sprintf("projects/%s/locations/global", projectID)
	resp, err := service.Projects.Locations.Keys.List(parent).Do()
	if err != nil {
		return "", err
	}

	var targetKeyName string
	for _, key := range resp.Keys {
		if key.DisplayName == "MCP API Key" {
			targetKeyName = key.Name
			break
		}
	}

	if targetKeyName == "" {
		return "", fmt.Errorf("MCP API Key not found")
	}

	respKey, err := service.Projects.Locations.Keys.GetKeyString(targetKeyName).Do()
	if err != nil {
		return "", err
	}

	return respKey.KeyString, nil
}

func fetchMCPAPIKey(ctx context.Context, projectID string) (string, error) {
	slog.Info("Fetching MCP API Key", "projectID", projectID)
	key, err := fetchMCPAPIKeyGcloud(projectID)
	if err == nil {
		slog.Info("Successfully fetched API key via gcloud")
		return key, nil
	}

	slog.Info("Falling back to library-based API key fetch", "error", err)
	return fetchMCPAPIKeyLibrary(ctx, projectID)
}

func collectSystemInfo(apiStatus string) string {
	var sb strings.Builder
	sb.WriteString("System Information Report\n")
	sb.WriteString("=========================\n\n")

	if apiStatus != "" {
		sb.WriteString(apiStatus + "\n")
	}

	hInfo, _ := host.Info()
	sb.WriteString("System Information\n")
	sb.WriteString("------------------\n")
	sb.WriteString(fmt.Sprintf("System Name:      %s\n", runtime.GOOS))
	sb.WriteString(fmt.Sprintf("OS Name:          %s\n", hInfo.OS))
	sb.WriteString(fmt.Sprintf("Host Name:        %s\n", hInfo.Hostname))
	sb.WriteString("\n")

	cpuCount, _ := cpu.Counts(true)
	sb.WriteString("CPU Information\n")
	sb.WriteString("---------------\n")
	sb.WriteString(fmt.Sprintf("Number of Cores:  %d\n", cpuCount))
	sb.WriteString("\n")

	vMem, _ := mem.VirtualMemory()
	sMem, _ := mem.SwapMemory()
	sb.WriteString("Memory Information\n")
	sb.WriteString("------------------\n")
	sb.WriteString(fmt.Sprintf("Total Memory:     %d MB\n", vMem.Total/(1024*1024)))
	sb.WriteString(fmt.Sprintf("Used Memory:      %d MB\n", vMem.Used/(1024*1024)))
	sb.WriteString(fmt.Sprintf("Total Swap:       %d MB\n", sMem.Total/(1024*1024)))
	sb.WriteString(fmt.Sprintf("Used Swap:        %d MB\n", sMem.Used/(1024*1024)))
	sb.WriteString("\n")

	sb.WriteString("Network Interfaces\n")
	sb.WriteString("------------------\n")
	interfaces, _ := net.Interfaces()
	ioCounters, _ := net.IOCounters(true)
	for _, iface := range interfaces {
		mac := iface.HardwareAddr
		if mac == "" {
			mac = "unknown"
		}

		var rx, tx uint64
		found := false
		for _, io := range ioCounters {
			if io.Name == iface.Name {
				rx = io.BytesRecv
				tx = io.BytesSent
				found = true
				break
			}
		}

		if found {
			sb.WriteString(fmt.Sprintf("%-18s: RX: %10d bytes, TX: %10d bytes (MAC: %s)\n", iface.Name, rx, tx, mac))
		} else {
			sb.WriteString(fmt.Sprintf("%-18s: (No IO stats) (MAC: %s)\n", iface.Name, mac))
		}
	}

	return sb.String()
}

func collectDiskUsage() string {
	var sb strings.Builder
	sb.WriteString("Disk Usage Report\n")
	sb.WriteString("=================\n\n")

	parts, _ := disk.Partitions(false)
	for _, part := range parts {
		usage, err := disk.Usage(part.Mountpoint)
		if err != nil {
			continue
		}
		usedMB := usage.Used / (1024 * 1024)
		totalMB := usage.Total / (1024 * 1024)
		sb.WriteString(fmt.Sprintf("%-20s %-10s %10d / %10d MB used (%.1f%%)\n",
			part.Mountpoint, part.Fstype, usedMB, totalMB, usage.UsedPercent))
	}

	return sb.String()
}

func checkAPIKeyStatus(ctx context.Context, args []string) (string, bool) {
	var sb strings.Builder
	sb.WriteString("MCP API Key Status\n")
	sb.WriteString("------------------\n")
	isValid := false

	projectID := getProjectID()
	expectedKey := ""
	if projectID != "" {
		sb.WriteString(fmt.Sprintf("Cloud Project:    %s\n", projectID))
		key, err := fetchMCPAPIKey(ctx, projectID)
		if err == nil {
			expectedKey = key
			sb.WriteString("Cloud Match:      [EXPECTED KEY FETCHED]\n")
		} else {
			sb.WriteString(fmt.Sprintf("Cloud Match:      [ERROR: %v]\n", err))
		}
	} else {
		sb.WriteString("Cloud Match:      [ERROR: Project ID not found]\n")
	}

	providedKey := os.Getenv("MCP_API_KEY")
	if providedKey == "" {
		for i, arg := range args {
			if arg == "--key" && i+1 < len(args) {
				providedKey = args[i+1]
				break
			}
		}
	}

	if providedKey != "" {
		sb.WriteString("Provided Key:     [FOUND]\n")
		if expectedKey != "" {
			if providedKey == expectedKey {
				sb.WriteString("Key Validation:   [SUCCESS]\n")
				isValid = true
			} else {
				sb.WriteString("Key Validation:   [FAILED: Mismatch]\n")
			}
		}
	} else {
		sb.WriteString("Provided Key:     [NOT FOUND]\n")
	}

	sb.WriteString("\n")
	return sb.String(), isValid
}

func isTTY() bool {
	fi, err := os.Stdin.Stat()
	if err != nil {
		return false
	}
	return (fi.Mode() & os.ModeCharDevice) != 0
}

func main() {
	slog.SetDefault(slog.New(slog.NewJSONHandler(os.Stderr, nil)))
	ctx := context.Background()
	args := os.Args[1:]

	hasInfo := false
	hasDisk := false
	hasCheck := false

	for _, arg := range args {
		if arg == "info" {
			hasInfo = true
		} else if arg == "disk" {
			hasDisk = true
		} else if arg == "check" {
			hasCheck = true
		}
	}

	// Always check API key status
	status, isValid := checkAPIKeyStatus(ctx, os.Args)

	// If called directly (TTY) with no args or 'check'
	if (len(args) == 0 || hasCheck) && isTTY() {
		fmt.Print(status)
		if isValid {
			fmt.Println("Authentication Verified: Server is ready to be used by an MCP host.")
		} else {
			fmt.Println("Authentication Failed: Invalid or missing API Key.")
			fmt.Println("Please set MCP_API_KEY environment variable or use --key flag.")
		}
		if hasCheck {
			if isValid {
				os.Exit(0)
			}
			os.Exit(1)
		}
		// If no args and valid, we still exit because it's a TTY
		if len(args) == 0 {
			return
		}
	}

	if !isValid {
		if isTTY() {
			fmt.Fprintln(os.Stderr, status)
			fmt.Fprintln(os.Stderr, "Authentication Failed: Invalid or missing API Key")
		} else {
			slog.Error("Authentication Failed", "reason", "Invalid or missing API Key", "status", status)
		}
		os.Exit(1)
	}

	if hasCheck {
		fmt.Print(status)
		return
	}

	if hasInfo {
		fmt.Print(collectSystemInfo(status))
		return
	}

	if hasDisk {
		fmt.Print(collectDiskUsage())
		return
	}

	// Server mode
	slog.Info("Authentication Verified", "status", "MATCHED")

	s := server.NewMCPServer(
		"stdiokey-go",
		"1.0.0",
	)

	s.AddTool(mcp.NewTool("local_system_info",
		mcp.WithDescription("Get a detailed system information report including kernel, cores, and memory usage."),
	), func(ctx context.Context, request mcp.CallToolRequest) (*mcp.CallToolResult, error) {
		return mcp.NewToolResultText(collectSystemInfo("Authentication:   [VERIFIED] (Running as MCP Server)\n")), nil
	})

	s.AddTool(mcp.NewTool("disk_usage",
		mcp.WithDescription("Get disk usage information for all mounted disks."),
	), func(ctx context.Context, request mcp.CallToolRequest) (*mcp.CallToolResult, error) {
		return mcp.NewToolResultText(collectDiskUsage()), nil
	})

	slog.Info("Starting stdiokey-go MCP server", "transport", "stdio")

	if err := server.ServeStdio(s); err != nil {
		slog.Error("Failed to serve stdio", "error", err)
		os.Exit(1)
	}
}
