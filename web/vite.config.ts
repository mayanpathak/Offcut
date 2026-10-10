import { cpSync, existsSync, mkdirSync, readdirSync, readFileSync, rmSync, statSync } from "node:fs";
import { createRequire } from "node:module";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

import react from "@vitejs/plugin-react";
import { defaultClientConditions, defineConfig, type Plugin } from "vite";

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

// The files of ONNX Runtime that the speech recognizer loads while it runs:
// the loader and the module of its WebGPU build. They are served from the
// app's own origin, under the version they belong to, because `vercel.json`
// lets `/ort/*` be cached for a year (D-37).
const ORT_FILES = ["ort-wasm-simd-threaded.asyncify.mjs", "ort-wasm-simd-threaded.asyncify.wasm"];
const ORT_PUBLIC = fileURLToPath(new URL("./public/ort", import.meta.url));

/** The `onnxruntime-web` that the recognizer's runtime depends on: where it is, and its version. */
function ortPackage(): { dir: string; version: string } {
  // pnpm does not link a dependency of a dependency into web/node_modules,
  // so it is resolved from the package that depends on it.
  const fromWeb = createRequire(import.meta.url);
  const fromRuntime = createRequire(fromWeb.resolve("@huggingface/transformers"));
  let dir = dirname(fromRuntime.resolve("onnxruntime-web"));
  for (;;) {
    const manifest = join(dir, "package.json");
    if (existsSync(manifest)) {
      const { name, version } = JSON.parse(readFileSync(manifest, "utf8")) as { name?: string; version?: string };
      if (name === "onnxruntime-web" && version !== undefined) {
        return { dir, version };
      }
    }
    const parent = dirname(dir);
    if (parent === dir) {
      throw new Error("vite.config.ts: the package onnxruntime-web was not found");
    }
    dir = parent;
  }
}

/** Copies those files to `public/ort/<version>/` and removes the directory of any other version. */
function copyOrt(): Plugin {
  return {
    name: "offcut:copy-ort",
    buildStart() {
      const { dir, version } = ortPackage();
      for (const entry of existsSync(ORT_PUBLIC) ? readdirSync(ORT_PUBLIC) : []) {
        if (entry !== version) {
          rmSync(join(ORT_PUBLIC, entry), { recursive: true, force: true });
        }
      }
      const target = join(ORT_PUBLIC, version);
      mkdirSync(target, { recursive: true });
      for (const file of ORT_FILES) {
        const from = join(dir, "dist", file);
        const to = join(target, file);
        // A dev server that starts again does not copy 27 MB again.
        if (!existsSync(to) || statSync(to).size !== statSync(from).size) {
          cpSync(from, to);
        }
      }
    },
  };
}

export default defineConfig({
  plugins: [react(), copyOrt()],
  resolve: {
    // ONNX Runtime has a build that holds no reference to its own `.wasm`
    // file. Without this condition the bundler follows that reference and
    // ships a second copy of the module, 27 MB, under `assets/`.
    conditions: ["onnxruntime-web-use-extern-wasm", ...defaultClientConditions],
  },
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
    // No proxy, which the preview server would otherwise take over from
    // `server`. The E2E tests run against it with a fake API in the browser;
    // a request that slips past the fake, such as the analytics batch a
    // closing page sends, must end here and never reach a real API.
    proxy: {},
  },
  build: {
    target: "esnext",
    // Nothing is inlined as a `data:` URL: the CSP would block a font or a WASM file.
    assetsInlineLimit: 0,
  },
  worker: { format: "es" },
});
