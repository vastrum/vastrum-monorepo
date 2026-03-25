import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'
import wasm from "vite-plugin-wasm";
import dts from 'vite-plugin-dts';

export default defineConfig({
  plugins: [react(), wasm(), dts()],
  build: {
    target: "esnext",
    lib: {
      entry: './src/index.ts',
      formats: ['es'],
      fileName: 'vastrum-react-lib'
    }
  }
})
