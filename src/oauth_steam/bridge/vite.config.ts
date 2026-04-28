import { defineConfig } from 'vite'
import react, { reactCompilerPreset } from '@vitejs/plugin-react'
import babel from '@rolldown/plugin-babel'
import { dotLocal } from './src/utils/dotLocalProxy'

// https://vite.dev/config/
export default defineConfig({
  plugins: [
    react(),
    babel({ presets: [reactCompilerPreset()] })
  ],
  base: "./",
  publicDir: false,
  server: {
    // headers: {
    //   'Cross-Origin-Embedder-Policy': 'require-corp',
    //   'Cross-Origin-Opener-Policy': 'same-origin'
    // },
    proxy: {
      ...dotLocal(),
    }
  }
})
