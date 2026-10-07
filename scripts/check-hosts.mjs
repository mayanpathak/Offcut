// Network gate (TS §24.1). The browser may contact two hosts: the app's own
// and the asset host. This script reads the built app, not the source, so it
// also sees what a dependency brings in.
//
// Fails when:
//   - web/dist contains an http:// or https:// literal whose host is not the
//     asset host and that is not on the short list below;
//   - the hosts in the CSP of web/vercel.json are not exactly the asset host.
//
// It also prints the gzip size of the app shell.
//
// Usage: node scripts/check-hosts.mjs      (after `vite build`)
// VITE_ASSET_BASE_URL comes from the environment, or from web/.env.local.

import { existsSync, readdirSync, readFileSync } from "node:fs";
import { dirname, join, relative, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { gzipSync } from "node:zlib";

const ROOT = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const DIST = resolve(ROOT, "web/dist");
const VERCEL = resolve(ROOT, "web/vercel.json");
const ENV_LOCAL = resolve(ROOT, "web/.env.local");

/**
 * URL literals in the bundle that are never requested. Each entry is the
 * start of the literal and the reason it is harmless. Keep this list short:
 * a new entry needs the same proof, that nothing fetches it.
 */
const NON_NETWORK_LITERALS = [
  { prefix: "http://www.w3.org/", reason: "XML namespace names (SVG, MathML, XLink, XML) that React passes to createElementNS; a name, not an address" },
  { prefix: "https://react.dev/errors/", reason: "the address React puts in the text of a production error, for a person to open" },
  { prefix: "https://reactrouter.com/", reason: "a documentation address in the text of a React Router error" },
  // Exactly this and nothing after it: a port or a path would be a real address.
  { prefix: "http://localhost", exact: true, reason: "the placeholder base React Router gives to `new URL()` to parse a relative path; never requested" },
];

const URL_LITERAL = /https?:\/\/[A-Za-z0-9._~:/?#[\]@!$&'()*+,;=%-]+/g;
/** What the browser downloads to show the first page. */
const SHELL = /\.(html|js|css)$/;

function fail(message) {
  console.error(`check-hosts: ${message}`);
  process.exit(1);
}

function assetBaseUrl() {
  const fromEnv = process.env.VITE_ASSET_BASE_URL;
  if (fromEnv !== undefined && fromEnv !== "") {
    return fromEnv;
  }
  if (existsSync(ENV_LOCAL)) {
    const line = readFileSync(ENV_LOCAL, "utf8").split(/\r?\n/).find((entry) => entry.startsWith("VITE_ASSET_BASE_URL="));
    if (line !== undefined) {
      return line.slice("VITE_ASSET_BASE_URL=".length).trim();
    }
  }
  return fail("VITE_ASSET_BASE_URL is not set, in the environment or in web/.env.local.");
}

function filesUnder(dir) {
  return readdirSync(dir, { withFileTypes: true }).flatMap((entry) =>
    entry.isDirectory() ? filesUnder(join(dir, entry.name)) : [join(dir, entry.name)],
  );
}

const assetOrigin = new URL(assetBaseUrl()).origin;
if (!existsSync(DIST)) {
  fail("web/dist does not exist. Build first: pnpm build:vite");
}

// --- 1. Hosts in the bundle ------------------------------------------------------

const problems = [];
const tolerated = new Map();
const files = filesUnder(DIST);
for (const file of files) {
  // latin1 maps every byte to one character, so a binary file can be searched too.
  const text = readFileSync(file, "latin1");
  for (const [literal] of text.matchAll(URL_LITERAL)) {
    if (literal === assetOrigin || literal.startsWith(`${assetOrigin}/`)) {
      continue;
    }
    const known = NON_NETWORK_LITERALS.find((entry) =>
      entry.exact === true ? literal === entry.prefix : literal.startsWith(entry.prefix),
    );
    if (known !== undefined) {
      tolerated.set(known.prefix, (tolerated.get(known.prefix) ?? 0) + 1);
      continue;
    }
    problems.push(`${relative(ROOT, file).replaceAll("\\", "/")}: ${literal.slice(0, 120)}`);
  }
}

// --- 2. Hosts in the CSP -----------------------------------------------------------

const vercel = JSON.parse(readFileSync(VERCEL, "utf8"));
const csp = vercel.headers
  .flatMap((rule) => rule.headers)
  .find((header) => header.key.toLowerCase() === "content-security-policy")?.value;
if (csp === undefined) {
  fail("web/vercel.json has no Content-Security-Policy header.");
}
const cspHosts = [...new Set((csp.match(/[a-z][a-z0-9+.-]*:\/\/[^\s;]+/gi) ?? []).map((source) => source.replace(/\/+$/, "")))];
if (cspHosts.length !== 1 || cspHosts[0] !== assetOrigin) {
  problems.push(
    `web/vercel.json: the CSP names ${cspHosts.length === 0 ? "no host" : cspHosts.join(", ")}, ` +
      `but VITE_ASSET_BASE_URL is on ${assetOrigin}. They must be the same one host.`,
  );
}

// --- Result ------------------------------------------------------------------------

if (problems.length > 0) {
  console.error(`check-hosts: ${problems.length} problem${problems.length === 1 ? "" : "s"}`);
  console.error(`  The browser may contact the app's own host and ${assetOrigin}, and no other (TS §24.1).`);
  for (const problem of problems) {
    console.error(`  ${problem}`);
  }
  process.exit(1);
}

const shell = files.filter((file) => SHELL.test(file));
const sizes = shell.map((file) => ({
  name: relative(DIST, file).replaceAll("\\", "/"),
  gzip: gzipSync(readFileSync(file)).length,
}));
const total = sizes.reduce((sum, file) => sum + file.gzip, 0);
const kB = (bytes) => `${(bytes / 1000).toFixed(1)} kB`;

console.log(`check-hosts: ok. ${files.length} files in web/dist; the only host is ${assetOrigin}, the same as in the CSP.`);
for (const [prefix, count] of tolerated) {
  console.log(`  not a request: ${prefix} (${count}x)`);
}
console.log(`  app shell, gzip: ${kB(total)} (${sizes.map((file) => `${file.name} ${kB(file.gzip)}`).join(", ")})`);
