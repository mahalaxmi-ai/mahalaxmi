import { NextResponse } from 'next/server';
import { cookies } from 'next/headers';

export async function PATCH(request, { params }) {
  const { id } = await params;
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

  let body;
  try {
    body = await request.json();
  } catch {
    return NextResponse.json({ error: 'Invalid request body' }, { status: 400 });
  }

  const { project_name } = body;
  if (!project_name || typeof project_name !== 'string') {
    return NextResponse.json({ error: 'project_name is required' }, { status: 400 });
  }

  try {
    const res = await fetch(`${platformUrl}/api/v1/mahalaxmi/servers/${id}/configure`, {
      method: 'PATCH',
      headers: {
        'Authorization': `Bearer ${token}`,
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({ project_name }),
    });

    if (res.status === 409) {
      const conflictData = await res.json().catch(() => ({}));
      return NextResponse.json(conflictData, { status: 409 });
    }

    if (!res.ok) {
      const errorBody = await res.text();
      console.error(`[servers/${id}/configure] platform error ${res.status}`, errorBody);
      return NextResponse.json({ error: 'Configuration failed' }, { status: 502 });
    }

    const data = await res.json();
    return NextResponse.json(data, { status: 200 });
  } catch {
    return NextResponse.json({ error: 'Service unreachable' }, { status: 502 });
  }
}
