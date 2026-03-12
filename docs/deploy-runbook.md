# Mahalaxmi Website — Production Deployment Runbook

**Domain:** mahalaxmi.ai
**Server:** 5.161.189.182
**Deploy path:** /opt/deployments/mahalaxmi-website
**Port:** 4025 (behind Nginx)
**Repo:** https://github.com/thrivetech2t/mahalaxmi-website.git
**Integration branch:** integration

---

## Pre-Deploy Smoke Test Checklist (15 checks — all must pass)

Run these against the integration branch before every production deploy. Zero tolerance for failures.

| # | Check | Expected Result |
|---|-------|-----------------|
| 1 | `npx next build` on integration branch | Zero errors, zero warnings |
| 2 | `grep -r 'thrivetechservice.com' src/` | Zero results (no user-visible strings) |
| 3 | Register → branded verification email → verify → login → `/dashboard/servers` loads | Full flow completes; Mahalaxmi-branded email received |
| 4 | `/dashboard/servers` empty state | Page loads, zero console errors |
| 5 | Server card "Open in VS Code" | Calls `/api/mahalaxmi/servers/:id/vscode-config`; uses `deep_link` value directly |
| 6 | `/products` | All 3 products render with live pricing from Platform API |
| 7 | `/products/mahalaxmi-ai-terminal-orchestration` | Download CTA visible and points to GitHub releases URL |
| 8 | `/products/mahalaxmi-headless-orchestration` | "Get Started" CTA links to `/cloud/pricing` |
| 9 | `/products/mahalaxmi-vscode-extension` | "Install in VS Code" CTA visible |
| 10 | "Buy Now" on `/cloud/pricing` (unauthenticated) | Redirects to `/login` |
| 11 | `/open-source` | GitHub link and MIT badge both render |
| 12 | `/open-source/security` | Page shows `security@mahalaxmi.ai` |
| 13 | `/docs/quickstart` | 3-command quick start block renders |
| 14 | Footer | `legal@mahalaxmi.ai` link is present on every page |
| 15 | Env vars | All six vars confirmed in `.env.local` and `docker-compose.yml` on the production server (see Section 3 below) |

All 15 checks must pass before proceeding to deployment.

---

## Required Environment Variables

All six must be present in both `.env.local` and the `environment:` block of `docker-compose.yml` on the production server before the container starts.

```
MAHALAXMI_AUTH_API_URL=https://tokenapi.thrivetechservice.com
MAHALAXMI_PLATFORM_API_URL=https://tokenapi.thrivetechservice.com
MAHALAXMI_ACTIVATION_API_URL=https://tokenadmin.thrivetechservice.com
MAHALAXMI_CLOUD_PAK_KEY=pak_live_...        # mahalaxmi-headless-orchestration
MAHALAXMI_TERMINAL_PAK_KEY=pak_live_...     # mahalaxmi-ai-terminal-orchestration
MAHALAXMI_VSCODE_PAK_KEY=pak_live_...       # mahalaxmi-vscode-extension
```

`AUTH_API_URL` and `PLATFORM_API_URL` currently point to the same server. Both are defined separately so infrastructure can be split without a code change.
`MAHALAXMI_TERMINAL_PAK_KEY` value lives in `thrivetechWebsite/backend/.env` under `PRODUCT_PLATFORM_PAK_KEYS` — copy it, do not generate a new one.

---

## Deployment Steps

### Step 1 — SSH to the production server

```bash
ssh root@5.161.189.182
```

### Step 2 — Prepare the deploy directory

```bash
mkdir -p /opt/deployments/mahalaxmi-website
cd /opt/deployments/mahalaxmi-website
```

### Step 3 — Pull the latest code

**First deploy (directory is empty):**

```bash
git clone https://github.com/thrivetech2t/mahalaxmi-website.git .
git checkout integration
```

**Subsequent deploys:**

```bash
git pull origin integration
```

### Step 4 — Write the .env.local file

Create `/opt/deployments/mahalaxmi-website/.env.local` with all six production values (see Section above). The file must not be committed to git.

Verify all six are present:

```bash
grep -c 'MAHALAXMI_' .env.local
# Expected output: 6
```

Verify the same six appear in `docker-compose.yml`:

```bash
grep 'MAHALAXMI_' docker-compose.yml | wc -l
# Expected output: 6
```

### Step 5 — Build and start the container

```bash
docker compose up --build -d
```

### Step 6 — Wait for the healthcheck to pass

```bash
docker compose ps
```

The `mahalaxmi-web` service must show `healthy` status. Re-run until healthy — do not proceed while status is `starting` or `unhealthy`.

### Step 7 — Confirm the site is live

```bash
curl -I https://mahalaxmi.ai
```

Expected: `HTTP/2 200`. There must be no redirect to `thrivetechservice.com`. Confirm the `Location` header is absent (or points within `mahalaxmi.ai`).

If a redirect to `thrivetechservice.com` appears, **stop** — do not cut over traffic. Investigate Nginx config and re-check.

### Step 8 — 5-minute stability window

Wait at least 5 minutes with the new container healthy and serving traffic. Monitor logs:

```bash
docker compose logs -f --tail=100
```

Watch for 5xx errors or auth failures. If the log is clean after 5 minutes, proceed.

### Step 9 — Stop the old container

**Only after the new deployment is confirmed healthy for 5 minutes:**

```bash
cd /opt/deployments/thrivetech-website
docker compose stop mahalaxmi-web
```

Do not delete the old container or its volumes until the new deployment has been stable for at least 24 hours.

---

## Rollback Procedure

If any step fails or the site is unhealthy after step 7:

1. Restart the old container immediately:
   ```bash
   cd /opt/deployments/thrivetech-website
   docker compose start mahalaxmi-web
   ```
2. Stop the new (broken) container:
   ```bash
   cd /opt/deployments/mahalaxmi-website
   docker compose down
   ```
3. Investigate logs before retrying:
   ```bash
   docker compose logs mahalaxmi-web 2>&1 | tail -100
   ```

---

## Post-Deploy Verification

After completing all steps, re-run the critical path manually:

1. Open https://mahalaxmi.ai — confirm branding (no ThriveTech references)
2. Check footer renders `legal@mahalaxmi.ai`
3. Visit `/products` — confirm all three products show live pricing
4. Visit `/open-source/security` — confirm `security@mahalaxmi.ai` is displayed
5. Confirm `/cloud/pricing` → "Buy Now" redirects unauthenticated users to `/login`

---

## Recurring Deploy Notes

- Never expose `MAHALAXMI_AUTH_API_URL`, `MAHALAXMI_PLATFORM_API_URL`, or any PAK key to the browser. All are server-side only.
- The httpOnly cookie `mahalaxmi_token` is scoped to `mahalaxmi.ai` — do not change the cookie domain.
- The VS Code deep link uses the `api_key` parameter (not `license_key`). If a future deploy changes this, update extension.ts:429 at the same time.
- All auth calls pass `client_id: "mahalaxmi"` to trigger Mahalaxmi-branded emails from the Platform auth API.
- `@thrivetechservice.com` must never appear in any user-visible string. Run check #2 before every deploy.
