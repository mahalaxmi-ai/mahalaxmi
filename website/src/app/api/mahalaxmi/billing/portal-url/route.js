import { NextResponse } from 'next/server';
import { cookies } from 'next/headers';

export async function POST() {
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

  try {
    const res = await fetch(`${platformUrl}/api/v1/mahalaxmi/billing/portal-url`, {
      method: 'POST',
      headers: {
        'Authorization': `Bearer ${token}`,
        'Content-Type': 'application/json',
      },
      cache: 'no-store',
    });

    if (!res.ok) {
      const errorBody = await res.text();
      console.error(`[billing/portal-url] platform error ${res.status}`, errorBody);
      return NextResponse.json({ error: 'Billing portal unavailable' }, { status: 502 });
    }

    const data = await res.json();
    return NextResponse.json({ url: data.url });
  } catch {
    return NextResponse.json({ error: 'Billing portal unreachable' }, { status: 502 });
  }
}
