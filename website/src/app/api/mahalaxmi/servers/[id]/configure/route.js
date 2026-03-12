import { NextResponse } from 'next/server';
import { cookies } from 'next/headers';

const PROJECT_NAME_REGEX = /^[a-z0-9][a-z0-9-]{1,38}[a-z0-9]$/;

export async function PATCH(request, { params }) {
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

  let body;
  try {
    body = await request.json();
  } catch {
    return NextResponse.json({ error: 'Invalid request body' }, { status: 400 });
  }

  const { project_name } = body;
  if (!project_name || !PROJECT_NAME_REGEX.test(project_name)) {
    return NextResponse.json(
      { error: 'Invalid project name. Must be 3-40 characters, lowercase letters, digits, and hyphens only. Cannot start or end with a hyphen.' },
      { status: 400 }
    );
  }

  const { id } = await params;

  try {
    const res = await fetch(`${platformUrl}/api/v1/mahalaxmi/servers/${id}/configure`, {
      method: 'PATCH',
      headers: {
        Authorization: `Bearer ${pakKey}`,
        'Content-Type': 'application/json',
        'x-user-id': userId,
      },
      body: JSON.stringify({ project_name }),
      cache: 'no-store',
    });

    if (res.status === 409) {
      let errorData;
      try {
        errorData = await res.json();
      } catch {
        return NextResponse.json({ error: 'Conflict error' }, { status: 502 });
      }

      if (errorData?.code === 'name_taken') {
        return NextResponse.json(
          { error: 'That project name is already taken. Please choose another.', code: 'name_taken' },
          { status: 409 }
        );
      }

      if (errorData?.code === 'already_configured') {
        return NextResponse.json(
          { error: 'This server is already configured.', code: 'already_configured' },
          { status: 409 }
        );
      }

      return NextResponse.json({ error: 'Conflict' }, { status: 409 });
    }

    if (!res.ok) {
      return NextResponse.json({ error: 'Configure request failed' }, { status: 502 });
    }

    const data = await res.json();
    return NextResponse.json(data);
  } catch {
    return NextResponse.json({ error: 'Configure service unreachable' }, { status: 502 });
  }
}
