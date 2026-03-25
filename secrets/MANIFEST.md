# Secrets Manifest — mahalaxmi-oss (Public Repo)

This is the public open-source repository. No actual secrets are stored here.
This directory contains the SOPS infrastructure scaffold only.

## Pattern

All ThriveTech private repos use SOPS + age encryption to store secrets alongside the
code they belong to. The pattern is:

```
secrets/
  .gitkeep           # tracks the directory in git
  MANIFEST.md        # documents what secrets exist and where they decrypt to
  *.enc              # SOPS-encrypted secret files (safe to commit)
```

Encrypted files (`.enc`) are committed to git. Plaintext files are never committed.

## Encrypted file naming convention

| Encrypted file | Decrypts to |
|---|---|
| `secrets/.env.enc` | deployment path `.env` |
| `secrets/.env.prod.enc` | deployment path `.env.prod` |
| `secrets/docker-compose.prod.enc` | deployment path `docker-compose.yml` |
| `secrets/nginx.<name>.conf.enc` | `/etc/nginx/sites-enabled/<name>.conf` |

## How to encrypt a secret file

```bash
# Ensure SOPS_AGE_KEY_FILE is set or ~/.config/sops/age/keys.txt exists
export SOPS_AGE_KEY_FILE=~/.config/sops/age/keys.txt

sops --encrypt path/to/plaintext.env > secrets/plaintext.env.enc
git add secrets/plaintext.env.enc
git commit -m "chore: update encrypted secrets"
```

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
- Server: `/root/.config/sops/age/keys.txt`
- Local: `~/.config/sops/age/keys.txt`
- Backup: stored in ThriveTech password manager under "SOPS age key"
