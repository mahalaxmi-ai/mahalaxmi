#!/bin/bash
set -euo pipefail
SECRETS_DIR="$(cd "$(dirname "$0")/../secrets" && pwd)"
DEPLOY_PATH="/opt/deployments/mahalaxmi-website/website"
PUBLIC_KEY=$(age-keygen -y "$HOME/.config/sops/age/keys.txt")
export SOPS_AGE_KEY_FILE="$HOME/.config/sops/age/keys.txt"
echo "Encrypting mahalaxmi-website secrets..."
sops --encrypt --config /dev/null --age "$PUBLIC_KEY" \
  --input-type dotenv --output-type dotenv \
  "$DEPLOY_PATH/.env.prod" > "$SECRETS_DIR/.env.prod.enc"
echo "✅ secrets/.env.prod.enc updated — git add secrets/ && git commit"
