import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import tailwindcss from "@tailwindcss/vite";
import { resolve } from "node:path";
export default defineConfig({
  root: ".native-parity",
  plugins: [react(), tailwindcss()],
  resolve: { alias: { "@": resolve(__dirname, "../packages/ui/src") } },
  server: { port: 5188, strictPort: true },
});
