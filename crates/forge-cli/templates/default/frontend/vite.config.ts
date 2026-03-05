import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'
import path from 'path'

const VITE_BACKEND_URL = process.env.VITE_BACKEND_URL
if (!VITE_BACKEND_URL) {
  throw new Error('VITE_BACKEND_URL is not set')
}

export default defineConfig({
  resolve: {
    alias: { '@': path.resolve(process.cwd(), 'src') },
  },
  server: {
    proxy: {
      '/api': { target: VITE_BACKEND_URL, changeOrigin: true },
      '/ws': { target: VITE_BACKEND_URL, ws: true, changeOrigin: true },
    },
  },
  plugins: [react()],
  build: {
    target: ['es2022'],
    cssMinify: 'lightningcss',
  },
})
