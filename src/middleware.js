import createIntlMiddleware from 'next-intl/middleware';
import { NextResponse } from 'next/server';
import { routing } from '@/i18n/routing';

const intlMiddleware = createIntlMiddleware(routing);
const PROTECTED = ['/dashboard', '/account'];

export function middleware(request) {
  const { pathname } = request.nextUrl;
  if (PROTECTED.some(p => pathname.includes(p))) {
    if (!request.cookies.get('mahalaxmi_token')) {
      const url = new URL('/login', request.url);
      url.searchParams.set('redirect', pathname);
      return NextResponse.redirect(url);
    }
  }
  return intlMiddleware(request);
}

export const config = {
  matcher: ['/((?!_next|api|favicon.ico|.*\\..*).*)']
};
