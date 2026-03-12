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
