import type { Metadata } from 'next';
import { AppRouterCacheProvider } from '@mui/material-nextjs/v14-appRouter';
import { getLocale } from 'next-intl/server';
import './globals.css';
import Providers from './Providers';

export const metadata: Metadata = {
  title: 'Mahalaxmi AI',
  description: 'Multi-agent AI orchestration platform by ThriveTech Services LLC.',
};

export default async function RootLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  const locale = await getLocale();

  return (
    <html lang={locale}>
      <body>
        <AppRouterCacheProvider>
          <Providers>{children}</Providers>
        </AppRouterCacheProvider>
      </body>
    </html>
  );
}
