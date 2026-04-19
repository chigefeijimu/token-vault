/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx}",
  ],
  theme: {
    extend: {
      colors: {
        'vault-bg': '#0D0D1A',
        'vault-card': '#1A1A2E',
        'vault-border': '#2D2D44',
        'vault-text': '#FFFFFF',
        'vault-text-secondary': '#A0AEC0',
        'vault-success': '#10B981',
        'vault-warning': '#F59E0B',
        'vault-error': '#EF4444',
      },
      fontFamily: {
        'sans': ['Inter', 'system-ui', 'sans-serif'],
        'mono': ['JetBrains Mono', 'Consolas', 'monospace'],
      },
      backgroundImage: {
        'vault-gradient': 'linear-gradient(135deg, #667EEA 0%, #764BA2 100%)',
      },
    },
  },
  plugins: [],
}
