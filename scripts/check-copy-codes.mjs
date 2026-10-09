// Copy gate (TS §11.3, v1implementation D-13). Every code a person can be
// shown must have its words in web/src/copy/messages.ts, or be on the list
// of codes whose words are due in a later version.
//
// Fails when:
//   - a code has neither copy nor a pending entry;
//   - a pending code already has copy (remove it from the list: the list
//     can only shrink, and must be empty at V7);
//   - the pending list names a code that does not exist.
//
// Usage: node scripts/check-copy-codes.mjs

import { readFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const ROOT = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const DOMAIN = resolve(ROOT, "web/src/gen/domain.ts");
const MESSAGES = resolve(ROOT, "web/src/copy/messages.ts");

/**
 * Codes whose copy belongs to a flow that does not exist yet. An entry is
 * removed in the change that writes its copy.
 */
const COPY_PENDING = [
  // The 14 rejections: due in V3, with real ingest.
  "REJECT_CONTAINER",
  "REJECT_CORRUPT",
  "REJECT_NO_VIDEO",
  "REJECT_NO_AUDIO",
  "REJECT_MULTI_AUDIO_TRACK",
  "REJECT_VIDEO_CODEC",
  "REJECT_HEVC",
  "REJECT_AUDIO_CODEC",
  "REJECT_DURATION",
  "REJECT_FILE_SIZE",
  "REJECT_RESOLUTION",
  "REJECT_FRAME_RATE",
  "REJECT_DECODE_UNSUPPORTED",
  "REJECT_NO_SPEECH",
  // Every error but the six a person can meet in V1 and the three of the
  // model download, written in V2: due by V7.
  "E_STORAGE_QUOTA",
  "E_STORAGE_IO",
  "E_DECODE_AUDIO",
  "E_DECODE_VIDEO",
  "E_ASR_RUNTIME",
  "E_ASR_OOM",
  "E_DSP",
  "E_GPU_INIT",
  "E_GPU_LOST",
  "E_ENCODE_VIDEO",
  "E_ENCODE_AUDIO",
  "E_MUX",
  "E_AUTH_LINK_INVALID",
  "E_AUTH_LINK_EXPIRED",
  "E_AUTH_SESSION_EXPIRED",
  "E_AUTH_EMAIL_UNAVAILABLE",
  "E_BILLING_UNAVAILABLE",
  "E_BILLING_PENDING",
  "E_ENTITLEMENT_INVALID",
  "E_ENTITLEMENT_EXPIRED",
];

/** The three code lists of gen/domain.ts. Each is written one code per line. */
const LISTS = ["ERROR_CODES", "REJECT_REASONS", "UNSUPPORTED_REASONS"];
/** A list none of whose codes may be pending: its copy is needed now. */
const NEVER_PENDING = ["UNSUPPORTED_REASONS"];

function readCodes(source, list) {
  const lines = source.split(/\r?\n/);
  const start = lines.indexOf(`export const ${list} = [`);
  const end = lines.indexOf("] as const;", start);
  if (start < 0 || end < 0) {
    throw new Error(`gen/domain.ts: cannot find the list ${list}`);
  }
  return lines.slice(start + 1, end).map((line) => {
    const code = /^\s*"([A-Z0-9_]+)",$/.exec(line)?.[1];
    if (code === undefined) {
      throw new Error(`gen/domain.ts: cannot read this line of ${list}: ${line}`);
    }
    return code;
  });
}

const domain = readFileSync(DOMAIN, "utf8");
const messages = readFileSync(MESSAGES, "utf8");

/** The codes that appear in messages.ts as the key of an entry. */
const withCopy = new Set([...messages.matchAll(/^\s*((?:E|REJECT|UNSUPPORTED)_[A-Z0-9_]+):/gm)].map((match) => match[1]));
const pending = new Set(COPY_PENDING);
const problems = [];
const known = new Set();
const counts = [];

for (const list of LISTS) {
  const codes = readCodes(domain, list);
  let written = 0;
  for (const code of codes) {
    known.add(code);
    const hasCopy = withCopy.has(code);
    const isPending = pending.has(code);
    written += hasCopy ? 1 : 0;
    if (hasCopy && isPending) {
      problems.push(`${code}: has copy and is still in COPY_PENDING. Remove it from the list.`);
    } else if (!hasCopy && !isPending) {
      problems.push(`${code}: has no copy in messages.ts and is not in COPY_PENDING.`);
    } else if (isPending && NEVER_PENDING.includes(list)) {
      problems.push(`${code}: a code of ${list} may not be pending; it needs copy now.`);
    }
  }
  counts.push(`${list} ${written} of ${codes.length}`);
}

for (const code of COPY_PENDING.filter((code) => !known.has(code))) {
  problems.push(`${code}: is in COPY_PENDING, but gen/domain.ts has no such code.`);
}
for (const code of [...withCopy].filter((code) => !known.has(code))) {
  problems.push(`${code}: has copy in messages.ts, but gen/domain.ts has no such code.`);
}
if (pending.size !== COPY_PENDING.length) {
  problems.push("COPY_PENDING names a code twice.");
}

if (problems.length > 0) {
  console.error(`check-copy-codes: ${problems.length} problem${problems.length === 1 ? "" : "s"}`);
  for (const problem of problems) {
    console.error(`  ${problem}`);
  }
  process.exit(1);
}
console.log(`check-copy-codes: ok. With copy: ${counts.join(", ")}. Pending: ${COPY_PENDING.length}.`);
