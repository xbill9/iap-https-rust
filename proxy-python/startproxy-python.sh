source ../set_env.sh

echo "Starting Local Proxy"
gcloud run services proxy sysutils-proxy-python --region us-central1 --port=3000



