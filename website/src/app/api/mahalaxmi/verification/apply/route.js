import { getUserToken, jwtHeaders, unauthorizedResponse } from '@/lib/proxyHelpers';

export async function POST(request) {
  const token = getUserToken(request);
  if (!token) return unauthorizedResponse();

  const platformUrl = process.env.MAHALAXMI_PLATFORM_API_URL;
  if (!platformUrl) return Response.json({ error: 'Not configured' }, { status: 503 });

  let body;
  try {
    body = await request.json();
  } catch {
    return Response.json({ error: 'Invalid request body' }, { status: 400 });
  }

  const { tier_id } = body;
  if (!tier_id) {
    return Response.json({ error: 'Missing tier_id' }, { status: 400 });
  }

  const platformRes = await fetch(`${platformUrl}/api/v1/mahalaxmi/verification/apply`, {
    method: 'POST',
    headers: jwtHeaders(token),
    body: JSON.stringify({ tier_id }),
  }).catch(() => null);

  if (!platformRes) return Response.json({ error: 'Service unreachable' }, { status: 502 });

  if (!platformRes.ok) {
    const errorBody = await platformRes.text();
    console.error(`[verification/apply] platform error ${platformRes.status}`, errorBody);
    return Response.json({ error: 'platform_error', detail: errorBody }, { status: platformRes.status });
  }

  const data = await platformRes.json();
  return Response.json(data);
}
