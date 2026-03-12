import { NextResponse } from 'next/server';
import { cookies } from 'next/headers';

export async function GET(request) {
  const cookieStore = cookies();
  const token = cookieStore.get('mahalaxmi_token');
  if (!token) {
    return NextResponse.json(
      { error: 'Authentication required' },
      { status: 401 }
    );
  }

  const { MAHALAXMI_PLATFORM_API_URL, MAHALAXMI_CLOUD_PAK_KEY } = process.env;
  if (!MAHALAXMI_PLATFORM_API_URL || !MAHALAXMI_CLOUD_PAK_KEY) {
    return NextResponse.json(
      { error: 'Billing portal is temporarily unavailable. Please contact support@mahalaxmi.ai' },
      { status: 503 }
    );
  }

  try {
    const res = await fetch(
      `${MAHALAXMI_PLATFORM_API_URL}/api/v1/mahalaxmi/billing/portal`,
      {
        method: 'POST',
        headers: {
          'X-Channel-API-Key': MAHALAXMI_CLOUD_PAK_KEY,
          'x-user-id': request.headers.get('x-user-id') || '',
          'Content-Type': 'application/json'
        },
        body: JSON.stringify({})
      }
    );
    if (!res.ok) throw new Error(`Platform error ${res.status}`);
    const data = await res.json();
    return NextResponse.json({ url: data.url });
  } catch {
    return NextResponse.json(
      { error: 'Unable to access billing portal. Please try again or contact support@mahalaxmi.ai' },
      { status: 502 }
    );
  }
}
