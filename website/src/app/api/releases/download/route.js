import { NextResponse } from 'next/server';

export async function GET(request) {
  const platformUrl = process.env.MAHALAXMI_PLATFORM_API_URL;
  const pakKey = process.env.MAHALAXMI_TERMINAL_PAK_KEY;

  if (!platformUrl || !pakKey) {
    return NextResponse.json({ error: 'Not configured' }, { status: 503 });
  }

  const { searchParams } = new URL(request.url);
  const releaseId = searchParams.get('id');
  if (!releaseId) {
    return NextResponse.json({ error: 'Missing release id' }, { status: 400 });
  }

  try {
    const upstream = await fetch(
      `${platformUrl}/api/v1/public/releases/${releaseId}/download`,
      { headers: { 'X-Channel-API-Key': pakKey } }
    );

    if (!upstream.ok) {
      return NextResponse.json({ error: 'Download unavailable' }, { status: upstream.status });
    }

    const contentType = upstream.headers.get('content-type') || 'application/octet-stream';
    const contentDisposition = upstream.headers.get('content-disposition') || '';
    const contentLength = upstream.headers.get('content-length') || '';

    const headers = new Headers();
    headers.set('content-type', contentType);
    if (contentDisposition) headers.set('content-disposition', contentDisposition);
    if (contentLength) headers.set('content-length', contentLength);

    return new Response(upstream.body, { status: 200, headers });
  } catch {
    return NextResponse.json({ error: 'Download service unreachable' }, { status: 502 });
  }
}
