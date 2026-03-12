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

  try {
    await fetch(`${authApiUrl}/api/v1/auth/forgot-password`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ ...body, clientId: 'mahalaxmi' }),
    });
  } catch {
    // Intentionally swallow errors to avoid leaking email existence
  }

  return NextResponse.json({
    success: true,
    message: 'If that email exists, a reset link was sent',
  });
}
