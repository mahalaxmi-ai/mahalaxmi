import { NextResponse } from 'next/server';
import { cookies } from 'next/headers';

export async function GET() {
  const cookieStore = await cookies();
  const token = cookieStore.get('mahalaxmi_token')?.value;

  if (!token) {
    return NextResponse.json({ user: null, isAuthenticated: false });
  }

  try {
    const backendRes = await fetch(`${process.env.MAHALAXMI_AUTH_API_URL}/v1/auth/me`, {
      headers: { Authorization: `Bearer ${token}` },
    });
    const data = await backendRes.json();
    if (!backendRes.ok || !data.success) {
      return NextResponse.json({ user: null, isAuthenticated: false });
    }
    return NextResponse.json({ user: data.user, isAuthenticated: true });
  } catch {
    return NextResponse.json({ user: null, isAuthenticated: false });
  }
}
