package main

import (
	"context"
	"fmt"
	"log/slog"
	"net/http"
	"os"
	"runtime"
	"strings"
	"sync"

	"github.com/modelcontextprotocol/go-sdk/mcp"
	"github.com/shirou/gopsutil/v3/cpu"
	"github.com/shirou/gopsutil/v3/disk"
	"github.com/shirou/gopsutil/v3/host"
	"github.com/shirou/gopsutil/v3/mem"
	"github.com/shirou/gopsutil/v3/net"
)

const MiB = 1024 * 1024

func collectSystemInfo() string {
	var sb strings.Builder
	fmt.Fprintln(&sb, "System Information Report")
	fmt.Fprintln(&sb, "=========================")
	fmt.Fprintln(&sb)

	fmt.Fprintln(&sb, "System Information")
	fmt.Fprintln(&sb, "------------------")
	fmt.Fprintf(&sb, "System Name:      %s\n", runtime.GOOS)
	if hInfo, err := host.Info(); err == nil {
		fmt.Fprintf(&sb, "OS Name:          %s\n", hInfo.OS)
		fmt.Fprintf(&sb, "Host Name:        %s\n", hInfo.Hostname)
	} else {
		fmt.Fprintf(&sb, "OS/Host Info:     Error: %v\n", err)
	}

	fmt.Fprintln(&sb, "\nCPU Information")
	fmt.Fprintln(&sb, "---------------")
	if cpuCount, err := cpu.Counts(true); err == nil {
		fmt.Fprintf(&sb, "Number of Cores:  %d\n", cpuCount)
	} else {
		fmt.Fprintf(&sb, "CPU Info:         Error: %v\n", err)
	}

	fmt.Fprintln(&sb, "\nMemory Information")
	fmt.Fprintln(&sb, "------------------")
	if vMem, err := mem.VirtualMemory(); err == nil {
		fmt.Fprintf(&sb, "Total Memory:     %d MB\n", vMem.Total/MiB)
		fmt.Fprintf(&sb, "Used Memory:      %d MB\n", vMem.Used/MiB)
	} else {
		fmt.Fprintf(&sb, "Memory Info:      Error: %v\n", err)
	}
	if sMem, err := mem.SwapMemory(); err == nil {
		fmt.Fprintf(&sb, "Total Swap:       %d MB\n", sMem.Total/MiB)
		fmt.Fprintf(&sb, "Used Swap:        %d MB\n", sMem.Used/MiB)
	}

	fmt.Fprintln(&sb, "\nNetwork Interfaces")
	fmt.Fprintln(&sb, "------------------")
	interfaces, err := net.Interfaces()
	if err != nil {
		fmt.Fprintf(&sb, "Network Info:     Error fetching interfaces: %v\n", err)
		return sb.String()
	}

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
			fmt.Fprintf(&sb, "%-18s: RX: %10d bytes, TX: %10d bytes (MAC: %s)\n", inter.Name, rx, tx, mac)
		} else {
			fmt.Fprintf(&sb, "%-18s: (No IO stats) (MAC: %s)\n", inter.Name, mac)
		}
	}

	return sb.String()
}

func collectDiskUsage() string {
	var sb strings.Builder
	fmt.Fprintln(&sb, "Disk Usage Report")
	fmt.Fprintln(&sb, "=================")
	fmt.Fprintln(&sb)

	partitions, err := disk.Partitions(false)
	if err != nil {
		fmt.Fprintf(&sb, "Error fetching partitions: %v\n", err)
		return sb.String()
	}

	for _, p := range partitions {
		usage, err := disk.Usage(p.Mountpoint)
		if err == nil {
			usedMB := usage.Used / MiB
			totalMB := usage.Total / MiB
			fmt.Fprintf(&sb, "%-20s %-10s %10d / %10d MB used (%.1f%%)\n",
				p.Mountpoint, p.Fstype, usedMB, totalMB, usage.UsedPercent)
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

	bearerToken := os.Getenv("MCP_BEARER_TOKEN")
	if bearerToken != "" {
		slog.Info("MCP_BEARER_TOKEN found")
	}

	if len(os.Args) <= 1 {
		runServer(port, bearerToken)
		return
	}

	handleCLI(os.Args[1], bearerToken)
}

func runServer(port, bearerToken string) {
	slog.Info("Entering Server Mode", "port", port, "auth_enabled", bearerToken != "")

	var (
		server     *mcp.Server
		once       sync.Once
		initServer = func() {
			once.Do(func() {
				slog.Info("Lazy Initialization started")
				server = mcp.NewServer(&mcp.Implementation{Name: "bearer-go", Version: "1.0.0"}, nil)
				type empty struct{}

				mcp.AddTool(server, &mcp.Tool{Name: "local_system_info", Description: "System info"},
					func(ctx context.Context, request *mcp.CallToolRequest, input empty) (*mcp.CallToolResult, any, error) {
						return &mcp.CallToolResult{Content: []mcp.Content{&mcp.TextContent{Text: collectSystemInfo()}}}, nil, nil
					})

				mcp.AddTool(server, &mcp.Tool{Name: "disk_usage", Description: "Disk usage"},
					func(ctx context.Context, request *mcp.CallToolRequest, input empty) (*mcp.CallToolResult, any, error) {
						return &mcp.CallToolResult{Content: []mcp.Content{&mcp.TextContent{Text: collectDiskUsage()}}}, nil, nil
					})
				slog.Info("Lazy Initialization complete")
			})
		}
	)

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

		if bearerToken != "" {
			authHeader := r.Header.Get("Authorization")
			if !strings.HasPrefix(authHeader, "Bearer ") || strings.TrimPrefix(authHeader, "Bearer ") != bearerToken {
				slog.Warn("Unauthorized request")
				http.Error(w, "Unauthorized", http.StatusUnauthorized)
				return
			}
		}

		mcpHandler.ServeHTTP(w, r)
	})

	slog.Info("Starting ListenAndServe", "address", "0.0.0.0:"+port)
	if err := http.ListenAndServe("0.0.0.0:"+port, mux); err != nil {
		slog.Error("ListenAndServe failed", "error", err)
		os.Exit(1)
	}
}

func handleCLI(command, bearerToken string) {
	switch command {
	case "info":
		fmt.Print(collectSystemInfo())
	case "disk":
		fmt.Print(collectDiskUsage())
	case "check":
		if isTTY() {
			authMsg := "No Authentication Required"
			if bearerToken != "" {
				authMsg = "Bearer Token Authentication Enabled"
			}
			fmt.Printf("System utilities available (%s)\n", authMsg)
		} else {
			slog.Info("System utilities available", "auth_enabled", bearerToken != "")
		}
	default:
		fmt.Printf("Unknown command: %s\n", command)
		os.Exit(1)
	}
}
