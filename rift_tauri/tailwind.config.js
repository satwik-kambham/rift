/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{vue,js,ts,jsx,tsx}",
  ],
  theme: {
    extend: {
      colors: {
        "black": "var(--color-black)",
        "bg-dark": "var(--color-bg-dark)",
        "bg": "var(--color-bg)",
        "bg-hover": "var(--color-bg-hover)",
        "bg-light": "var(--color-bg-light)",
        "bg-icon": "var(--color-bg-icon)",
        "highlight": "var(--color-highlight)",
        "text": "var(--color-text)",
        "text-dark": "var(--color-text-dark)",
        "text-light": "var(--color-text-light)",
        "primary": "var(--color-primary)",
        "highlight-None": "var(--color-highlight-None)",
        "highlight-White": "var(--color-highlight-White)",
        "highlight-Red": "var(--color-highlight-Red)",
        "highlight-Orange": "var(--color-highlight-Orange)",
        "highlight-Blue": "var(--color-highlight-Blue)",
        "highlight-Green": "var(--color-highlight-Green)",
        "highlight-Purple": "var(--color-highlight-Purple)",
        "highlight-Yellow": "var(--color-highlight-Yellow)",
        "highlight-Gray": "var(--color-highlight-Gray)",
        "highlight-Turquoise": "var(--color-highlight-Turquoise)"
      }
    },
  },
  plugins: [],
}
