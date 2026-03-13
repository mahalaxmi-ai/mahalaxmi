import { NextResponse } from 'next/server';
import { cookies } from 'next/headers';

// Platform endpoint not yet live — returns 501 until confirmed ready.
export async function POST(request, { params }) {
  const { id } = await params;

  const cookieStore = await cookies();
  const token = cookieStore.get('mahalaxmi_token')?.value;
  if (!token) {
    return NextResponse.json({ error: 'Authentication required' }, { status: 401 });
  }

  const platformUrl = process.env.MAHALAXMI_PLATFORM_API_URL;
  const pakKey = process.env.MAHALAXMI_CLOUD_PAK_KEY;

  if (!platformUrl || !pakKey) {
    return NextResponse.json({ error: 'Not implemented' }, { status: 501 });
  }

  // TODO: remove stub and forward when Platform confirms endpoint is live
  // const cookieHeader = request.headers.get('cookie') || '';
  // const userId = request.headers.get('x-user-id') || '';
  // const userEmail = request.headers.get('x-user-email') || '';
  // const res = await fetch(`${platformUrl}/api/v1/mahalaxmi/servers/${id}/stop`, {
  //   method: 'POST',
  //   headers: {
  //     'X-Channel-API-Key': pakKey,
  //     'Cookie': cookieHeader,
  //     'x-user-id': userId,
  //     'x-user-email': userEmail,
  //   },
  // });
  // if (!res.ok) return NextResponse.json({ error: 'Stop failed' }, { status: res.status });
  // return NextResponse.json({}, { status: 202 });

  void id;
  return NextResponse.json({ error: 'Not implemented' }, { status: 501 });
}
