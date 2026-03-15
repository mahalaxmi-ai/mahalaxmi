import { getUserToken, jwtHeaders, unauthorizedResponse } from '@/lib/proxyHelpers';

export async function PATCH(request, { params }) {
  const token = getUserToken(request);
  if (!token) return unauthorizedResponse();
  const { id } = await params;
  const body = await request.json();

  const platformRes = await fetch(
    `${process.env.MAHALAXMI_PLATFORM_API_URL}/api/v1/mahalaxmi/servers/${id}/timeout`,
    {
      method: 'PATCH',
      headers: jwtHeaders(token),
      body: JSON.stringify(body),
    }
  );

  if (!platformRes.ok) {
    const error = await platformRes.text();
    console.error(`[timeout] platform error ${platformRes.status}`, error);
    return Response.json(
      { error: 'timeout_update_failed', detail: error },
      { status: platformRes.status }
    );
  }

  return Response.json(await platformRes.json());
}
