import { defineConfig } from "astro/config";
import react from "@astrojs/react";

// Config lives in the app root; client sources live in client/. Builds to dist/, which the
// eDd server serves from WEB_DIR (default `dist`). Same origin in production, so the dev
// server proxies /api to a locally running server instead of configuring CORS.
export default defineConfig({
  output: "static",
  srcDir: "./client/src",
  publicDir: "./client/public",
  integrations: [react()],
  vite: {
    server: { proxy: { "/api": "http://127.0.0.1:13002" } },
  },
});
