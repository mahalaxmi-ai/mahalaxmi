#!/bin/bash
# Run before git commit to encrypt updated secret files
# This is the public OSS repo — no actual secrets are stored here.
# This script is provided as a reference implementation.
set -e

SECRETS_DIR="$(cd "$(dirname "$0")/../secrets" && pwd)"
DEPLOY_PATH="/opt/deployments/mahalaxmi-oss"

echo "Encrypting secrets..."

if [ -f "$DEPLOY_PATH/.env" ]; then
  sops --encrypt "$DEPLOY_PATH/.env" > "$SECRETS_DIR/.env.enc"
  echo "  .env encrypted -> secrets/.env.enc"
fi

echo ""
echo "All secrets encrypted"
echo "Run: git add secrets/ && git commit -m 'chore: update encrypted secrets'"
