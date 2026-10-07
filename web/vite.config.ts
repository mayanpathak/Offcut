import react from "@vitejs/plugin-react";
import { defineConfig } from "vite";

import vercel from "./vercel.json" with { type: "json" };

// `vite preview` serves the headers of the `/(.*)` rule of vercel.json, so the
// E2E tests run against the production CSP and there is one source for it (D-15).
const ALL_PATHS = "/(.*)";
const productionHeaders = Object.fromEntries(
  vercel.headers
    .filter((rule) => rule.source === ALL_PATHS)
    .flatMap((rule) => rule.headers)
    .map((header) => [header.key, header.value]),
);

export default defineConfig({
  plugins: [react()],
  server: {
    // Cross-origin isolation only. The production CSP would block Vite's hot reload.
    headers: {
      "Cross-Origin-Opener-Policy": "same-origin",
      "Cross-Origin-Embedder-Policy": "require-corp",
    },
    proxy: { "/api/v1": "http://localhost:8080" },
  },
  preview: {
    headers: productionHeaders,
  },
  build: {
    target: "esnext",
    // Nothing is inlined as a `data:` URL: the CSP would block a font or a WASM file.
    assetsInlineLimit: 0,
  },
  worker: { format: "es" },
});
