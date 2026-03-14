import { NextResponse } from 'next/server';

export async function POST(request) {
  const body = await request.json();

  try {
    const backendRes = await fetch(`${process.env.MAHALAXMI_AUTH_API_URL}/v1/auth/forgot-password`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ ...body, resetBaseUrl: 'https://mahalaxmi.ai' }),
    });
    const data = await backendRes.json();
    return NextResponse.json(data, { status: backendRes.status });
  } catch {
    return NextResponse.json({ success: false, message: 'Service unavailable. Please try again.' }, { status: 503 });
  }
}
