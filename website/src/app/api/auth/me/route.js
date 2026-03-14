import { NextResponse } from 'next/server';
import { cookies } from 'next/headers';

function parseJwt(token) {
  try {
    return JSON.parse(Buffer.from(token.split('.')[1], 'base64url').toString());
  } catch {
    return null;
  }
}

function isTokenExpired(payload) {
  if (!payload?.exp) return false; // no exp claim — don't assume expired
  return Math.floor(Date.now() / 1000) >= payload.exp;
}

function userFromPayload(payload) {
  if (!payload) return null;
  return payload.user || { email: payload.email || null, id: payload.sub || null };
}

export async function GET() {
  const cookieStore = await cookies();
  const token = cookieStore.get('mahalaxmi_token')?.value;

  if (!token) {
    return NextResponse.json({ user: null, isAuthenticated: false });
  }

  const payload = parseJwt(token);
  if (!payload || isTokenExpired(payload)) {
    return NextResponse.json({ user: null, isAuthenticated: false });
  }

  // Token is locally valid — try backend for up-to-date user data
  try {
    const backendRes = await fetch(`${process.env.MAHALAXMI_AUTH_API_URL}/v1/auth/me`, {
      headers: { Authorization: `Bearer ${token}` },
    });
    const data = await backendRes.json();
    if (backendRes.ok && data.success) {
      return NextResponse.json({ user: data.user, isAuthenticated: true });
    }
    // Backend returned an error (e.g. revoked token) — treat as unauthenticated
    if (backendRes.status === 401 || backendRes.status === 403) {
      return NextResponse.json({ user: null, isAuthenticated: false });
    }
    // Backend is temporarily unavailable — trust the local JWT
    return NextResponse.json({ user: userFromPayload(payload), isAuthenticated: true });
  } catch {
    // Network error — trust the local JWT rather than falsely evicting the user
    return NextResponse.json({ user: userFromPayload(payload), isAuthenticated: true });
  }
}
