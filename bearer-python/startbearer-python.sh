source ../set_env.sh

SERVICE_NAME="sysutils-bearer-python"
REGION="${REGION:-us-central1}"

echo "Fetching Cloud Run service URL for $SERVICE_NAME in $REGION..."
SERVICE_URL=$(gcloud run services describe $SERVICE_NAME --region $REGION --format='value(status.url)' 2>/dev/null)

if [ -z "$SERVICE_URL" ]; then
    echo "Error: Could not find URL for service $SERVICE_NAME in $REGION."
    echo "Please ensure the service is deployed and you have access."
    echo "You might need to run: make deploy"
fi

echo "Service URL: $SERVICE_URL"

echo "Fetching identity token from ADC..."
export ID_TOKEN=$(gcloud auth print-identity-token 2>/dev/null)
export MCP_BEARER_TOKEN=$(gcloud auth print-identity-token 2>/dev/null)

if [ -z "$ID_TOKEN" ]; then
    echo "Error: Failed to get identity token."
    echo "Please run: gcloud auth application-default login"
fi

echo ""
echo "--- Authentication Debug Information ---"
ACTIVE_ACCOUNT=$(gcloud auth list --filter=status:ACTIVE --format="value(account)")
echo "Authenticated as: $ACTIVE_ACCOUNT"
echo "Active Project:   $(gcloud config get-value project 2>/dev/null)"
echo "Target Service:   $SERVICE_NAME"
echo "Target Region:    $REGION"
echo "----------------------------------------"
echo ""

echo "Done. $SETTINGS_FILE is now configured for direct Cloud Run access."
