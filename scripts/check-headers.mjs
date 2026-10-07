// Header gate (TS §24.3, §33). Fetches `/` from a running deployment and
// compares every header of the `/(.*)` rule of web/vercel.json with the
// response, value for value. A header that the host drops, rewrites or adds
// a second value to fails here, before a browser meets it.
//
// Usage: node scripts/check-headers.mjs <url>
//   node scripts/check-headers.mjs https://<project>.vercel.app
//   node scripts/check-headers.mjs http://localhost:4173     (vite preview)

import { readFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const ROOT = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const VERCEL = resolve(ROOT, "web/vercel.json");
/** The rule that applies to every path. */
const ALL_PATHS = "/(.*)";
const TIMEOUT_MS = 30_000;

function fail(message) {
  console.error(`check-headers: ${message}`);
  process.exit(1);
}

const [target] = process.argv.slice(2);
if (target === undefined) {
  fail("usage: node scripts/check-headers.mjs <url>");
}
let url;
try {
  url = new URL("/", target);
} catch {
  fail(`not a URL: ${target}`);
}

const rule = JSON.parse(readFileSync(VERCEL, "utf8")).headers.find((entry) => entry.source === ALL_PATHS);
if (rule === undefined || rule.headers.length === 0) {
  fail(`web/vercel.json has no header rule for ${ALL_PATHS}.`);
}

let response;
try {
  // A redirect is not followed: the headers must be on the page itself.
  response = await fetch(url, { redirect: "manual", signal: AbortSignal.timeout(TIMEOUT_MS) });
} catch (error) {
  fail(`cannot fetch ${url.href}: ${error instanceof Error ? error.message : String(error)}`);
}
// Only the headers are needed. From here on the script ends by returning, not
// by process.exit: Node on Windows aborts when it exits during a live fetch.
await response.body?.cancel();

const problems = [];
if (response.status !== 200) {
  problems.push(`status: ${response.status}, not 200`);
}
for (const { key, value } of rule.headers) {
  const actual = response.headers.get(key);
  if (actual === null) {
    problems.push(`${key}: missing`);
  } else if (actual !== value) {
    problems.push(`${key}:\n      expected  ${value}\n      received  ${actual}`);
  }
}

if (problems.length > 0) {
  console.error(`check-headers: ${problems.length} problem${problems.length === 1 ? "" : "s"} at ${url.href}`);
  for (const problem of problems) {
    console.error(`  ${problem}`);
  }
  process.exitCode = 1;
} else {
  console.log(`check-headers: ok. ${url.href} sends the ${rule.headers.length} headers of web/vercel.json, value for value.`);
}
