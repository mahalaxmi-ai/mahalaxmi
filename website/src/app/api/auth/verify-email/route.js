import { NextResponse } from 'next/server';

export async function GET(request) {
  const { searchParams } = new URL(request.url);
  const token = searchParams.get('token');

  if (!token) {
    return NextResponse.json({ success: false, message: 'Token is required' }, { status: 400 });
  }

  try {
    const backendRes = await fetch(
      `${process.env.MAHALAXMI_AUTH_API_URL}/v1/auth/verify-email?token=${encodeURIComponent(token)}`
    );
    const data = await backendRes.json();
    return NextResponse.json(data, { status: backendRes.status });
  } catch {
    return NextResponse.json({ success: false, message: 'Service unavailable. Please try again.' }, { status: 503 });
  }
}
