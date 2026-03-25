import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'

// https://vite.dev/config/
export default defineConfig({
  base: "./",
  plugins: [react()],
  server: {
    port: 1420,
    host: '0.0.0.0',
    proxy: {
      '/v1': {
        target: 'http://localhost:5173',
        changeOrigin: true,
        ws: true,
      },
      '/oauth-callback': {
        target: 'http://localhost:5173',
        changeOrigin: true,
      }
    }
  }
})
