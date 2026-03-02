import path from 'path';
import { fileURLToPath } from 'url';
import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';
import macros from 'unplugin-parcel-macros';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const backend = process.env.VITE_BACKEND_URL ?? 'http://localhost:4000';

export default defineConfig({
  resolve: {
    alias: {
      // Shim so s2@0.8.0 (expects UNSTABLE_TableLoadingIndicator) works with react-aria-components 1.10.1 (exports UNSTABLE_TableLoadingSentinel)
      'react-aria-components': path.resolve(__dirname, 'src/shim-react-aria-components.ts'),
      'react-aria-components-original': path.resolve(__dirname, 'node_modules/react-aria-components/dist/import.mjs'),
      'react-aria-components-table': path.resolve(__dirname, 'node_modules/react-aria-components/dist/Table.mjs'),
    },
  },
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
