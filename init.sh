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
PROJECT_FILE="${PROJECT_FILE:-$HOME/project_id.txt}"
REGION="${REGION:-us-central1}"

echo "--- Setting Google Cloud Project ID ---"

if [[ -f "$PROJECT_FILE" ]]; then
  DEFAULT_PROJECT=$(cat "$PROJECT_FILE")
  read -p "Enter Google Cloud project ID [$DEFAULT_PROJECT]: " user_project_id
  user_project_id=${user_project_id:-$DEFAULT_PROJECT}
else
  read -p "Please enter your Google Cloud project ID: " user_project_id
fi

if [[ -z "$user_project_id" ]]; then
  handle_error "No project ID was entered." || return 1
fi

echo "$user_project_id" > "$PROJECT_FILE"
chmod 600 "$PROJECT_FILE"

# Source environment variables
# Note: set_env.sh handles its own authentication checks and exports PROJECT_ID, REGION, etc.
source ./set_env.sh

# Set gcloud region (project is already set by set_env.sh)
gcloud config set compute/region "$REGION" --quiet

echo "Enabling Services..."
gcloud services enable \
    aiplatform.googleapis.com \
    compute.googleapis.com \
    apikeys.googleapis.com


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
