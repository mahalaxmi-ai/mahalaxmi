import { cookies } from 'next/headers';
import { NextResponse } from 'next/server';

export async function GET() {
  const authApiUrl = process.env.MAHALAXMI_AUTH_API_URL;
  if (!authApiUrl) {
    return NextResponse.json({ error: 'Auth service unavailable' }, { status: 503 });
  }

  const cookieStore = cookies();
  const token = cookieStore.get('mahalaxmi_token')?.value;

  if (!token) {
    return NextResponse.json({ user: null, isAuthenticated: false });
  }

  let response;
  try {
    response = await fetch(`${authApiUrl}/api/v1/auth/me`, {
      headers: { Authorization: `Bearer ${token}` },
    });
  } catch {
    return NextResponse.json({ error: 'Auth service unavailable' }, { status: 503 });
  }

  if (response.status === 401) {
    const cookieStore2 = cookies();
    cookieStore2.delete('mahalaxmi_token');
    return NextResponse.json({ user: null, isAuthenticated: false });
  }

  if (!response.ok) {
    return NextResponse.json({ error: 'Auth service error' }, { status: 503 });
  }

  let data;
  try {
    data = await response.json();
  } catch {
    return NextResponse.json({ error: 'Invalid response from auth service' }, { status: 503 });
  }

  return NextResponse.json({ user: data.user, isAuthenticated: true });
}
