import { defineConfig } from 'vite';
import { svelte } from '@sveltejs/vite-plugin-svelte';
import { alphaTab } from '@coderline/alphatab-vite';

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [svelte(), alphaTab()],
  clearScreen: false,
  server: {
    port: 5173,
    strictPort: true,
  },
  envPrefix: ['VITE_', 'TAURI_ENV_*'],
  build: {
    target: 'es2022',
  },
});
