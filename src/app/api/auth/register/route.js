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
    response = await fetch(`${authApiUrl}/api/v1/auth/register`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ ...body, clientId: 'mahalaxmi' }),
    });
  } catch {
    return NextResponse.json({ error: 'Registration failed. Please try again.' }, { status: 500 });
  }

  if (response.status === 201) {
    return NextResponse.json(
      { success: true, message: 'Check your email to verify your account' },
      { status: 201 }
    );
  }

  if (response.status === 409) {
    return NextResponse.json(
      { error: 'An account with this email already exists' },
      { status: 409 }
    );
  }

  if (response.status >= 500) {
    return NextResponse.json(
      { error: 'Registration failed. Please try again.' },
      { status: 500 }
    );
  }

  let errorData;
  try {
    errorData = await response.json();
  } catch {
    errorData = { error: 'Registration failed. Please try again.' };
  }

  return NextResponse.json(errorData, { status: response.status });
}
