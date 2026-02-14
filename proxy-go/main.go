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

func collectSystemInfo() string {
	var sb strings.Builder
	sb.WriteString("System Information Report\n")
	sb.WriteString("=========================\n\n")

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

	if len(os.Args) <= 1 {
		slog.Info("Entering Server Mode", "port", port)

		var once sync.Once
		var server *mcp.Server

		initServer := func() {
			once.Do(func() {
				slog.Info("Lazy Initialization started")
				server = mcp.NewServer(&mcp.Implementation{Name: "proxy-go", Version: "1.0.0"}, nil)
				type empty struct{}
				mcp.AddTool(server, &mcp.Tool{Name: "local_system_info", Description: "System info"}, func(ctx context.Context, request *mcp.CallToolRequest, input empty) (*mcp.CallToolResult, any, error) {
					return &mcp.CallToolResult{Content: []mcp.Content{&mcp.TextContent{Text: collectSystemInfo()}}}, nil, nil
				})
				mcp.AddTool(server, &mcp.Tool{Name: "disk_usage", Description: "Disk usage"}, func(ctx context.Context, request *mcp.CallToolRequest, input empty) (*mcp.CallToolResult, any, error) {
					return &mcp.CallToolResult{Content: []mcp.Content{&mcp.TextContent{Text: collectDiskUsage()}}}, nil, nil
				})
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

	switch command {
	case "info":
		fmt.Print(collectSystemInfo())
	case "disk":
		fmt.Print(collectDiskUsage())
	case "check":
		if isTTY() {
			fmt.Println("System utilities available (No Authentication Required)")
		} else {
			slog.Info("System utilities available")
		}
	default:
		fmt.Printf("Unknown command: %s\n", command)
		os.Exit(1)
	}
}
