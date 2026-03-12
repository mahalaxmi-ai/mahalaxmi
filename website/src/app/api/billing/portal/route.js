import { NextResponse } from 'next/server';
import { cookies } from 'next/headers';

export async function POST(request) {
  const cookieStore = await cookies();
  const token = cookieStore.get('mahalaxmi_token')?.value;
  if (!token) {
    return NextResponse.json({ error: 'Authentication required' }, { status: 401 });
  }

  const platformUrl = process.env.MAHALAXMI_PLATFORM_API_URL;
  const pakKey = process.env.MAHALAXMI_CLOUD_PAK_KEY;

  if (!platformUrl || !pakKey) {
    return NextResponse.json({ available: false }, { status: 200 });
  }

  const userId = request.headers.get('x-user-id') || '';
  const userEmail = request.headers.get('x-user-email') || '';

  try {
    const res = await fetch(`${platformUrl}/api/v1/mahalaxmi/billing/portal`, {
      method: 'POST',
      headers: {
        'X-Channel-API-Key': pakKey,
        'Content-Type': 'application/json',
        'x-user-id': userId,
        'x-user-email': userEmail,
      },
    });

    if (res.status === 404 || res.status === 501) {
      // Billing portal not yet implemented on platform
      return NextResponse.json({ available: false });
    }

    if (!res.ok) {
      return NextResponse.json({ available: false });
    }

    const data = await res.json();
    return NextResponse.json({ portal_url: data.portal_url, available: true });
  } catch {
    return NextResponse.json({ available: false });
  }
}
