// The lint rules of technicalspec.md §7 for the web app. Every rule is on from
// the first day, including the ones that have nothing to check yet.

import { readFileSync } from "node:fs";

import { defineConfig, globalIgnores } from "eslint/config";
import boundaries from "eslint-plugin-boundaries";
import react from "eslint-plugin-react";
import tseslint from "typescript-eslint";

const SRC = "src/**/*.{ts,tsx}";

// --- Layers (TS §2, §7) ------------------------------------------------------

// Each folder under src/ is one layer. The three entry files in src/ itself
// (main.tsx, App.tsx, routes.tsx) belong to no layer and are not checked.
const LAYERS = [
  "ui",
  "usecases",
  "state",
  "persistence",
  "models",
  "net",
  "analytics",
  "workers",
  "wasm",
  "platform",
  "config",
  "copy",
  "gen",
  "entitlement",
];

/** A whole layer, or the named files of it (paths relative to the layer's folder). */
const to = (layer, ...files) => ({
  element: files.length > 0 ? { type: layer, fileInternalPath: files } : { type: layer },
});
const from = (layer) => ({ element: { type: layer } });
/** The named files of a layer, as the importing side. */
const fromFile = (layer, ...files) => ({ element: { type: layer, fileInternalPath: files } });

// What each layer may import, beyond itself. Everything else is refused.
// Every layer may import `gen`, which holds types and constants only.
const MAY_IMPORT = {
  ui: [to("usecases"), to("state"), to("copy"), to("config")],
  usecases: [
    to("state"),
    to("persistence"),
    to("models"),
    to("analytics"),
    to("entitlement"),
    to("platform"),
    // For `env.dev`: whether this is a development build.
    to("config"),
    to("workers", "pool.ts"),
    to("net", "api-client.ts"),
  ],
  state: [],
  persistence: [],
  models: [to("net", "asset-fetch.ts"), to("persistence", "opfs.ts"), to("wasm", "load-core.ts")],
  net: [to("config")],
  analytics: [to("net", "api-client.ts")],
  workers: [to("wasm")],
  wasm: [],
  // The capability check reads the encoder constants (v1implementation §11.8).
  platform: [to("workers", "render/encoders.ts")],
  config: [],
  copy: [to("config")],
  gen: [],
  entitlement: [to("config")],
};

const INTERNAL = { relationship: { to: "internal" } };

const layerPolicies = [
  ...LAYERS.map((layer) => ({
    from: from(layer),
    allow: [...MAY_IMPORT[layer], ...(layer === "gen" ? [] : [to("gen")])].map((target) => ({
      to: target,
    })),
  })),
  // A layer's files may import each other, except in `usecases`: a use-case
  // imports another only if it is cancel-job or restore-clip (TS §7).
  ...LAYERS.filter((layer) => layer !== "usecases").map((layer) => ({
    from: from(layer),
    allow: { dependency: INTERNAL },
  })),
  {
    from: from("usecases"),
    allow: { to: to("usecases", "cancel-job.ts", "restore-clip.ts") },
  },
  // A test imports the file it tests.
  {
    from: { file: { categories: "test" } },
    allow: { dependency: INTERNAL },
  },
  // A page shows the demo video, which is on the asset host. `ui` may take
  // the two names that build its URL from net/asset-fetch.ts, and nothing
  // from `net` that makes a request (v1implementation §11.12).
  {
    from: from("ui"),
    allow: {
      to: to("net", "asset-fetch.ts"),
      dependency: { specifiers: ["assetUrl", "DEMO_VIDEO_PATH"] },
    },
  },
  // `net` may name the types of the worker protocol, never its code (D-8).
  {
    from: from("net"),
    allow: { to: to("workers", "protocol.ts"), dependency: { kind: "type" } },
  },

  // The six edges V2 adds (v2implementation D-27, a to f). The flows of the
  // spec need them and the matrix above did not have them.
  // (a) The sample clip is fetched through asset-fetch.ts and handed to importClip (TS §15.4).
  {
    from: fromFile("usecases", "import-clip.ts"),
    allow: { to: to("net", "asset-fetch.ts") },
  },
  // (b) opfs.ts is the only builder of OPFS paths, and the workers write to those paths (TS §23.1).
  {
    from: from("workers"),
    allow: { to: to("persistence", "opfs.ts") },
  },
  // (c) The render worker verifies the entitlement token (TS §21.3).
  {
    from: fromFile("workers", "render.worker.ts"),
    allow: { to: to("config", "entitlement-public-key.ts") },
  },
  // (d) The model manager writes the model store and reads the bundled manifest (TS §12.1).
  {
    from: from("models"),
    allow: [{ to: to("state", "model-store.ts") }, { to: to("config", "model-manifest.json") }],
  },
  // (e) ensureReady records in `meta` that it asked for persistent storage.
  {
    from: fromFile("models", "model-manager.ts"),
    allow: { to: to("persistence", "db.ts") },
  },
  // (f) The stores, the model manager and the use-cases hold and pass on a
  // FeedLine or an AppFailure: the types of the worker protocol, never its code.
  {
    from: { element: { types: { anyOf: ["state", "models", "usecases"] } } },
    allow: { to: to("workers", "protocol.ts"), dependency: { kind: "type" } },
  },

  // The two use-case pairs V2 adds (D-58): importClip starts the pipeline
  // (TS C-2), and startExport pauses and locks the preview (TS C-9).
  {
    from: fromFile("usecases", "import-clip.ts"),
    allow: { to: to("usecases", "run-pipeline.ts") },
  },
  {
    from: fromFile("usecases", "start-export.ts"),
    allow: { to: to("usecases", "control-preview.ts") },
  },
];

// --- Network access (TS §7, §24.1) -------------------------------------------

const NETWORK_GLOBALS = ["fetch", "XMLHttpRequest", "WebSocket", "EventSource", "RTCPeerConnection"];
const GLOBAL_OBJECTS = ["window", "self", "globalThis"];
// The only files that may call `fetch`. No file may use the other four.
const FETCH_FILES = [
  "src/net/http.ts",
  "src/net/asset-fetch.ts",
  "src/wasm/load-core.ts",
  "src/wasm/load-render.ts",
];

const networkMessage = (name) =>
  `${name} is not available here. Network requests go through net/http.ts or net/asset-fetch.ts.`;

const restrictedGlobals = (names) => names.map((name) => ({ name, message: networkMessage(name) }));
const restrictedProperties = (names) => [
  { object: "navigator", property: "sendBeacon", message: networkMessage("navigator.sendBeacon") },
  ...GLOBAL_OBJECTS.flatMap((object) =>
    names.map((property) => ({ object, property, message: networkMessage(property) })),
  ),
];

const NETWORK_GLOBALS_BUT_FETCH = NETWORK_GLOBALS.filter((name) => name !== "fetch");

// --- Restricted syntax (TS §7, §11.3) ----------------------------------------

// The branded unit and id types, read from the generated file so the list
// cannot fall behind it.
const domain = readFileSync(new URL("./src/gen/domain.ts", import.meta.url), "utf8");
const BRANDS = [...domain.matchAll(/^export type (\w+) = (?:number|string) & \{ readonly __unit:/gm)].map(
  (match) => match[1],
);
if (BRANDS.length === 0) {
  throw new Error("eslint.config.js found no branded type in src/gen/domain.ts");
}
const brand = `TSTypeReference[typeName.name=/^(${BRANDS.join("|")})$/]`;

const NO_IMPORT_META_ENV = [
  {
    selector: "MemberExpression[object.type='MetaProperty'][property.name='env']",
    message: "import.meta.env is read in config/env.ts only. Import `env` from there.",
  },
];
const NO_BRAND_CAST = [`TSAsExpression > ${brand}`, `TSTypeAssertion > ${brand}`].map((selector) => ({
  selector,
  message:
    "A cast to a unit or id type is allowed in gen/ and workers/ only. Elsewhere the value must arrive already typed.",
}));
const NO_SWALLOWED_ERROR = [
  "CatchClause > BlockStatement[body.length=0]",
  "CallExpression[callee.property.name='catch'] > :function > BlockStatement[body.length=0]",
].map((selector) => ({
  selector,
  message: "A failure must not be swallowed. Only analytics/client.ts may drop one (TS §11.3).",
}));

const restrictedSyntax = (...groups) => ["error", ...groups.flat()];

// Seven files of the main thread turn a raw browser value into a unit or an
// id (v2implementation D-59). Each may hold one cast, to one type, in one
// named place: the last `return` of its minting helper or, in model-store.ts,
// the constant that is the zero of the initial state. Every other cast in
// the file is refused as before.
const MINTING = [
  { file: "src/persistence/opfs.ts", brand: "Bytes", helper: "toBytes" },
  { file: "src/models/download.ts", brand: "Bytes", helper: "toBytes" },
  { file: "src/models/model-manager.ts", brand: "Bytes", helper: "toBytes" },
  { file: "src/state/model-store.ts", brand: "Bytes", constant: "ZERO_BYTES" },
  { file: "src/usecases/import-clip.ts", brand: "ClipId", helper: "newClipId" },
  { file: "src/usecases/start-export.ts", brand: "ExportId", helper: "newExportId" },
  { file: "src/usecases/control-preview.ts", brand: "TimeMs", helper: "toTimeMs" },
];

/** The brand-cast rule for a minting file: NO_BRAND_CAST, less its one named cast. */
const noBrandCastExcept = ({ file, brand: minted, helper, constant }) => {
  if (!BRANDS.includes(minted)) {
    throw new Error(`eslint.config.js: ${minted} (${file}) is not a branded type of src/gen/domain.ts`);
  }
  const others = BRANDS.filter((name) => name !== minted);
  const place =
    helper === undefined
      ? `Program > VariableDeclaration[kind='const'] > VariableDeclarator[id.name='${constant}'] > TSAsExpression[expression.value=0]`
      : `FunctionDeclaration[id.name='${helper}'] > BlockStatement > ReturnStatement:last-child > TSAsExpression`;
  const where = helper === undefined ? `\`const ${constant} = 0 as ${minted}\`` : `the last return of ${helper}()`;
  const message = `In this file one cast is allowed: to ${minted}, in ${where} (D-59). Every other value must arrive already typed.`;
  return [
    `TSAsExpression > TSTypeReference[typeName.name=/^(${others.join("|")})$/]`,
    `TSAsExpression:not(${place}) > TSTypeReference[typeName.name='${minted}']`,
    `TSTypeAssertion > ${brand}`,
  ].map((selector) => ({ selector, message }));
};

// --- The configuration -------------------------------------------------------

export default defineConfig(
  // Generated and built files are not linted.
  globalIgnores(["dist/", "src/gen/", "src/wasm/pkg/", "playwright-report/", "test-results/"]),

  {
    files: [SRC],
    extends: [tseslint.configs.recommendedTypeChecked],
    languageOptions: {
      parserOptions: { projectService: true, tsconfigRootDir: import.meta.dirname },
    },
    plugins: { boundaries },
    settings: {
      "boundaries/root-path": import.meta.dirname,
      "boundaries/elements": LAYERS.map((layer) => ({ type: layer, pattern: `src/${layer}` })),
      "boundaries/files": [{ category: "test", pattern: "**/*.test.{ts,tsx}" }],
      "import/resolver": { node: { extensions: [".ts", ".tsx", ".js", ".json"] } },
    },
    rules: {
      "boundaries/dependencies": [
        "error",
        { default: "disallow", checkInternals: true, policies: layerPolicies },
      ],

      "no-restricted-globals": ["error", ...restrictedGlobals(NETWORK_GLOBALS)],
      "no-restricted-properties": ["error", ...restrictedProperties(NETWORK_GLOBALS)],
      "no-restricted-syntax": restrictedSyntax(NO_IMPORT_META_ENV, NO_BRAND_CAST, NO_SWALLOWED_ERROR),

      "@typescript-eslint/no-explicit-any": "error",
      "@typescript-eslint/no-non-null-assertion": "error",
      "@typescript-eslint/ban-ts-comment": [
        "error",
        { "ts-ignore": true, "ts-nocheck": true, "ts-expect-error": "allow-with-description" },
      ],
      // A switch over a union names every member, and ends in a default.
      "@typescript-eslint/switch-exhaustiveness-check": [
        "error",
        { considerDefaultExhaustiveForUnions: false, requireDefaultForNonUnion: true },
      ],
      "no-console": "error",
    },
  },

  // The E2E tests: the type-aware rules, without the app's layer and network
  // rules. A test drives a browser; it is not part of the app.
  {
    files: ["tests-e2e/**/*.ts"],
    extends: [tseslint.configs.recommendedTypeChecked],
    languageOptions: {
      parserOptions: { projectService: true, tsconfigRootDir: import.meta.dirname },
    },
    rules: {
      "@typescript-eslint/no-explicit-any": "error",
      "@typescript-eslint/no-non-null-assertion": "error",
      "no-console": "error",
    },
  },

  // The four fetch files: `fetch` is allowed, the other network APIs are not.
  {
    files: FETCH_FILES,
    rules: {
      "no-restricted-globals": ["error", ...restrictedGlobals(NETWORK_GLOBALS_BUT_FETCH)],
      "no-restricted-properties": ["error", ...restrictedProperties(NETWORK_GLOBALS_BUT_FETCH)],
    },
  },

  // The one exception to each restricted-syntax rule.
  {
    files: ["src/config/env.ts"],
    rules: { "no-restricted-syntax": restrictedSyntax(NO_BRAND_CAST, NO_SWALLOWED_ERROR) },
  },
  {
    files: ["src/workers/**/*.ts"],
    rules: { "no-restricted-syntax": restrictedSyntax(NO_IMPORT_META_ENV, NO_SWALLOWED_ERROR) },
  },
  {
    files: ["src/analytics/client.ts"],
    rules: { "no-restricted-syntax": restrictedSyntax(NO_IMPORT_META_ENV, NO_BRAND_CAST) },
  },
  // The seven minting files: one named cast each (D-59).
  ...MINTING.map((entry) => ({
    files: [entry.file],
    rules: {
      "no-restricted-syntax": restrictedSyntax(NO_IMPORT_META_ENV, noBrandCastExcept(entry), NO_SWALLOWED_ERROR),
    },
  })),

  // User-facing text lives in copy/messages.ts, never in a component.
  {
    files: ["src/ui/**/*.tsx", "src/*.tsx"],
    plugins: { react },
    languageOptions: { parserOptions: { ecmaFeatures: { jsx: true } } },
    settings: { react: { version: "19" } },
    rules: {
      "react/jsx-no-literals": [
        "error",
        {
          noStrings: true,
          ignoreProps: true,
          allowedStrings: [".", ",", ":", ";", "!", "?", "-", "–", "—", "/", "(", ")", "·", "…"],
        },
      ],
    },
  },
);
