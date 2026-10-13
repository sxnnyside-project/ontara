import { fileURLToPath } from 'node:url';
import { svelte } from '@sveltejs/vite-plugin-svelte';
import UnoCSS from 'unocss/vite';
import { defineConfig } from 'vite';

// Tauri loads the dev server on a fixed port, so a busy port must fail instead of drifting.
export default defineConfig({
  plugins: [UnoCSS(), svelte()],
  resolve: {
    alias: { '@': fileURLToPath(new URL('./src', import.meta.url)) },
  },
  clearScreen: false,
  server: {
    port: 5173,
    strictPort: true,
    watch: { ignored: ['**/src-tauri/**'] },
  },
});
