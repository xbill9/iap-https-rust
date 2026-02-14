source ../set_env.sh

echo "Starting Local Proxy"
gcloud run services proxy sysutils-proxy-rust --region us-central1 --port=3000



