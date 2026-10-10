// Structural gate (technicalspec.md §5, §7, §29). Fails when:
//
//   1. a tracked source file is not in the file tree of TS §5;
//   2. a source file is longer than 400 lines, tests not counted;
//   3. a crate depends on an `offcut-*` crate that the graph of TS §7 does
//      not give it, or a pure crate resolves a browser or randomness crate.
//
// A file the tree lists and the repository does not have yet is not an
// error: the tree describes V10.
//
// Usage: node scripts/check-file-tree.mjs

import { execFileSync } from "node:child_process";
import { readFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const ROOT = resolve(dirname(fileURLToPath(import.meta.url)), "..");
process.chdir(ROOT);

const SPEC = "docs/technicalspec.md";
const TREE_HEADING = "## 5. Complete File Tree";

/** A tracked file under one of these, or directly in the repository root, must be in the tree. */
const CHECKED_ROOTS = ["crates/", "server/", "web/", "scripts/", "verify/", "corpus/", "bench/", ".github/"];

/** Generated and data paths, which the tree does not list one by one. */
const IGNORED = [
  /^Cargo\.lock$/,
  /^pnpm-lock\.yaml$/,
  /^server\/\.sqlx\//,
  /\.stderr$/, // trybuild's expected compiler output
  /\.snap$/,
  /(^|\/)__snapshots__\//,
];

const MAX_LINES = 400;
const SOURCE_EXTENSIONS = /\.(rs|ts|tsx|js|mjs|cjs|sh|css|py|sql)$/;
/** Whole files of tests, which the line limit does not apply to. */
const TEST_FILES = [/\.test\.tsx?$/, /^web\/tests-e2e\//, /^server\/tests\//, /^crates\/[^/]+\/tests\//];
/** Written by a generator, not by a person. */
const GENERATED = [/^web\/src\/gen\//];

/**
 * The `offcut-*` crates each crate may depend on (TS §7; arrows point to
 * what may be imported). Every crate may use `offcut-types`.
 */
const ALLOWED_CRATE_DEPS = {
  "offcut-types": [],
  "offcut-api-types": ["offcut-types"],
  "offcut-mp4": ["offcut-types"],
  "offcut-text": ["offcut-types"],
  "offcut-dsp": ["offcut-types"],
  // offcut-text for format_quantity only (v2implementation D-28).
  "offcut-scene": ["offcut-text", "offcut-types"],
  "offcut-entitlement": ["offcut-types"],
  "offcut-detect": ["offcut-text", "offcut-types"],
  "offcut-render": ["offcut-scene", "offcut-types"],
  "offcut-wasm-core": ["offcut-mp4", "offcut-text", "offcut-dsp", "offcut-types"],
  "offcut-wasm-render": ["offcut-render", "offcut-detect", "offcut-mp4", "offcut-entitlement", "offcut-types"],
  "offcut-api": ["offcut-api-types", "offcut-entitlement", "offcut-types"],
};

/**
 * Crates that must not read a clock, randomness or the browser (TS §7), and
 * the crates that would let them. `cargo deny` cannot see this for `uuid`,
 * which the server uses with its random features and these crates without.
 */
const PURE_CRATES = [
  "offcut-types",
  "offcut-api-types",
  "offcut-mp4",
  "offcut-dsp",
  "offcut-text",
  "offcut-detect",
  "offcut-scene",
  "offcut-entitlement",
];
const NOT_IN_PURE_CRATES = ["rand", "getrandom", "wasm-bindgen", "js-sys", "web-sys", "wgpu"];

const problems = [];

// --- 1. The tree ---------------------------------------------------------------

/** The paths of the fenced tree of TS §5: its files, and its directories that list no content. */
function readTree() {
  const lines = readFileSync(SPEC, "utf8").split(/\r?\n/);
  const heading = lines.indexOf(TREE_HEADING);
  const open = lines.findIndex((line, i) => i > heading && line.startsWith("```"));
  const close = lines.findIndex((line, i) => i > open && line.startsWith("```"));
  if (heading < 0 || open < 0 || close < 0) {
    throw new Error(`${SPEC}: cannot find the fenced tree under "${TREE_HEADING}"`);
  }

  const files = new Set();
  const directories = [];
  const parents = []; // parents[depth] is the directory that entries at that depth are in
  for (const line of lines.slice(open + 1, close)) {
    const marker = line.search(/[├└]── /);
    if (marker < 0) {
      continue; // the root line
    }
    const depth = marker / 4;
    // The name ends at the first run of two spaces; a comment follows it.
    const name = line.slice(marker + 4).split(/\s{2,}/)[0].trim();
    const path = (depth === 0 ? "" : parents[depth - 1]) + name;
    if (name.endsWith("/")) {
      parents[depth] = path;
      directories.push(path);
    } else {
      files.add(path);
    }
  }

  const listed = [...files, ...directories];
  const leafDirectories = directories.filter((dir) => !listed.some((path) => path !== dir && path.startsWith(dir)));
  return { files, leafDirectories };
}

// Tracked files, and new files that are not ignored: a file is checked
// before its first commit, not after.
const tracked = execFileSync("git", ["ls-files", "--cached", "--others", "--exclude-standard"], { encoding: "utf8" })
  .split("\n")
  .filter(Boolean);
const tree = readTree();

const isChecked = (path) => !path.includes("/") || CHECKED_ROOTS.some((root) => path.startsWith(root));
const isIgnored = (path) => IGNORED.some((pattern) => pattern.test(path));
const isListed = (path) => tree.files.has(path) || tree.leafDirectories.some((dir) => path.startsWith(dir));

for (const path of tracked) {
  if (isChecked(path) && !isIgnored(path) && !isListed(path)) {
    problems.push(`${path}: not in the file tree of TS §5. Add it there in the same change, or remove the file.`);
  }
}

// --- 2. File length ------------------------------------------------------------

/** The lines that count: all of them, or for Rust the lines above the inline test module. */
function countedLines(path) {
  const lines = readFileSync(path, "utf8").split(/\r?\n/);
  if (lines.at(-1) === "") {
    lines.pop();
  }
  if (path.endsWith(".rs")) {
    const tests = lines.indexOf("#[cfg(test)]");
    return tests < 0 ? lines.length : tests;
  }
  return lines.length;
}

let longest = { path: "", lines: 0 };
for (const path of tracked) {
  const exempt = [...TEST_FILES, ...GENERATED].some((pattern) => pattern.test(path));
  if (!isChecked(path) || !SOURCE_EXTENSIONS.test(path) || exempt) {
    continue;
  }
  const lines = countedLines(path);
  if (lines > longest.lines) {
    longest = { path, lines };
  }
  if (lines > MAX_LINES) {
    problems.push(`${path}: ${lines} lines, over the limit of ${MAX_LINES} (TS §29). Split the file.`);
  }
}

// --- 3. Crate dependencies -----------------------------------------------------

/** The package name of a manifest and the `offcut-*` crates it names in any dependency table. */
function readManifest(path) {
  let section = "";
  let name;
  const deps = new Set();
  for (const raw of readFileSync(path, "utf8").split(/\r?\n/)) {
    const line = raw.trim();
    const header = /^\[(.+)\]$/.exec(line);
    if (header) {
      section = header[1];
      // [dependencies.offcut-x] and [target.'cfg(...)'.dependencies.offcut-x]
      const table = /dependencies\.(offcut-[a-z0-9-]+)$/.exec(section);
      if (table) {
        deps.add(table[1]);
      }
      continue;
    }
    const key = /^"?([A-Za-z0-9_-]+)"?\s*(=|\.)/.exec(line)?.[1];
    if (key === undefined) {
      continue;
    }
    if (section === "package" && key === "name") {
      name = /=\s*"([^"]+)"/.exec(line)?.[1];
    } else if (/dependencies$/.test(section) && key.startsWith("offcut-")) {
      deps.add(key);
    }
  }
  return { name, deps: [...deps] };
}

const manifests = tracked.filter((path) => /^(crates\/[^/]+|server)\/Cargo\.toml$/.test(path));
const crateNames = [];
for (const path of manifests) {
  const { name, deps } = readManifest(path);
  const allowed = name === undefined ? undefined : ALLOWED_CRATE_DEPS[name];
  if (name === undefined || allowed === undefined) {
    problems.push(`${path}: crate "${name ?? "?"}" is not in the dependency graph of TS §7. Add it to the spec and to ALLOWED_CRATE_DEPS.`);
    continue;
  }
  crateNames.push(name);
  for (const dep of deps.filter((dep) => !allowed.includes(dep))) {
    problems.push(`${path}: ${name} may not depend on ${dep} (TS §7). It may depend on: ${allowed.join(", ") || "no offcut crate"}.`);
  }
}

for (const crate of PURE_CRATES.filter((name) => crateNames.includes(name))) {
  let resolved;
  try {
    resolved = execFileSync("cargo", ["tree", "-p", crate, "-e", "normal", "--prefix", "none", "--locked"], {
      encoding: "utf8",
      stdio: ["ignore", "pipe", "pipe"],
    });
  } catch (error) {
    const reason = String(error.stderr ?? error.message).trim().split("\n")[0];
    problems.push(`${crate}: \`cargo tree\` failed, so its dependencies could not be checked: ${reason}`);
    continue;
  }
  const names = new Set(resolved.split("\n").map((line) => line.split(" ")[0]));
  for (const banned of NOT_IN_PURE_CRATES.filter((name) => names.has(name))) {
    problems.push(`${crate}: resolves ${banned}, which a pure crate must not use (TS §7). Check the features of its dependencies.`);
  }
}

// --- Result --------------------------------------------------------------------

if (problems.length > 0) {
  console.error(`check-file-tree: ${problems.length} problem${problems.length === 1 ? "" : "s"}`);
  for (const problem of problems) {
    console.error(`  ${problem}`);
  }
  process.exit(1);
}
console.log(
  `check-file-tree: ok. ${tracked.filter((path) => isChecked(path) && !isIgnored(path)).length} files in the tree of TS §5; ` +
    `longest source file ${longest.path} (${longest.lines} lines); ${crateNames.length} crates within the graph of TS §7.`,
);
