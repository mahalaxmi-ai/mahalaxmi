import { NextResponse } from 'next/server';
import { cookies } from 'next/headers';

export async function GET() {
  const platformUrl = process.env.MAHALAXMI_PLATFORM_API_URL;
  const pakKey = process.env.MAHALAXMI_CLOUD_PAK_KEY;

  if (!platformUrl || !pakKey) {
    return NextResponse.json({ error: 'Not configured' }, { status: 503 });
  }

  const cookieStore = await cookies();
  const token = cookieStore.get('mahalaxmi_token')?.value;
  if (!token) {
    return NextResponse.json({ error: 'Authentication required' }, { status: 401 });
  }

  try {
    const res = await fetch(`${platformUrl}/api/v1/mahalaxmi/servers`, {
      headers: {
        'Authorization': `Bearer ${token}`,
      },
      cache: 'no-store',
    });

    if (!res.ok) {
      const errorBody = await res.text();
      console.error(`[servers] platform error ${res.status}`, errorBody);
      return NextResponse.json(
        { error: 'failed_to_load_servers', detail: errorBody },
        { status: res.status }
      );
    }

    const data = await res.json();
    return NextResponse.json(data);
  } catch {
    return NextResponse.json({ error: 'Service unreachable' }, { status: 502 });
  }
}
