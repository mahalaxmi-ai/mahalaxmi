import { getUserToken, jwtHeaders, unauthorizedResponse } from '@/lib/proxyHelpers';

export async function DELETE(request, { params }) {
  const { id } = await params;
  const token = getUserToken(request);
  if (!token) return unauthorizedResponse();

  const platformUrl = process.env.MAHALAXMI_PLATFORM_API_URL;
  if (!platformUrl) return Response.json({ error: 'Not configured' }, { status: 503 });

  const platformRes = await fetch(`${platformUrl}/api/v1/mahalaxmi/projects/${id}`, {
    method: 'DELETE',
    headers: jwtHeaders(token),
  }).catch(() => null);

  if (!platformRes) return Response.json({ error: 'Service unreachable' }, { status: 502 });

  if (!platformRes.ok) {
    const errorBody = await platformRes.text();
    console.error(`[projects/${id}] platform error ${platformRes.status}`, errorBody);
    return Response.json({ error: 'platform_error', detail: errorBody }, { status: platformRes.status });
  }

  const data = await platformRes.json();
  return Response.json(data, { status: 202 });
}
