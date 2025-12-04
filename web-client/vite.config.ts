import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import tailwindcss from "@tailwindcss/vite";
import wasm from "vite-plugin-wasm";

// https://vite.dev/config/
//https://www.npmjs.com/package/vite-plugin-wasm
export default defineConfig({
  plugins: [react(), tailwindcss(), wasm()],
  build: {
    target: "esnext",
  },
});
