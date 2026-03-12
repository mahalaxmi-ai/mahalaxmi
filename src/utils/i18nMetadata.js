import { locales, defaultLocale } from '@/i18n/routing';

const SITE_URL = 'https://mahalaxmi.ai';

export function getAlternateLanguages(pathname = '') {
  const languages = {};
  for (const locale of locales) {
    const prefix = locale === defaultLocale ? '' : `/${locale}`;
    languages[locale] = `${SITE_URL}${prefix}${pathname}`;
  }
  languages['x-default'] = `${SITE_URL}${pathname}`;
  return languages;
}

export function getCanonical(locale, pathname = '') {
  const prefix = locale === defaultLocale ? '' : `/${locale}`;
  return `${SITE_URL}${prefix}${pathname}`;
}

export function getOpenGraphLocale(locale) {
  return locale.replace('-', '_');
}
