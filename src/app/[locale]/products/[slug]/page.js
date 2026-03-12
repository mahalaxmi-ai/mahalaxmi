import { notFound } from 'next/navigation';
import { setRequestLocale } from 'next-intl/server';
import { getAlternateLanguages, getCanonical, getOpenGraphLocale } from '@/utils/i18nMetadata';
import { fetchProductBySlug } from '@/lib/serverApi';
import JsonLd from '@/components/SEO/JsonLd';
import { productSchema, softwareApplicationSchema, faqPageSchema, breadcrumbSchema } from '@/utils/seoSchemas';
import ProductDetailContent from './ProductDetailContent';

export const revalidate = 3600;

export async function generateMetadata({ params }) {
  const { locale, slug } = await params;
  const product = await fetchProductBySlug(slug);

  if (!product) {
    return { title: 'Product Not Found' };
  }

  return {
    title: product.name,
    description: product.short_description,
    alternates: {
      canonical: getCanonical(locale, `/products/${slug}`),
      languages: getAlternateLanguages(`/products/${slug}`),
    },
    openGraph: {
      title: product.name,
      description: product.short_description,
      url: `/products/${slug}`,
      images: [{ url: product.image || '/mahalaxmi-logo.png' }],
      locale: getOpenGraphLocale(locale),
    },
  };
}

export default async function ProductDetailPage({ params }) {
  const { locale, slug } = await params;
  setRequestLocale(locale);

  const product = await fetchProductBySlug(slug);

  if (!product) {
    notFound();
  }

  return (
    <>
      <JsonLd data={productSchema(product)} />
      {product.category_name !== 'Consulting' && <JsonLd data={softwareApplicationSchema(product)} />}
      {product.faqs && product.faqs.length > 0 && <JsonLd data={faqPageSchema(product.faqs)} />}
      <JsonLd data={breadcrumbSchema([
        { name: 'Home', url: '/' },
        { name: 'Products', url: '/products' },
        { name: product.category_name || 'Category', url: '/products' },
        { name: product.name },
      ])} />
      <ProductDetailContent product={product} slug={slug} />
    </>
  );
}
