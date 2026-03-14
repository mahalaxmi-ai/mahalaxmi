import { getUserToken, jwtHeaders, unauthorizedResponse } from '@/lib/proxyHelpers';

export async function PATCH(request, { params }) {
  const { id } = await params;
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

  const { project_name } = body;
  if (!project_name || typeof project_name !== 'string') {
    return Response.json({ error: 'project_name is required' }, { status: 400 });
  }

  const platformRes = await fetch(`${platformUrl}/api/v1/mahalaxmi/servers/${id}/configure`, {
    method: 'PATCH',
    headers: jwtHeaders(token),
    body: JSON.stringify({ project_name }),
  }).catch(() => null);

  if (!platformRes) return Response.json({ error: 'Service unreachable' }, { status: 502 });

  if (platformRes.status === 409) {
    const conflictData = await platformRes.json().catch(() => ({}));
    return Response.json(conflictData, { status: 409 });
  }

  if (!platformRes.ok) {
    const errorBody = await platformRes.text();
    console.error(`[servers/${id}/configure] platform error ${platformRes.status}`, errorBody);
    return Response.json({ error: 'platform_error', detail: errorBody }, { status: platformRes.status });
  }

  const data = await platformRes.json();
  return Response.json(data);
}
