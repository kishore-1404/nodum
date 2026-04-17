/** @type {import('tailwindcss').Config} */
export default {
  content: ['./index.html', './src/**/*.{js,ts,jsx,tsx}'],
  theme: {
    extend: {
      colors: {
        ink: {
          50: '#f7f6f3',
          100: '#eae8e1',
          200: '#d5d1c6',
          300: '#b8b2a2',
          400: '#9a917d',
          500: '#7d7464',
          600: '#635c50',
          700: '#504a41',
          800: '#3d3935',
          900: '#2a2723',
          950: '#1a1816',
        },
        parchment: {
          50: '#fdfcf9',
          100: '#faf8f2',
          200: '#f5f1e6',
          300: '#ede7d4',
          400: '#e0d7be',
        },
        node: {
          concept: '#6366f1',
          fact: '#3b82f6',
          principle: '#8b5cf6',
          example: '#10b981',
          question: '#f59e0b',
          insight: '#ec4899',
        },
        accent: {
          primary: '#c47a3a',
          secondary: '#8b6f4e',
          success: '#4a7c59',
          warning: '#b8860b',
          danger: '#a0522d',
        },
      },
      fontFamily: {
        display: ['"Playfair Display"', 'Georgia', 'serif'],
        body: ['"Source Serif 4"', 'Georgia', 'serif'],
        mono: ['"JetBrains Mono"', 'monospace'],
        sans: ['"DM Sans"', 'system-ui', 'sans-serif'],
      },
      animation: {
        'fade-in': 'fadeIn 0.4s ease-out',
        'slide-up': 'slideUp 0.3s ease-out',
        'pulse-soft': 'pulseSoft 2s ease-in-out infinite',
        'node-appear': 'nodeAppear 0.5s cubic-bezier(0.34, 1.56, 0.64, 1)',
      },
      keyframes: {
        fadeIn: {
          '0%': { opacity: '0' },
          '100%': { opacity: '1' },
        },
        slideUp: {
          '0%': { opacity: '0', transform: 'translateY(12px)' },
          '100%': { opacity: '1', transform: 'translateY(0)' },
        },
        pulseSoft: {
          '0%, 100%': { opacity: '0.6' },
          '50%': { opacity: '1' },
        },
        nodeAppear: {
          '0%': { opacity: '0', transform: 'scale(0)' },
          '100%': { opacity: '1', transform: 'scale(1)' },
        },
      },
    },
  },
  plugins: [],
}
