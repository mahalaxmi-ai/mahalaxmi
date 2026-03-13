import { NextResponse } from 'next/server';
import { cookies } from 'next/headers';

async function getAuthenticatedToken() {
  const cookieStore = cookies();
  const token = cookieStore.get('mahalaxmi_token')?.value;
  if (!token) return null;

  const authApiUrl = process.env.MAHALAXMI_AUTH_API_URL;
  if (!authApiUrl) return null;

  try {
    const res = await fetch(`${authApiUrl}/api/v1/auth/me`, {
      headers: { Authorization: `Bearer ${token}` },
      cache: 'no-store',
    });
    if (!res.ok) return null;
    return token;
  } catch {
    return null;
  }
}

export async function GET() {
  const { MAHALAXMI_PLATFORM_API_URL, MAHALAXMI_CLOUD_PAK_KEY } = process.env;
  if (!MAHALAXMI_PLATFORM_API_URL || !MAHALAXMI_CLOUD_PAK_KEY) {
    return NextResponse.json({ error: 'Service unavailable' }, { status: 503 });
  }

  const token = await getAuthenticatedToken();
  if (!token) {
    return NextResponse.json({ error: 'Authentication required' }, { status: 401 });
  }

  try {
    const res = await fetch(`${MAHALAXMI_PLATFORM_API_URL}/api/v1/mahalaxmi/verification/status`, {
      headers: {
        'X-Channel-API-Key': MAHALAXMI_CLOUD_PAK_KEY,
        Cookie: `mahalaxmi_token=${token}`,
      },
      cache: 'no-store',
    });
    const data = await res.json();
    return NextResponse.json(data, { status: res.status });
  } catch (err) {
    console.error('[verification/status] Network error:', err);
    return NextResponse.json({ error: 'Service unreachable' }, { status: 502 });
  }
}
