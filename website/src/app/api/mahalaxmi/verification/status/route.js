import { NextResponse } from 'next/server';
import { cookies } from 'next/headers';

export async function GET(request) {
  const cookieStore = await cookies();
  const token = cookieStore.get('mahalaxmi_token')?.value;
  if (!token) {
    return NextResponse.json({ error: 'Authentication required' }, { status: 401 });
  }

  const platformUrl = process.env.MAHALAXMI_PLATFORM_API_URL;
  const pakKey = process.env.MAHALAXMI_CLOUD_PAK_KEY;
  if (!platformUrl || !pakKey) {
    return NextResponse.json({ error: 'Service unavailable' }, { status: 503 });
  }

  const cookieHeader = request.headers.get('cookie') || '';

  try {
    const res = await fetch(`${platformUrl}/api/v1/mahalaxmi/verification/status`, {
      headers: {
        'X-Channel-API-Key': pakKey,
        'Cookie': cookieHeader,
      },
      cache: 'no-store',
    });
    const data = await res.json();
    return NextResponse.json(data, { status: res.status });
  } catch {
    return NextResponse.json({ error: 'Service unreachable' }, { status: 502 });
  }
}
