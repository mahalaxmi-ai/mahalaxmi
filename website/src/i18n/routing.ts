import { defineRouting } from 'next-intl/routing';

export const locales = [
  'en-US',
  'es-ES',
  'fr-FR',
  'de-DE',
  'pt-BR',
  'ja-JP',
  'zh-CN',
  'ko-KR',
  'hi-IN',
  'ar-SA',
] as const;

export type Locale = (typeof locales)[number];

export const defaultLocale: Locale = 'en-US';

export const localeNames: Record<Locale, string> = {
  'en-US': 'English',
  'es-ES': 'Español',
  'fr-FR': 'Français',
  'de-DE': 'Deutsch',
  'pt-BR': 'Português',
  'ja-JP': '日本語',
  'zh-CN': '中文',
  'ko-KR': '한국어',
  'hi-IN': 'हिन्दी',
  'ar-SA': 'العربية',
};

export const rtlLocales: Locale[] = ['ar-SA'];

export const routing = defineRouting({
  locales,
  defaultLocale,
  localePrefix: 'as-needed',
});
