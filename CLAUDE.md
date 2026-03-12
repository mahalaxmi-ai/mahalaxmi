# Mahalaxmi Website

## Project Overview
Website for Mahalaxmi, developed under ThriveTech.

## Development Guidelines
- Keep code simple and maintainable
- Follow semantic HTML practices
- Ensure responsive design across devices

## Commands

```bash
# Development
cd website && npm run dev

# Production build
cd website && npm run build

# Run production build output
cd website && npm start
```

## Phase 0 Completion Status

**Gate run timestamp:** 2026-03-12T06:30:00Z
**Ran by:** worker-44 (task-44)

| Check | Result |
|-------|--------|
| `npx next build` exits 0 | ❌ FAIL — `package.json` / Next.js scaffolding missing from `website/` |
| No `thrivetechservice.com` in `src/` | ✅ PASS |
| `public/mahalaxmi_logo.png` exists | ❌ FAIL — not present |
| `public/mahalaxmi_logo.jpg` exists | ❌ FAIL — not present |
| `messages/` has 10+ locale files | ❌ FAIL — directory absent |

**Overall Phase 0 gate:** NOT PASSED

Full details in [`docs/phase0-build-status.md`](docs/phase0-build-status.md).

The domain-hygiene check (no `thrivetechservice.com` strings) passes cleanly.
Remaining failures are infrastructure gaps: the base Next.js project (`package.json`,
`next.config.*`, `node_modules`, `public/`, `messages/`) has not yet been
committed to the repository. Once those assets are in place, re-run
`npx next build` from `website/` to confirm exit 0.

## Notes

- All backend API URLs must use `process.env.MAHALAXMI_*_API_URL` — never hardcode domain strings
- Auth token stored in `mahalaxmi_token` httpOnly cookie; never expose PAK keys to the browser
- Docker standalone build targets port 4025 behind Nginx on 5.161.189.182

---

## Mahalaxmi Cloud Checkout Contract (LOCKED — do not deviate)

Mahalaxmi Cloud is sold from both mahalaxmi.ai and thrivetechservice.com. Both sites call the same endpoint. This contract is final.

### Checkout Session Creation

```
POST https://tokenapi.thrivetechservice.com/api/v1/mahalaxmi/checkout/session
Headers:
  X-Channel-API-Key: <MAHALAXMI_CLOUD_PAK_KEY>
  Content-Type: application/json
Body:
{
  "tier": "cloud-solo" | "cloud-builder" | "cloud-power" | "cloud-team",
  "cloud_provider": "hetzner",
  "billing_cycle": "monthly" | "annual",
  "success_url": "https://mahalaxmi.ai/checkout/success?session_id={CHECKOUT_SESSION_ID}",
  "cancel_url": "<see below>"
}
Response: { "checkout_url": "https://checkout.stripe.com/..." }
```

Redirect the customer to `checkout_url`. Stripe handles everything from there.

### URLs — mahalaxmi.ai specific

```
success_url: https://mahalaxmi.ai/checkout/success?session_id={CHECKOUT_SESSION_ID}
cancel_url:  https://mahalaxmi.ai/cloud/pricing
```

### Auth Gate — mahalaxmi.ai

1. Customer clicks Buy on mahalaxmi.ai/cloud/pricing
2. Check if customer has a `mahalaxmi_token` cookie
3. If not authenticated → redirect to `/register?redirect=/cloud/pricing`
4. After registration/login → customer returns to pricing → clicks Buy again
5. Now authenticated → call checkout endpoint server-side → redirect to Stripe

### cloud_provider Field

- Launch: always send `"cloud_provider": "hetzner"` — only active provider
- Future: pricing page will show AWS/GCP selector; same endpoint, different value — no other changes needed

### mahalaxmi.ai Team Tasks

- Buy button on `/cloud/pricing` calls checkout endpoint server-side using `MAHALAXMI_CLOUD_PAK_KEY`
- Auth gate: if no `mahalaxmi_token` → redirect to `/register?redirect=/cloud/pricing`
- Always pass `success_url: https://mahalaxmi.ai/checkout/success?session_id={CHECKOUT_SESSION_ID}`
- Always pass `cancel_url: https://mahalaxmi.ai/cloud/pricing`
- `/checkout/success` page is already built and locked — no changes needed there
