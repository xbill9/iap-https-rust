#!/bin/bash

# Only exit on error if the script is being executed, not sourced.
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
  set -euo pipefail
fi

# --- Function for error handling ---
handle_error() {
  echo "Error: $1" >&2
  if [[ "${BASH_SOURCE[0]}" != "${0}" ]]; then
    return 1
  else
    exit 1
  fi
}

# --- Part 1: Set Google Cloud Project ID ---
REGION="${REGION:-us-central1}"

echo "--- Setting Google Cloud Project ID ---"

# Ensure we have a project ID set in gcloud
user_project_id=$(gcloud config get-value project 2>/dev/null)

if [[ -z "$user_project_id" ]]; then
  read -p "Please enter your Google Cloud project ID: " user_project_id
  if [[ -n "$user_project_id" ]]; then
    gcloud config set project "$user_project_id" --quiet
  else
    handle_error "No project ID was entered." || return 1
  fi
else
  echo "Using Google Cloud project: $user_project_id"
fi

# --- Part 2: Check and Set MCP API Key ---
echo "Checking for existing MCP API Key..."
KEY_NAME=$(gcloud services api-keys list --filter='displayName="MCP API Key"' --format="value(name)" --limit=1)

if [[ -z "$KEY_NAME" ]]; then
    echo "Creating MCP API Key..."
    gcloud services api-keys create --display-name="MCP API Key" || echo "API Key creation failed."
    
    # Wait for the key to be available (max 30 seconds)
    echo "Waiting for API Key to be ready..."
    for i in {1..10}; do
        KEY_NAME=$(gcloud services api-keys list --filter='displayName="MCP API Key"' --format="value(name)" --limit=1)
        if [[ -n "$KEY_NAME" ]]; then
            break
        fi
        sleep 3
    done
else
    echo "Using existing MCP API Key: $KEY_NAME"
fi

if [[ -z "$KEY_NAME" ]]; then
    handle_error "Failed to retrieve or create MCP API Key." || return 1
fi

echo "Retrieving API Key string..."
MCP_API_KEY=$(gcloud services api-keys get-key-string "$KEY_NAME" --format="value(keyString)")

if [[ -n "$MCP_API_KEY" ]]; then
    export MCP_API_KEY
    echo "MCP API Key retrieved and exported."
    
    echo ""
    echo "This key can be used with all variants that support API key validation:"
    echo "  - Rust: manual, local, stdiokey"
    echo "  - Python: manual-python, local-python, stdiokey-python"
    echo ""
    echo "Ensure this script was sourced: source ./set_key.sh"
else
    handle_error "Failed to retrieve MCP API Key string." || return 1
fi

# Environment checks
echo "--- Environment Checks ---"
if [[ -n "${CLOUD_SHELL:-}" ]]; then
    echo "Running in Google Cloud Shell."
elif curl -s -m 2 -i metadata.google.internal | grep -q "Metadata-Flavor: Google"; then
    echo "Running on a Google Cloud VM."
else
    echo "Not running in Google Cloud VM or Shell. Checking ADC..."
    if ! gcloud auth application-default print-access-token >/dev/null 2>&1; then
        echo "Setting ADC Credentials..."
        gcloud auth application-default login
    fi
fi

if [[ -n "${FIREBASE_DEPLOY_AGENT:-}" ]]; then
    echo "Running in Firebase Studio terminal."
fi

if [[ -d "/mnt/chromeos" ]]; then
    echo "Running on ChromeOS."
fi

echo "--- Initial Setup complete ---"