#!/bin/bash
set -euo pipefail
SECRETS_DIR="$(cd "$(dirname "$0")/../secrets" && pwd)"
DEPLOY_PATH="/opt/deployments/mahalaxmi-website/website"
export SOPS_AGE_KEY_FILE="$HOME/.config/sops/age/keys.txt"
echo "Decrypting mahalaxmi-website secrets..."
mkdir -p "$DEPLOY_PATH"
sops --decrypt --config /dev/null --input-type dotenv --output-type dotenv \
  "$SECRETS_DIR/.env.prod.enc" > "$DEPLOY_PATH/.env.prod"
chmod 600 "$DEPLOY_PATH/.env.prod"
echo "✅ .env.prod → $DEPLOY_PATH/.env.prod"
