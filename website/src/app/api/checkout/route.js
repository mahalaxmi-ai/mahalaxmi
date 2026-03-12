import { NextResponse } from 'next/server';
import { cookies } from 'next/headers';

export async function GET() {
  const platformUrl = process.env.MAHALAXMI_PLATFORM_API_URL;
  const pakKey = process.env.MAHALAXMI_CLOUD_PAK_KEY;

  if (!platformUrl || !pakKey) {
    return NextResponse.json({ error: 'Pricing not configured' }, { status: 503 });
  }

  try {
    const res = await fetch(`${platformUrl}/api/v1/public/product`, {
      headers: { 'X-Channel-API-Key': pakKey },
      next: { revalidate: 60 },
    });

    if (!res.ok) {
      return NextResponse.json({ error: 'Pricing unavailable' }, { status: 502 });
    }

    const data = await res.json();
    return NextResponse.json(data);
  } catch {
    return NextResponse.json({ error: 'Pricing unavailable' }, { status: 502 });
  }
}

export async function POST(request) {
  const platformUrl = process.env.MAHALAXMI_PLATFORM_API_URL;
  const pakKey = process.env.MAHALAXMI_CLOUD_PAK_KEY;

  if (!platformUrl || !pakKey) {
    return NextResponse.json({ error: 'Checkout not configured' }, { status: 503 });
  }

  const cookieStore = await cookies();
  const token = cookieStore.get('mahalaxmi_token')?.value;
  if (!token) {
    return NextResponse.json({ error: 'Authentication required' }, { status: 401 });
  }

  let body;
  try {
    body = await request.json();
  } catch {
    return NextResponse.json({ error: 'Invalid request body' }, { status: 400 });
  }

  const { tier, billing_cycle, success_url, cancel_url } = body;
  if (!tier || !success_url || !cancel_url) {
    return NextResponse.json(
      { error: 'Missing required fields: tier, success_url, cancel_url' },
      { status: 400 }
    );
  }

  const billingCycle = billing_cycle === 'annual' ? 'annual' : 'monthly';

  const userId = request.headers.get('x-user-id') || '';
  const userEmail = request.headers.get('x-user-email') || '';

  try {
    const res = await fetch(`${platformUrl}/api/v1/mahalaxmi/checkout/session`, {
      method: 'POST',
      headers: {
        Authorization: `Bearer ${pakKey}`,
        'Content-Type': 'application/json',
        'x-user-id': userId,
        'x-user-email': userEmail,
      },
      body: JSON.stringify({ tier, billing_cycle: billingCycle, success_url, cancel_url }),
    });

    if (!res.ok) {
      return NextResponse.json({ error: 'Checkout unavailable' }, { status: 502 });
    }

    const data = await res.json();
    return NextResponse.json({ checkout_url: data.checkout_url });
  } catch {
    return NextResponse.json({ error: 'Checkout service unreachable' }, { status: 502 });
  }
}
