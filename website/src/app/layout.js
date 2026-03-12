import './globals.css';

export const metadata = {
  metadataBase: new URL('https://mahalaxmi.ai'),
  icons: {
    icon: '/favicon.ico',
  },
};

export default function RootLayout({ children }) {
  return children;
}
