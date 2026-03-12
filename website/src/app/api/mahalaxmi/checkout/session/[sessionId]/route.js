import { NextResponse } from 'next/server';
import { cookies } from 'next/headers';

export async function GET(request, { params }) {
  const { sessionId } = await params;
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

  const userId = request.headers.get('x-user-id') || '';
  const userEmail = request.headers.get('x-user-email') || '';

  try {
    const res = await fetch(
      `${platformUrl}/api/v1/mahalaxmi/checkout/session/${sessionId}`,
      {
        headers: {
          'X-Channel-API-Key': pakKey,
          'x-user-id': userId,
          'x-user-email': userEmail,
        },
        cache: 'no-store',
      }
    );

    // Pass through the exact status code — 202 means still provisioning, 200 means terminal
    const data = await res.json();
    return NextResponse.json(data, { status: res.status });
  } catch {
    return NextResponse.json({ error: 'Service unreachable' }, { status: 502 });
  }
}
