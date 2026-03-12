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

  try {
    const res = await fetch(
      `${platformUrl}/api/v1/mahalaxmi/checkout/session/${sessionId}`,
      {
        headers: {
          Authorization: `Bearer ${pakKey}`,
          'x-user-id': userId,
        },
        cache: 'no-store',
      }
    );

    if (!res.ok) {
      return NextResponse.json({ error: 'Session unavailable' }, { status: 502 });
    }

    const data = await res.json();
    return NextResponse.json(data);
  } catch {
    return NextResponse.json({ error: 'Service unreachable' }, { status: 502 });
  }
}
