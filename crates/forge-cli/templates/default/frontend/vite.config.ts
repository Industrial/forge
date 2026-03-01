import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';
import macros from 'unplugin-parcel-macros';

const backend = process.env.VITE_BACKEND_URL ?? 'http://localhost:4000';

export default defineConfig({
  plugins: [
    macros.vite(), // Must be first! (React Spectrum style macros)
    react(),
  ],
  base: '/',
  root: '.',
  publicDir: 'public',
  build: {
    target: ['es2022'],
    outDir: 'dist',
    emptyOutDir: true,
    manifest: true,
    cssMinify: 'lightningcss',
    rollupOptions: {
      input: 'src/main.tsx',
      output: {
        manualChunks(id) {
          if (/macro-(.*)\.css$/.test(id) || /@react-spectrum\/s2\/.*\.css$/.test(id)) {
            return 's2-styles';
          }
        },
      },
    },
  },
  server: {
    port: 3000,
    strictPort: true,
    origin: 'http://localhost:3000',
    proxy: {
      '/api': { target: backend, changeOrigin: true },
      '/ws': { target: backend, changeOrigin: true, ws: true },
    },
  },
});
