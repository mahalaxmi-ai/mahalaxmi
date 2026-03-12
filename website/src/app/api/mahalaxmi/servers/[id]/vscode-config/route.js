import { NextResponse } from 'next/server';
import { cookies } from 'next/headers';

export async function GET(request, { params }) {
  const platformUrl = process.env.MAHALAXMI_PLATFORM_API_URL;
  const pakKey = process.env.MAHALAXMI_CLOUD_PAK_KEY;

  if (!platformUrl || !pakKey) {
    return NextResponse.json({ error: 'Service not configured' }, { status: 503 });
  }

  const cookieStore = await cookies();
  const token = cookieStore.get('mahalaxmi_token')?.value;
  if (!token) {
    return NextResponse.json({ error: 'Authentication required' }, { status: 401 });
  }

  const userId = request.headers.get('x-user-id') || '';
  const { id } = await params;

  try {
    const res = await fetch(`${platformUrl}/api/v1/mahalaxmi/servers/${id}/vscode-config`, {
      headers: {
        Authorization: `Bearer ${pakKey}`,
        'x-user-id': userId,
      },
      cache: 'no-store',
    });

    if (!res.ok) {
      return NextResponse.json({ error: 'VSCode config unavailable' }, { status: 502 });
    }

    const data = await res.json();
    return NextResponse.json({ deep_link: data.deep_link, config_json: data.config_json });
  } catch {
    return NextResponse.json({ error: 'VSCode config service unreachable' }, { status: 502 });
  }
}
