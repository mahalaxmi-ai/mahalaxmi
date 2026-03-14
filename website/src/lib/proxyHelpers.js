export function getUserToken(request) {
  const cookieHeader = request.headers.get('cookie') || '';
  return cookieHeader
    .split(';')
    .find(c => c.trim().startsWith('mahalaxmi_token='))
    ?.split('=')[1]?.trim() || null;
}

export function jwtHeaders(token) {
  return {
    'Authorization': `Bearer ${token}`,
    'Content-Type': 'application/json',
  };
}

export function unauthorizedResponse() {
  return Response.json({ error: 'unauthorized' }, { status: 401 });
}
