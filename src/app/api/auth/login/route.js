import { cookies } from 'next/headers';
import { NextResponse } from 'next/server';

export async function POST(request) {
  const authApiUrl = process.env.MAHALAXMI_AUTH_API_URL;
  if (!authApiUrl) {
    return NextResponse.json({ error: 'Auth service unavailable' }, { status: 503 });
  }

  let body;
  try {
    body = await request.json();
  } catch {
    return NextResponse.json({ error: 'Invalid request body' }, { status: 400 });
  }

  const { email, password } = body;

  let response;
  try {
    response = await fetch(`${authApiUrl}/api/v1/auth/login`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ email, password }),
    });
  } catch {
    return NextResponse.json({ error: 'Auth service unavailable' }, { status: 503 });
  }

  if (response.status === 401) {
    return NextResponse.json({ error: 'Invalid email or password' }, { status: 401 });
  }

  if (response.status === 403) {
    return NextResponse.json(
      { error: 'Please verify your email before logging in' },
      { status: 403 }
    );
  }

  if (!response.ok) {
    return NextResponse.json({ error: 'Login failed. Please try again.' }, { status: response.status });
  }

  let data;
  try {
    data = await response.json();
  } catch {
    return NextResponse.json({ error: 'Invalid response from auth service' }, { status: 503 });
  }

  const token = data.token || data.jwt || data.accessToken;
  if (!token) {
    return NextResponse.json({ error: 'Auth service returned no token' }, { status: 503 });
  }

  cookies().set('mahalaxmi_token', token, {
    httpOnly: true,
    secure: true,
    sameSite: 'lax',
    path: '/',
    maxAge: 604800,
  });

  return NextResponse.json({ success: true, user: data.user });
}
