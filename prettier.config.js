/** @type {import('prettier').Config} */
const config = {
  singleQuote: true,
  endOfLine: 'lf',
  plugins: ['prettier-plugin-svelte'],
  overrides: [{ files: '*.svelte', options: { parser: 'svelte' } }],
};

export default config;
