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
	"github.com/shirou/gopsutil/v3/host"
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
			if err != nil {
				return "", err
			}
			return respKey.KeyString, nil
		}
	}
	return "", fmt.Errorf("MCP API Key not found")
}

func collectSystemInfo(apiStatus string) string {
	var sb strings.Builder
	sb.WriteString("System Information Report\n")
	sb.WriteString(fmt.Sprintf("System Name: %s\n", runtime.GOOS))
	hInfo, _ := host.Info()
	sb.WriteString(fmt.Sprintf("OS Name: %s\n", hInfo.OS))
	return sb.String()
}

func collectDiskUsage() string {
	return "Disk Usage Report Placeholder"
}

func main() {
	fmt.Fprintln(os.Stderr, "APP_STARTING")
	slog.SetDefault(slog.New(slog.NewJSONHandler(os.Stderr, nil)))
	port := os.Getenv("PORT")
	if port == "" {
		port = "8080"
	}

	// Always provide server mode if no args
	if len(os.Args) <= 1 {
		fmt.Fprintf(os.Stderr, "Entering Server Mode on port %s\n", port)

		var once sync.Once
		var server *mcp.Server
		var expectedKey string

		initServer := func() {
			once.Do(func() {
				fmt.Fprintln(os.Stderr, "Lazy Initialization started")
				server = mcp.NewServer(&mcp.Implementation{Name: "manual-go", Version: "1.0.0"}, nil)
				type empty struct{}
				mcp.AddTool(server, &mcp.Tool{Name: "local_system_info", Description: "System info"}, func(ctx context.Context, request *mcp.CallToolRequest, input empty) (*mcp.CallToolResult, any, error) {
					return &mcp.CallToolResult{Content: []mcp.Content{&mcp.TextContent{Text: collectSystemInfo("Verified")}}}, nil, nil
				})

				projectID := getProjectID()
				if projectID != "" {
					ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
					defer cancel()
					key, _ := fetchMCPAPIKeyLibrary(ctx, projectID)
					expectedKey = key
				}
				fmt.Fprintln(os.Stderr, "Lazy Initialization complete")
			})
		}

		mcpHandler := mcp.NewStreamableHTTPHandler(func(r *http.Request) *mcp.Server {
			initServer()
			return server
		}, nil)

		mux := http.NewServeMux()
		mux.HandleFunc("/", func(w http.ResponseWriter, r *http.Request) {
			if r.URL.Path == "/" || r.URL.Path == "/healthz" {
				fmt.Fprintln(os.Stderr, "Health check received")
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

		fmt.Fprintf(os.Stderr, "Starting ListenAndServe on :%s\n", port)
		err := http.ListenAndServe(":"+port, mux)
		if err != nil {
			fmt.Fprintf(os.Stderr, "ListenAndServe failed: %v\n", err)
			os.Exit(1)
		}
		return
	}

	fmt.Println("CLI mode not implemented")
}
