package main

import (
	"strings"
	"testing"
)

func TestCollectDiskUsage(t *testing.T) {
	output := collectDiskUsage()
	if !strings.Contains(output, "Disk Usage Report") {
		t.Errorf("Expected output to contain 'Disk Usage Report', got: %s", output)
	}
	// We expect at least one mount point or an error message if partitions can't be read
	if !strings.Contains(output, "/") && !strings.Contains(output, "Error") && !strings.Contains(output, "C:") {
		t.Errorf("Expected output to contain some disk info or error, got: %s", output)
	}
}

func TestCollectSystemInfo(t *testing.T) {
	output := collectSystemInfo("test status")
	if !strings.Contains(output, "System Information Report") {
		t.Errorf("Expected output to contain 'System Information Report', got: %s", output)
	}
	if !strings.Contains(output, "test status") {
		t.Errorf("Expected output to contain 'test status', got: %s", output)
	}
	if !strings.Contains(output, "CPU Information") {
		t.Errorf("Expected output to contain 'CPU Information', got: %s", output)
	}
	if !strings.Contains(output, "Memory Information") {
		t.Errorf("Expected output to contain 'Memory Information', got: %s", output)
	}
}
