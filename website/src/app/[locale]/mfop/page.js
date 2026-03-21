import { setRequestLocale } from 'next-intl/server';
import { locales } from '@/i18n/routing';
import MfopOverviewContent from './MfopOverviewContent';

export function generateStaticParams() {
  return locales.map((locale) => ({ locale }));
}

export const metadata = {
  title: 'Protocol — MFOP | Mahalaxmi',
  description:
    'Mahalaxmi Federation and Orchestration Protocol (MFOP) — open specification for federated distributed AI orchestration. Read the draft or leave a comment.',
  openGraph: {
    title: 'MFOP — Mahalaxmi Federation and Orchestration Protocol',
    description:
      'Open specification for federated distributed AI orchestration across heterogeneous compute nodes with compliance-zone-aware routing and cryptographically signed billing receipts.',
    type: 'website',
  },
};

export default async function MfopOverviewPage({ params }) {
  const { locale } = await params;
  setRequestLocale(locale);
  return <MfopOverviewContent />;
}
