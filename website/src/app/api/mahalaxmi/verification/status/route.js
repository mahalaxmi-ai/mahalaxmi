import { getUserToken, jwtHeaders, unauthorizedResponse } from '@/lib/proxyHelpers';

export async function GET(request) {
  const token = getUserToken(request);
  if (!token) return unauthorizedResponse();

  const platformUrl = process.env.MAHALAXMI_PLATFORM_API_URL;
  if (!platformUrl) return Response.json({ error: 'Not configured' }, { status: 503 });

  const platformRes = await fetch(`${platformUrl}/api/v1/mahalaxmi/verification/status`, {
    headers: jwtHeaders(token),
    cache: 'no-store',
  }).catch(() => null);

  if (!platformRes) return Response.json({ error: 'Service unreachable' }, { status: 502 });

  if (!platformRes.ok) {
    const errorBody = await platformRes.text();
    console.error(`[verification/status] platform error ${platformRes.status}`, errorBody);
    return Response.json({ error: 'platform_error', detail: errorBody }, { status: platformRes.status });
  }

  const data = await platformRes.json();
  return Response.json(data);
}
