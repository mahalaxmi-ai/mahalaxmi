import { NextResponse } from 'next/server';

export async function GET() {
  const platformUrl = process.env.MAHALAXMI_PLATFORM_API_URL;
  const pakKey = process.env.MAHALAXMI_TERMINAL_PAK_KEY;

  if (!platformUrl || !pakKey) {
    return NextResponse.json({ error: 'Releases not configured' }, { status: 503 });
  }

  try {
    const res = await fetch(`${platformUrl}/api/v1/public/releases/latest`, {
      headers: { 'X-Channel-API-Key': pakKey },
      next: { revalidate: 300 },
    });

    if (!res.ok) {
      return NextResponse.json({ error: 'Releases unavailable' }, { status: 502 });
    }

    const data = await res.json();
    return NextResponse.json(data);
  } catch {
    return NextResponse.json({ error: 'Releases service unreachable' }, { status: 502 });
  }
}
