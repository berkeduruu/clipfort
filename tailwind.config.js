/** @type {import('tailwindcss').Config} */
export default {
  content: ['./src/**/*.{html,js,svelte,ts}'],
  theme: {
    extend: {
      colors: {
        paper: {
          50: '#faf8f5',
          100: '#f4efe8',
          200: '#e8e0d5',
          300: '#d7cabb',
          800: '#38332d',
          900: '#231f1a',
        },
        sage: {
          50: '#f4f7f4',
          100: '#e3ece3',
          500: '#588157',
          600: '#466845',
        }
      }
    },
  },
  plugins: [],
}

