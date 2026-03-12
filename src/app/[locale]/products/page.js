import { Suspense } from 'react';
import { setRequestLocale, getTranslations } from 'next-intl/server';
import { getAlternateLanguages, getCanonical, getOpenGraphLocale } from '@/utils/i18nMetadata';
import { Box, CircularProgress } from '@mui/material';
import ProductsContent from './ProductsContent';

export const dynamic = 'force-dynamic';

export async function generateMetadata({ params }) {
  const { locale } = await params;
  const t = await getTranslations({ locale, namespace: 'metadata' });

  return {
    title: t('productsTitle'),
    description: t('productsDescription'),
    alternates: {
      canonical: getCanonical(locale, '/products'),
      languages: getAlternateLanguages('/products'),
    },
    openGraph: {
      title: t('productsTitle'),
      description: t('productsDescription'),
      url: '/products',
      images: [{ url: '/mahalaxmi-logo.png' }],
      locale: getOpenGraphLocale(locale),
    },
  };
}

export default async function ProductsPage({ params }) {
  const { locale } = await params;
  setRequestLocale(locale);

  return (
    <Suspense
      fallback={
        <Box display="flex" justifyContent="center" p={4}>
          <CircularProgress />
        </Box>
      }
    >
      <ProductsContent />
    </Suspense>
  );
}
