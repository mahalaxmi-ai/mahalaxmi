# Secrets Manifest — mahalaxmi-website

## Encrypted files in this directory

| Encrypted file | Decrypted target path | Purpose |
|---|---|---|
| `.env.prod.enc` | `/opt/deployments/mahalaxmi-website/website/.env.prod` | Production env vars |

## Secret variables in .env.prod

| Variable | Purpose |
|---|---|
| `MAHALAXMI_AUTH_API_URL` | Mahalaxmi auth API endpoint |
| `MAHALAXMI_PLATFORM_API_URL` | Mahalaxmi platform API endpoint |
| `MAHALAXMI_ACTIVATION_API_URL` | Mahalaxmi activation API endpoint |
| `MAHALAXMI_CLOUD_PAK_KEY` | Mahalaxmi cloud channel PAK (live) |
| `MAHALAXMI_TERMINAL_PAK_KEY` | Mahalaxmi terminal channel PAK (live) |
| `MAHALAXMI_VSCODE_PAK_KEY` | Mahalaxmi VS Code extension PAK (live) |

## How to decrypt

```bash
bash scripts/decrypt-secrets.sh
```

## How to encrypt after editing

```bash
bash scripts/encrypt-secrets.sh
```

## Key location

The age private key lives at:
- **Server:** `/root/.config/sops/age/keys.txt`
- **Local:** `~/.config/sops/age/keys.txt`
- **Backup:** stored in ThriveTech password manager under "SOPS age key"

Public key: `age1sd79y4wfft3jmsr68eaaq53ay2fq4kkeu99harytmpa4cud06e6q3xm320`
