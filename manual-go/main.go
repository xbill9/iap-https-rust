package main

import (
	"context"
	"fmt"
	"log/slog"
	"net/http"
	"os"
	"os/exec"
	"runtime"
	"strings"
	"sync"
	"time"

	"github.com/modelcontextprotocol/go-sdk/mcp"
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
	out, err := exec.Command("gcloud", "config", "get-value", "project").Output()
	if err == nil {
		return strings.TrimSpace(string(out))
	}
	return ""
}

func fetchMCPAPIKeyGcloud(projectID string) (string, error) {
	out, err := exec.Command("gcloud", "services", "api-keys", "list",
		"--project", projectID,
		"--filter", "displayName='MCP API Key'",
		"--format", "value(name)").Output()
	if err != nil {
		return "", err
	}
	keyName := strings.TrimSpace(string(out))
	if keyName == "" {
		return "", fmt.Errorf("MCP API Key not found via gcloud")
	}

	out, err = exec.Command("gcloud", "services", "api-keys", "get-key-string",
		keyName, "--project", projectID,
		"--format", "value(keyString)").Output()
	if err == nil && len(out) > 0 {
		return strings.TrimSpace(string(out)), nil
	}
	return "", fmt.Errorf("failed to get key string via gcloud")
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
	for _, key := range resp.Keys {
		if key.DisplayName == "MCP API Key" {
			respKey, err := service.Projects.Locations.Keys.GetKeyString(key.Name).Do()
			if err == nil {
				return respKey.KeyString, nil
			}
		}
	}
	return "", fmt.Errorf("MCP API Key not found via library")
}

func fetchMCPAPIKey(ctx context.Context, projectID string) (string, error) {
	slog.Info("Fetching MCP API Key", "projectID", projectID)

	// Prefer library-based fetch (ADC), typical for Cloud Run
	key, err := fetchMCPAPIKeyLibrary(ctx, projectID)
	if err == nil {
		slog.Info("Successfully fetched MCP API Key from Google Cloud settings")
		return key, nil
	}

	slog.Info("Falling back to gcloud-based API key fetch", "error", err)
	key, err = fetchMCPAPIKeyGcloud(projectID)
	if err == nil {
		slog.Info("Successfully fetched API key via gcloud")
		return key, nil
	}

	slog.Warn("MCP API Key not found in Google Cloud project", "projectID", projectID, "error", err)
	return "", fmt.Errorf("MCP API Key not found")
}

func collectSystemInfo(apiStatus string) string {
	var sb strings.Builder
	sb.WriteString("System Information Report\n")
	sb.WriteString("=========================\n\n")

	if apiStatus != "" {
		sb.WriteString("MCP API Key Status\n")
		sb.WriteString("------------------\n")
		sb.WriteString(apiStatus + "\n\n")
	}

	hInfo, _ := host.Info()
	sb.WriteString("System Information\n")
	sb.WriteString("------------------\n")
	sb.WriteString(fmt.Sprintf("System Name:      %s\n", runtime.GOOS))
	if hInfo != nil {
		sb.WriteString(fmt.Sprintf("OS Name:          %s\n", hInfo.OS))
		sb.WriteString(fmt.Sprintf("Host Name:        %s\n", hInfo.Hostname))
	}

	cpuCount, _ := cpu.Counts(true)
	sb.WriteString("\nCPU Information\n")
	sb.WriteString("---------------\n")
	sb.WriteString(fmt.Sprintf("Number of Cores:  %d\n", cpuCount))

	vMem, _ := mem.VirtualMemory()
	sMem, _ := mem.SwapMemory()
	sb.WriteString("\nMemory Information\n")
	sb.WriteString("------------------\n")
	if vMem != nil {
		sb.WriteString(fmt.Sprintf("Total Memory:     %d MB\n", vMem.Total/1024/1024))
		sb.WriteString(fmt.Sprintf("Used Memory:      %d MB\n", vMem.Used/1024/1024))
	}
	if sMem != nil {
		sb.WriteString(fmt.Sprintf("Total Swap:       %d MB\n", sMem.Total/1024/1024))
		sb.WriteString(fmt.Sprintf("Used Swap:        %d MB\n", sMem.Used/1024/1024))
	}

	sb.WriteString("\nNetwork Interfaces\n")
	sb.WriteString("------------------\n")
	interfaces, _ := net.Interfaces()
	ioCounters, _ := net.IOCounters(true)
	for _, inter := range interfaces {
		var rx, tx uint64
		found := false
		for _, io := range ioCounters {
			if io.Name == inter.Name {
				rx = io.BytesRecv
				tx = io.BytesSent
				found = true
				break
			}
		}
		mac := inter.HardwareAddr
		if mac == "" {
			mac = "unknown"
		}
		if found {
			sb.WriteString(fmt.Sprintf("%-18s: RX: %10d bytes, TX: %10d bytes (MAC: %s)\n", inter.Name, rx, tx, mac))
		} else {
			sb.WriteString(fmt.Sprintf("%-18s: (No IO stats) (MAC: %s)\n", inter.Name, mac))
		}
	}

	return sb.String()
}

func collectDiskUsage() string {
	var sb strings.Builder
	sb.WriteString("Disk Usage Report\n")
	sb.WriteString("=================\n\n")

	partitions, _ := disk.Partitions(false)
	for _, p := range partitions {
		usage, err := disk.Usage(p.Mountpoint)
		if err == nil {
			usedMB := usage.Used / (1024 * 1024)
			totalMB := usage.Total / (1024 * 1024)
			sb.WriteString(fmt.Sprintf("%-20s %-10s %10d / %10d MB used (%.1f%%)\n",
				p.Mountpoint, p.Fstype, usedMB, totalMB, usage.UsedPercent))
		}
	}
	return sb.String()
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
	slog.Info("APP_STARTING")
	port := os.Getenv("PORT")
	if port == "" {
		port = "8080"
	}

	// If no args and it's a TTY, we might want to show status
	// but for HTTP variant we usually want to start the server.
	// stdiokey-go exits if TTY and no args.
	// We'll keep server mode as default for manual-go if no args.

	// Always provide server mode if no args
	if len(os.Args) <= 1 {
		slog.Info("Entering Server Mode", "port", port)

		var once sync.Once
		var server *mcp.Server
		var expectedKey string

		initServer := func() {
			once.Do(func() {
				slog.Info("Lazy Initialization started")
				server = mcp.NewServer(&mcp.Implementation{Name: "manual-go", Version: "1.0.0"}, nil)
				type empty struct{}
				mcp.AddTool(server, &mcp.Tool{Name: "local_system_info", Description: "System info"}, func(ctx context.Context, request *mcp.CallToolRequest, input empty) (*mcp.CallToolResult, any, error) {
					return &mcp.CallToolResult{Content: []mcp.Content{&mcp.TextContent{Text: collectSystemInfo("Verified")}}}, nil, nil
				})
				mcp.AddTool(server, &mcp.Tool{Name: "disk_usage", Description: "Disk usage"}, func(ctx context.Context, request *mcp.CallToolRequest, input empty) (*mcp.CallToolResult, any, error) {
					return &mcp.CallToolResult{Content: []mcp.Content{&mcp.TextContent{Text: collectDiskUsage()}}}, nil, nil
				})

				expectedKey = os.Getenv("MCP_API_KEY")
				if expectedKey == "" {
					projectID := getProjectID()
					if projectID != "" {
						ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
						defer cancel()
						key, _ := fetchMCPAPIKey(ctx, projectID)
						expectedKey = key
					}
				}

				if expectedKey != "" {
					slog.Info("Effective API Key established")
				} else {
					slog.Warn("No API Key found. Server may be unsecured or unauthorized.")
				}
				slog.Info("Lazy Initialization complete")
			})
		}

		mcpHandler := mcp.NewStreamableHTTPHandler(func(r *http.Request) *mcp.Server {
			initServer()
			return server
		}, nil)

		mux := http.NewServeMux()
		mux.HandleFunc("/", func(w http.ResponseWriter, r *http.Request) {
			if r.URL.Path == "/" || r.URL.Path == "/healthz" {
				slog.Info("Health check received")
				w.WriteHeader(http.StatusOK)
				w.Write([]byte("OK"))
				return
			}

			initServer()
			apiKey := r.Header.Get("x-goog-api-key")
			if apiKey == "" {
				apiKey = r.Header.Get("x-api-key")
			}
			if apiKey == "" {
				apiKey = r.URL.Query().Get("apiKey")
			}

			if expectedKey != "" && apiKey != expectedKey {
				http.Error(w, "Unauthorized", http.StatusUnauthorized)
				return
			}

			mcpHandler.ServeHTTP(w, r)
		})

		slog.Info("Starting ListenAndServe", "address", "0.0.0.0:"+port)
		err := http.ListenAndServe("0.0.0.0:"+port, mux)
		if err != nil {
			slog.Error("ListenAndServe failed", "error", err)
			os.Exit(1)
		}
		return
	}

	command := os.Args[1]
	providedKey := os.Getenv("MCP_API_KEY")
	projectID := getProjectID()
	var expectedKey string
	if projectID != "" {
		ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
		defer cancel()
		expectedKey, _ = fetchMCPAPIKey(ctx, projectID)
	}

	keyStatus := "Provided Key: [NOT FOUND]"
	if providedKey != "" {
		keyStatus = "Provided Key: [FOUND]"
		if expectedKey != "" {
			if providedKey == expectedKey {
				keyStatus += "\nCloud Match: [MATCHED]"
			} else {
				keyStatus += "\nCloud Match: [MISMATCH]"
			}
		}
	}

	authenticated := providedKey != "" && expectedKey != "" && providedKey == expectedKey

	switch command {
	case "info":
		if !authenticated {
			slog.Error("Authentication Failed", "reason", "Invalid or missing API Key", "status", keyStatus)
			os.Exit(1)
		}
		fmt.Print(collectSystemInfo(keyStatus))
	case "disk":
		fmt.Print(collectDiskUsage())
	case "check":
		if isTTY() {
			fmt.Printf("MCP API Key Status\n------------------\n%s\n", keyStatus)
			if !authenticated {
				fmt.Println("\nAuthentication Failed: Invalid or missing API Key")
			} else {
				fmt.Println("\nAuthentication Verified")
			}
		} else {
			if !authenticated {
				slog.Error("Authentication Failed", "reason", "Invalid or missing API Key", "status", keyStatus)
			} else {
				slog.Info("Authentication Verified", "status", "MATCHED")
			}
		}
		if !authenticated {
			os.Exit(1)
		}
	default:
		fmt.Printf("Unknown command: %s\n", command)
		os.Exit(1)
	}
}
