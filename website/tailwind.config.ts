import type { Config } from 'tailwindcss'

const config: Config = {
  content: ['./src/**/*.{ts,tsx,js,jsx}'],
  darkMode: 'class',
  theme: {
    extend: {
      colors: {
        mahalaxmi: {
          bg: '#0A2A2A',
          primary: '#00C8C8',
          gold: '#C8A040',
        },
      },
    },
  },
  plugins: [],
}

export default config
