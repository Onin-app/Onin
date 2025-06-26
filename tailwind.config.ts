/** @type {import('tailwindcss').Config} */

export default {
  content: ['./src/**/*.{html,js,svelte,ts}'],
  // 告诉 Tailwind 使用 'class' 策略来切换暗黑模式
  darkMode: 'class',
  theme: {
    extend: {},
  },
  plugins: [],
};
