import { NextResponse } from 'next/server';
import { cookies } from 'next/headers';

export async function POST(request) {
  const platformUrl = process.env.MAHALAXMI_PLATFORM_API_URL;
  const pakKey = process.env.MAHALAXMI_CLOUD_PAK_KEY;

  if (!platformUrl || !pakKey) {
    return NextResponse.json({ error: 'Billing not configured' }, { status: 503 });
  }

  const cookieStore = await cookies();
  const token = cookieStore.get('mahalaxmi_token')?.value;
  if (!token) {
    return NextResponse.json({ error: 'Authentication required' }, { status: 401 });
  }

  const cookieHeader = request.headers.get('cookie') || '';
  const userId = request.headers.get('x-user-id') || '';

  try {
    const res = await fetch(`${platformUrl}/api/v1/mahalaxmi/billing/portal-url`, {
      method: 'POST',
      headers: {
        'X-Channel-API-Key': pakKey,
        'Content-Type': 'application/json',
        'Cookie': cookieHeader,
        'x-user-id': userId,
      },
      cache: 'no-store',
    });

    if (!res.ok) {
      return NextResponse.json({ error: 'Billing portal unavailable' }, { status: 502 });
    }

    const data = await res.json();
    return NextResponse.json({ url: data.url });
  } catch {
    return NextResponse.json({ error: 'Billing portal unreachable' }, { status: 502 });
  }
}
