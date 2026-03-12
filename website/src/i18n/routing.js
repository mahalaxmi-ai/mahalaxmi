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
];

export const defaultLocale = 'en-US';

export const localeNames = {
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

export const rtlLocales = ['ar-SA'];

export const routing = defineRouting({
  locales,
  defaultLocale,
  localePrefix: 'as-needed',
});
