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

  let response;
  try {
    response = await fetch(`${authApiUrl}/api/v1/auth/reset-password`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(body),
    });
  } catch {
    return NextResponse.json({ error: 'Auth service unavailable' }, { status: 503 });
  }

  if (response.ok) {
    return NextResponse.json({ success: true });
  }

  if (response.status === 400 || response.status === 410) {
    let errorData;
    try {
      errorData = await response.json();
    } catch {
      errorData = { error: 'Invalid or expired reset token' };
    }
    return NextResponse.json(errorData, { status: response.status });
  }

  return NextResponse.json({ error: 'Password reset failed. Please try again.' }, { status: 500 });
}
