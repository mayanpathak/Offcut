// Loading offcut_core.wasm. This is the only file that fetches, compiles or
// instantiates it.
//
// scripts/build-wasm.sh writes the module and its JavaScript glue to
// ./pkg/core/ (not in git). Vite serves the .wasm file from the app's own
// origin under a content-hashed name, as `application/wasm`.

import initCore, { core_version } from "./pkg/core/offcut_core";
import coreWasmUrl from "./pkg/core/offcut_core_bg.wasm?url";

/** What the module offers. V2 adds the media, text and hashing functions. */
export type CoreApi = { coreVersion(): string };

let compiled: Promise<WebAssembly.Module> | undefined;
let loaded: Promise<CoreApi> | undefined;

/** Fetches and compiles the module, once. A failed attempt is forgotten, so a later call tries again. */
function compile(): Promise<WebAssembly.Module> {
  compiled ??= WebAssembly.compileStreaming(fetch(coreWasmUrl)).catch((error: unknown) => {
    compiled = undefined;
    throw error;
  });
  return compiled;
}

/**
 * Fetches and compiles the module without running it, so the first worker
 * that needs it does not wait for the download. Compiling is the step the
 * CSP must allow (`'wasm-unsafe-eval'`). Safe to call any number of times.
 */
export async function preloadCore(): Promise<void> {
  await compile();
}

/**
 * Instantiates the compiled module, compiling it first if needed. For
 * workers: the main thread never runs WASM (INV-17).
 */
export function loadCore(): Promise<CoreApi> {
  loaded ??= compile()
    .then((module) => initCore({ module_or_path: module }))
    .then((): CoreApi => ({ coreVersion: core_version }))
    .catch((error: unknown) => {
      loaded = undefined;
      throw error;
    });
  return loaded;
}
