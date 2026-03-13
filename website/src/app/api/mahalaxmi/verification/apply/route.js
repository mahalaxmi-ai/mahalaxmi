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
    return NextResponse.json({ error: 'Service unavailable' }, { status: 503 });
  }

  const cookieHeader = request.headers.get('cookie') || '';

  let body;
  try {
    body = await request.json();
  } catch {
    return NextResponse.json({ error: 'Invalid request body' }, { status: 400 });
  }

  const { tier_id } = body;
  if (!tier_id) {
    return NextResponse.json({ error: 'Missing tier_id' }, { status: 400 });
  }

  try {
    const res = await fetch(`${platformUrl}/api/v1/mahalaxmi/verification/apply`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        'X-Channel-API-Key': pakKey,
        'Cookie': cookieHeader,
      },
      body: JSON.stringify({ tier_id }),
    });
    const data = await res.json();
    return NextResponse.json(data, { status: res.status });
  } catch {
    return NextResponse.json({ error: 'Service unreachable' }, { status: 502 });
  }
}
