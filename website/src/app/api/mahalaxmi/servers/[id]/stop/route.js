import { NextResponse } from 'next/server';
import { cookies } from 'next/headers';

export async function POST(request, { params }) {
  const { id } = await params;

  const cookieStore = await cookies();
  const token = cookieStore.get('mahalaxmi_token')?.value;
  if (!token) {
    return NextResponse.json({ error: 'Authentication required' }, { status: 401 });
  }

  const platformUrl = process.env.MAHALAXMI_PLATFORM_API_URL;
  const pakKey = process.env.MAHALAXMI_CLOUD_PAK_KEY;

  if (!platformUrl || !pakKey) {
    return NextResponse.json({ error: 'Not configured' }, { status: 503 });
  }

  try {
    const res = await fetch(`${platformUrl}/api/v1/mahalaxmi/servers/${id}/stop`, {
      method: 'POST',
      headers: {
        'Authorization': `Bearer ${token}`,
        'Content-Type': 'application/json',
      },
    });
    if (!res.ok) {
      const errorBody = await res.text();
      console.error(`[servers/${id}/stop] platform error ${res.status}`, errorBody);
      return NextResponse.json({ error: 'Stop failed' }, { status: res.status });
    }
    return NextResponse.json({}, { status: 202 });
  } catch {
    return NextResponse.json({ error: 'Service unreachable' }, { status: 502 });
  }
}
