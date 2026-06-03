import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
import { resolve } from 'path'

export default defineConfig({
  plugins: [vue()],
  resolve: {
    alias: {
      '@': resolve(__dirname, 'src'),
    },
  },
  // Tauri 需要固定端口
  server: {
    port: 1420,
    strictPort: true,
    host: 'localhost',
  },
  // 前端构建目标：Tauri 使用系统 WebView
  build: {
    target: 'esnext',
    minify: 'esbuild',
    sourcemap: false,
  },
})
