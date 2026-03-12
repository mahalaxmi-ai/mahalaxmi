import { NextResponse } from 'next/server';
import { cookies } from 'next/headers';

export async function DELETE(request, { params }) {
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
    const res = await fetch(`${platformUrl}/api/v1/mahalaxmi/projects/${id}`, {
      method: 'DELETE',
      headers: {
        Authorization: `Bearer ${pakKey}`,
        'x-user-id': userId,
      },
      cache: 'no-store',
    });

    if (res.status === 404) {
      return NextResponse.json({ error: 'Project not found' }, { status: 404 });
    }

    if (!res.ok) {
      return NextResponse.json({ error: 'Delete request failed' }, { status: 502 });
    }

    const data = await res.json();
    return NextResponse.json(data, { status: 202 });
  } catch {
    return NextResponse.json({ error: 'Delete service unreachable' }, { status: 502 });
  }
}
