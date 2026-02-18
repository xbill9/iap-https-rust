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
}

func TestCollectSystemInfo(t *testing.T) {
	output := collectSystemInfo()
	if !strings.Contains(output, "System Information Report") {
		t.Errorf("Expected output to contain 'System Information Report', got: %s", output)
	}
}
