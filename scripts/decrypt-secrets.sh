#!/bin/bash
# Run after git pull to restore secret files to their target paths
# This is the public OSS repo — no actual secrets are stored here.
# This script is provided as a reference implementation.
set -e

SECRETS_DIR="$(cd "$(dirname "$0")/../secrets" && pwd)"
DEPLOY_PATH="/opt/deployments/mahalaxmi-oss"

echo "Decrypting secrets..."

if [ -f "$SECRETS_DIR/.env.enc" ]; then
  mkdir -p "$DEPLOY_PATH"
  sops --decrypt "$SECRETS_DIR/.env.enc" > "$DEPLOY_PATH/.env"
  chmod 600 "$DEPLOY_PATH/.env"
  echo "  .env -> $DEPLOY_PATH/.env"
fi

echo ""
echo "All secrets decrypted successfully"
