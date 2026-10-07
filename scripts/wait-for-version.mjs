// Deploy gate (TS §33): waits until an API answers /healthz with one exact
// commit. CI runs it twice after a deploy: against the API host, before the
// web app is deployed, and through the web app's rewrite afterwards. A new
// client must never reach an old server.
//
// Usage: node scripts/wait-for-version.mjs <origin> <commit> [seconds]
//   node scripts/wait-for-version.mjs https://<render-host> $GITHUB_SHA
//   node scripts/wait-for-version.mjs https://<project>.vercel.app $GITHUB_SHA 120
//
// <seconds> is how long to keep trying; 300 when left out. A sleeping API on
// the free plan needs about a minute to answer its first request.

const HEALTHZ_PATH = "/api/v1/healthz";
const DEFAULT_WAIT_S = 300;
const PAUSE_MS = 10_000;
/** One request may wait this long: longer than a cold start. */
const REQUEST_TIMEOUT_MS = 70_000;

function fail(message) {
  console.error(`wait-for-version: ${message}`);
  process.exit(1);
}

const [origin, commit, seconds] = process.argv.slice(2);
if (origin === undefined || commit === undefined) {
  fail("usage: node scripts/wait-for-version.mjs <origin> <commit> [seconds]");
}
let url;
try {
  url = new URL(HEALTHZ_PATH, origin);
} catch {
  fail(`not a URL: ${origin}`);
}
const waitS = seconds === undefined ? DEFAULT_WAIT_S : Number(seconds);
if (!Number.isFinite(waitS) || waitS <= 0) {
  fail(`not a number of seconds: ${seconds}`);
}

/** What the API answered, in a few words. */
async function ask() {
  try {
    const response = await fetch(url, { signal: AbortSignal.timeout(REQUEST_TIMEOUT_MS) });
    const text = await response.text();
    if (response.status !== 200) {
      return { seen: `status ${response.status}` };
    }
    const version = JSON.parse(text).version;
    return typeof version === "string" ? { version, seen: `version ${version}` } : { seen: "no version in the answer" };
  } catch (error) {
    return { seen: error instanceof Error ? error.message : String(error) };
  }
}

const deadline = Date.now() + waitS * 1000;
let found = false;
for (let attempt = 1; ; attempt += 1) {
  const { version, seen } = await ask();
  if (version === commit) {
    found = true;
    break;
  }
  console.log(`  attempt ${attempt}: ${seen}`);
  if (Date.now() + PAUSE_MS > deadline) {
    break;
  }
  await new Promise((done) => setTimeout(done, PAUSE_MS));
}

if (found) {
  console.log(`wait-for-version: ok. ${url.href} runs ${commit}.`);
} else {
  console.error(`wait-for-version: ${url.href} did not report ${commit} within ${waitS} s.`);
  // Not process.exit: Node on Windows aborts when it exits just after a fetch.
  process.exitCode = 1;
}
