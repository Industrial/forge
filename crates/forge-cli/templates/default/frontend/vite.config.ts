/*
 * Copyright 2024 Adobe. All rights reserved.
 * This file is licensed to you under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License. You may obtain a copy
 * of the License at http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software distributed under
 * the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR REPRESENTATIONS
 * OF ANY KIND, either express or implied. See the License for the specific language
 * governing permissions and limitations under the License.
 */

import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'
import optimizeLocales from '@react-aria/optimize-locales-plugin'
import macros from 'unplugin-parcel-macros';

const VITE_BACKEND_URL = process.env.VITE_BACKEND_URL;
if (!VITE_BACKEND_URL) {
  throw new Error('VITE_BACKEND_URL is not set');
}

export default defineConfig({
  server: {
    proxy: {
      '/api': { target: VITE_BACKEND_URL, changeOrigin: true },
      '/ws': { target: VITE_BACKEND_URL, ws: true, changeOrigin: true },
    },
  },
  plugins: [
    macros.vite(),
    react(),
    {
      ...optimizeLocales.vite({
        locales: ['en-US', 'fr-FR']
      }),
      enforce: 'pre'
    }
  ],
  build: {
    target: ['es2022'],
    // Lightning CSS produces a much smaller CSS bundle than the default minifier.
    cssMinify: 'lightningcss',
    rollupOptions: {
      output: {
        // Bundle all S2 and style-macro generated CSS into a single bundle instead of code splitting.
        // Because atomic CSS has so much overlap between components, loading all CSS up front results in
        // smaller bundles instead of producing duplication between pages.
        manualChunks(id) {
          if (/macro-(.*)\.css$/.test(id) || /@react-spectrum\/s2\/.*\.css$/.test(id)) {
            return 's2-styles';
          }
        }
      }
    }
  }
})
