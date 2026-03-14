import { NextResponse } from 'next/server';

export async function POST(request) {
  const body = await request.json();

  let backendRes, data;
  try {
    backendRes = await fetch(`${process.env.MAHALAXMI_AUTH_API_URL}/v1/auth/register`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ ...body, verificationBaseUrl: 'https://mahalaxmi.ai' }),
    });
    data = await backendRes.json();
  } catch {
    return NextResponse.json({ success: false, message: 'Service unavailable. Please try again.' }, { status: 503 });
  }

  if (!backendRes.ok || !data.success) {
    return NextResponse.json(data, { status: backendRes.status });
  }

  const response = NextResponse.json({ success: true, user: data.user, message: data.message });
  if (data.token) {
    response.cookies.set('mahalaxmi_token', data.token, {
      httpOnly: true,
      secure: process.env.NODE_ENV === 'production',
      sameSite: 'lax',
      maxAge: 24 * 60 * 60,
      path: '/',
    });
  }
  return response;
}
