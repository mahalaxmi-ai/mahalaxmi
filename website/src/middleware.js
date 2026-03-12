import createMiddleware from 'next-intl/middleware';
import { NextResponse } from 'next/server';
import { routing } from './i18n/routing';

const intlMiddleware = createMiddleware(routing);

const PROTECTED_PATTERNS = ['/dashboard', '/account'];

export default function middleware(request) {
  const { pathname } = request.nextUrl;

  // Strip locale prefix to check if route is protected
  const localePattern = /^\/([a-z]{2}-[A-Z]{2})(\/.*)?$/;
  const match = pathname.match(localePattern);
  const pathWithoutLocale = match ? (match[2] || '/') : pathname;

  const isProtected = PROTECTED_PATTERNS.some((p) => pathWithoutLocale.startsWith(p));

  if (isProtected) {
    const token = request.cookies.get('mahalaxmi_token');
    if (!token) {
      const loginUrl = new URL('/login', request.url);
      loginUrl.searchParams.set('redirect', pathWithoutLocale);
      return NextResponse.redirect(loginUrl);
    }
  }

  return intlMiddleware(request);
}

export const config = {
  matcher: [
    '/((?!_next|_vercel|.*\\..*).*)',
    '/([\\w-]+)?/dashboard(.*)?',
    '/([\\w-]+)?/account(.*)?',
  ],
};
