import { createHash } from "node:crypto";
import { cpSync, existsSync, mkdirSync, readdirSync, readFileSync, rmSync, statSync } from "node:fs";
import { createRequire } from "node:module";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

import react from "@vitejs/plugin-react";
import { defaultClientConditions, defineConfig, type Plugin } from "vite";

import { featureFlags } from "./src/config/features";
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

// What draws and encodes an export: a new version of any of them gives every
// file a new key in the render cache (v5implementation D-223). `pnpm build`
// builds the WASM first, so a production build always has it.
const RENDERER_FILES = ["src/wasm/pkg/render/offcut_render_bg.wasm", "src/workers/render/export-loop.ts", "src/workers/render/encoders.ts"];

/** 16 hex digits of the SHA-256 of those files; `"none"` before the WASM has been built. */
function rendererVersion(): string {
  const files = RENDERER_FILES.map((file) => fileURLToPath(new URL(`./${file}`, import.meta.url)));
  if (!files.every((file) => existsSync(file))) {
    return "none";
  }
  const hash = createHash("sha256");
  for (const file of files) {
    hash.update(readFileSync(file));
  }
  return hash.digest("hex").slice(0, 16);
}

export default defineConfig(({ mode }) => ({
  plugins: [react(), copyOrt()],
  define: {
    // Whether the workers take the development hooks' faults (workers/rpc-worker.ts):
    // the workers may not read `import.meta.env`, and a production build folds this to `false`.
    __OFFCUT_DEV_FAULTS__: JSON.stringify(mode !== "production"),
    __RENDERER_VERSION__: JSON.stringify(rendererVersion()),
    // The feature flags of config/features.ts (TS §41.3.1): each takes its
    // default unless OFFCUT_FEATURES_ON or OFFCUT_FEATURES_OFF (comma lists)
    // names it; the V6.x flags default off (v6.1implementation D-393).
    __OFFCUT_FEATURES__: JSON.stringify(featureFlags(process.env.OFFCUT_FEATURES_ON, process.env.OFFCUT_FEATURES_OFF)),
  },
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
}));
