import { NextResponse } from 'next/server';
import { cookies } from 'next/headers';

export async function DELETE(request) {
  const cookieStore = await cookies();
  const token = cookieStore.get('mahalaxmi_token')?.value;
  if (!token) {
    return NextResponse.json({ error: 'Authentication required' }, { status: 401 });
  }

  try {
    const backendRes = await fetch(`${process.env.MAHALAXMI_AUTH_API_URL}/v1/auth/account`, {
      method: 'DELETE',
      headers: {
        Authorization: `Bearer ${token}`,
        'Content-Type': 'application/json',
      },
    });
    const data = await backendRes.json().catch(() => ({}));
    if (!backendRes.ok) {
      return NextResponse.json(data, { status: backendRes.status });
    }
    const response = NextResponse.json({ success: true }, { status: 200 });
    response.cookies.delete('mahalaxmi_token');
    return response;
  } catch {
    return NextResponse.json({ success: false, message: 'Service unavailable. Please try again.' }, { status: 503 });
  }
}
