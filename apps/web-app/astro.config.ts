import { existsSync } from "node:fs";
import { fileURLToPath } from "node:url";
import type { AstroIntegration } from "astro";
import { defineConfig } from "astro/config";
import react from "@astrojs/react";

// Dev stand-in for the image's Caddyfile, which serves /config.js from the container's env.
function runtimeConfig(): AstroIntegration {
  return {
    name: "runtime-config",
    hooks: {
      "astro:server:setup": ({ server }) => {
        const envFile = fileURLToPath(new URL(".env", import.meta.url));
        if (existsSync(envFile)) process.loadEnvFile(envFile);
        const authApiUrl = new URL(process.env.AUTH_API_URL || "http://127.0.0.1:13001").origin;
        server.middlewares.use("/config.js", (_req, res) => {
          res.setHeader("Content-Type", "text/javascript; charset=utf-8");
          res.setHeader("Cache-Control", "no-store");
          res.end(`globalThis.AUTH_API_URL = ${JSON.stringify(authApiUrl)};\n`);
        });
      },
    },
  };
}

// Static MPA: every page prerenders to HTML; React islands hydrate where interactive.
// Origins are runtime config (/config.js), not build env: one image serves every environment.
export default defineConfig({
  output: "static",
  integrations: [react(), runtimeConfig()],
});
