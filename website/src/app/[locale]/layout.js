import { notFound } from 'next/navigation';
import { setRequestLocale } from 'next-intl/server';
import { NextIntlClientProvider } from 'next-intl';
import { getMessages } from 'next-intl/server';
import { AppRouterCacheProvider } from '@mui/material-nextjs/v14-appRouter';
import Providers from '@/contexts/Providers';
import Navbar from '@/components/Layout/Navbar';
import Footer from '@/components/Layout/Footer';
import { locales, rtlLocales } from '@/i18n/routing';
import { getAlternateLanguages } from '@/utils/i18nMetadata';

export function generateStaticParams() {
  return locales.map((locale) => ({ locale }));
}

export async function generateMetadata({ params }) {
  const { locale } = await params;
  return {
    title: {
      default: 'Mahalaxmi AI',
      template: '%s | Mahalaxmi AI',
    },
    description: 'AI-powered terminal and cloud orchestration platform by ThriveTech Services LLC.',
    openGraph: {
      type: 'website',
      locale: locale.replace('-', '_'),
      siteName: 'Mahalaxmi AI',
      images: [{ url: '/mahalaxmi_logo.png' }],
    },
    alternates: {
      languages: getAlternateLanguages(),
    },
  };
}

export default async function LocaleLayout({ children, params }) {
  const { locale } = await params;

  if (!locales.includes(locale)) {
    notFound();
  }

  setRequestLocale(locale);

  const messages = await getMessages();
  const dir = rtlLocales.includes(locale) ? 'rtl' : 'ltr';

  return (
    <html lang={locale} dir={dir}>
      <body>
        <NextIntlClientProvider messages={messages}>
          <AppRouterCacheProvider options={{ key: dir === 'rtl' ? 'mui-rtl' : 'mui' }}>
            <Providers locale={locale} dir={dir}>
              <Navbar />
              <main
                style={{
                  flexGrow: 1,
                  paddingTop: '64px',
                  minHeight: '100vh',
                  display: 'flex',
                  flexDirection: 'column',
                }}
              >
                {children}
              </main>
              <Footer />
            </Providers>
          </AppRouterCacheProvider>
        </NextIntlClientProvider>
      </body>
    </html>
  );
}
