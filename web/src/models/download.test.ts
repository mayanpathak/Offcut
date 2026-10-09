import { beforeAll, describe, expect, it } from "vitest";

import manifest from "../config/model-manifest.json";
import { type Bytes, LIMITS } from "../gen/domain";
import type { AssetResult, fetchAsset } from "../net/asset-fetch";
import { OpfsError, paths } from "../persistence/opfs";
import {
  createDownloader,
  type DownloadDeps,
  localName,
  type ManifestFile,
  MODEL_PART_BYTES,
  ModelError,
  type VerifyDeps,
  verifyAndFinalize,
  VERIFY_SLICE_BYTES,
} from "./download";

const MIB = 1024 * 1024;
const MODEL_ID = "asr-en-v1";

/** `count` bytes that differ from place to place, so a part written at the wrong offset shows. */
function pattern(count: number): Uint8Array<ArrayBuffer> {
  const bytes = new Uint8Array(new ArrayBuffer(count));
  for (let i = 0; i < count; i += 1) {
    bytes[i] = (i * 31 + (i >>> 8) * 7 + (i >>> 16)) & 0xff;
  }
  return bytes;
}

function sameBytes(a: Uint8Array | undefined, b: Uint8Array): boolean {
  if (a?.length !== b.length) {
    return false;
  }
  for (let i = 0; i < b.length; i += 1) {
    if (a[i] !== b[i]) {
      return false;
    }
  }
  return true;
}

function fileOf(source: Uint8Array, sha256 = ""): ManifestFile {
  return { path: `models/${MODEL_ID}/big.0123456789abcdef.bin`, bytes: source.length, sha256 };
}

const PART = paths.modelPart(MODEL_ID, "big.bin");
const FINAL = paths.modelFile(MODEL_ID, "big.bin");

/** What the fake host does with one request. The last step repeats. */
type Step = "serve" | "whole" | "offline" | "status" | "empty" | "long" | "abort";

type Call = { path: string; range: { start: number; endInclusive: number } | undefined };

/** A fake OPFS: a `Map` of byte arrays, with the writes it was asked to make. */
function fakeOpfs(failWith?: OpfsError) {
  const files = new Map<string, Uint8Array<ArrayBuffer>>();
  const writes: { path: string; offset: Bytes; length: number }[] = [];
  const truncates: { path: string; length: Bytes }[] = [];
  const resize = (path: string, length: number): Uint8Array<ArrayBuffer> => {
    const next = new Uint8Array(new ArrayBuffer(length));
    next.set((files.get(path) ?? next).subarray(0, length));
    files.set(path, next);
    return next;
  };
  const api: DownloadDeps["opfs"] & VerifyDeps["opfs"] = {
    size: () => Promise.resolve(null),
    writeAt: (path, offset, data) => {
      if (failWith !== undefined) {
        return Promise.reject(failWith);
      }
      writes.push({ path, offset, length: data.length });
      const current = files.get(path);
      const target = current !== undefined && current.length >= offset + data.length ? current : resize(path, offset + data.length);
      target.set(data, offset);
      return Promise.resolve();
    },
    truncate: (path, length) => {
      truncates.push({ path, length });
      resize(path, length);
      return Promise.resolve();
    },
    getFile: (path) => {
      const bytes = files.get(path);
      return bytes === undefined ? Promise.reject(new OpfsError("io", "absent")) : Promise.resolve(new File([bytes], "part"));
    },
    move: (from, to) => {
      const bytes = files.get(from);
      if (bytes === undefined) {
        return Promise.reject(new OpfsError("io", "absent"));
      }
      files.set(to, bytes);
      files.delete(from);
      return Promise.resolve();
    },
    remove: (path) => {
      files.delete(path);
      return Promise.resolve();
    },
  };
  return { api, files, writes, truncates };
}

/** `createDownloader` over a fake host that serves `source`, a fake OPFS and a `sleep` that only records. */
function setup(source: Uint8Array<ArrayBuffer>, steps: Step[] = ["serve"], failWith?: OpfsError) {
  const calls: Call[] = [];
  const sleeps: number[] = [];
  const store = fakeOpfs(failWith);
  const controller = new AbortController();

  const fake: typeof fetchAsset = (path, o) => {
    calls.push({ path, range: o.range === undefined ? undefined : { start: o.range.start, endInclusive: o.range.endInclusive } });
    const step = steps[Math.min(calls.length, steps.length) - 1] ?? "serve";
    const start = o.range?.start ?? 0;
    const end = (o.range?.endInclusive ?? source.length - 1) + 1;
    const answer = (status: 200 | 206, body: Uint8Array<ArrayBuffer>): Promise<AssetResult> =>
      Promise.resolve({ ok: true, status, response: new Response(body, { status }) });
    switch (step) {
      case "serve":
        return answer(206, source.slice(start, end));
      case "whole":
        return answer(200, source.slice());
      case "empty":
        return answer(206, new Uint8Array(new ArrayBuffer(0)));
      case "long":
        return answer(206, pattern(end - start + 1));
      case "offline":
        return Promise.resolve({ ok: false, cause: "offline" });
      case "status":
        return Promise.resolve({ ok: false, cause: "status", status: 503 });
      case "abort":
        controller.abort();
        return Promise.resolve({ ok: false, cause: "aborted" });
    }
  };

  const { fetchRanged } = createDownloader({
    fetchAsset: fake,
    opfs: store.api,
    sleep: (ms) => {
      sleeps.push(ms);
      return Promise.resolve();
    },
  });

  let received = 0;
  const run = (resumeFrom: Bytes, file = fileOf(source)) =>
    fetchRanged(file, {
      resumeFrom,
      signal: controller.signal,
      onBytes: (n) => {
        received += n;
      },
    });
  return { run, calls, sleeps, store, received: () => received };
}

const ranges = (calls: Call[]) => calls.map((call) => (call.range === undefined ? "none" : `${String(call.range.start)}-${String(call.range.endInclusive)}`));

// A test may not turn a number into `Bytes` (D-59). The three byte counts the
// cases start from are taken from the downloader itself: a download that is
// told it already has more than the file truncates its part to 0, and then
// writes at 0, 8 MiB and 16 MiB.
let ZERO: Bytes;
let ONE_PART: Bytes;
let TWO_PARTS: Bytes;

beforeAll(async () => {
  const { run, store } = setup(pattern(20 * MIB));
  await run(LIMITS.MAX_FILE_SIZE);
  const zero = store.truncates[0]?.length;
  const onePart = store.writes[1]?.offset;
  const twoParts = store.writes[2]?.offset;
  if (zero === undefined || onePart === undefined || twoParts === undefined) {
    throw new Error("the downloader did not truncate the part and write three times");
  }
  expect([zero, onePart, twoParts]).toEqual([0, MODEL_PART_BYTES, 2 * MODEL_PART_BYTES]);
  [ZERO, ONE_PART, TWO_PARTS] = [zero, onePart, twoParts];
});

describe("fetchRanged", () => {
  it("downloads a 20 MiB file into an empty part in three ranged requests", async () => {
    const source = pattern(20 * MIB);
    const { run, calls, store, received } = setup(source);
    await run(ZERO);
    expect(ranges(calls)).toEqual(["0-8388607", "8388608-16777215", "16777216-20971519"]);
    expect(sameBytes(store.files.get(PART), source)).toBe(true);
    expect(received()).toBe(20 * MIB);
  });

  it("resumes at the size of the part", async () => {
    const source = pattern(20 * MIB);
    const { run, calls, store } = setup(source);
    await store.api.writeAt(PART, ZERO, source.slice(0, ONE_PART));
    await run(ONE_PART);
    expect(ranges(calls)).toEqual(["8388608-16777215", "16777216-20971519"]);
    expect(sameBytes(store.files.get(PART), source)).toBe(true);
  });

  it("starts the part again from 0 when the host answers 200 with the whole file", async () => {
    const source = pattern(20 * MIB);
    const { run, calls, store } = setup(source, ["whole"]);
    await store.api.writeAt(PART, ZERO, pattern(ONE_PART).reverse());
    store.truncates.length = 0;
    store.writes.length = 0;
    await run(ONE_PART);
    expect(calls).toHaveLength(1);
    expect(store.truncates).toEqual([{ path: PART, length: 0 }]);
    expect(store.writes[0]?.offset).toBe(0);
    expect(sameBytes(store.files.get(PART), source)).toBe(true);
  });

  it("waits 1 s after one offline answer and asks for the same range again", async () => {
    const source = pattern(MIB);
    const { run, calls, sleeps, store } = setup(source, ["offline", "serve"]);
    await run(ZERO);
    expect(sleeps).toEqual([1000]);
    expect(ranges(calls)).toEqual(["0-1048575", "0-1048575"]);
    expect(sameBytes(store.files.get(PART), source)).toBe(true);
  });

  it("gives up with E_MODEL_DOWNLOAD after four failures in a row, and keeps the part", async () => {
    const source = pattern(20 * MIB);
    const { run, calls, sleeps, store } = setup(source, ["serve", "status"]);
    await expect(run(ZERO)).rejects.toMatchObject({ name: "ModelError", code: "E_MODEL_DOWNLOAD" });
    expect(sleeps).toEqual([1000, 3000, 9000]);
    expect(calls).toHaveLength(5);
    expect(sameBytes(store.files.get(PART), source.slice(0, ONE_PART))).toBe(true);
  });

  it("counts failures in a row: a success sets the count back", async () => {
    const source = pattern(20 * MIB);
    const { run, sleeps, store } = setup(source, ["offline", "serve", "status", "serve"]);
    await run(ZERO);
    expect(sleeps).toEqual([1000, 1000]);
    expect(sameBytes(store.files.get(PART), source)).toBe(true);
  });

  it("counts a 206 with no bytes, and a body longer than the range, as failed attempts", async () => {
    const source = pattern(MIB);
    const { run, calls, sleeps, store } = setup(source, ["empty", "long", "serve"]);
    await run(ZERO);
    expect(sleeps).toEqual([1000, 3000]);
    expect(calls).toHaveLength(3);
    expect(store.writes).toHaveLength(1);
    expect(sameBytes(store.files.get(PART), source)).toBe(true);
  });

  it("rejects with the abort when the signal aborts mid-file, keeps the part and does not wait", async () => {
    const source = pattern(20 * MIB);
    const { run, calls, sleeps, store } = setup(source, ["serve", "abort"]);
    await expect(run(ZERO)).rejects.toMatchObject({ name: "AbortError" });
    expect(calls).toHaveLength(2);
    expect(sleeps).toEqual([]);
    expect(sameBytes(store.files.get(PART), source.slice(0, ONE_PART))).toBe(true);
  });

  it("names a full disk E_MODEL_STORAGE and any other storage failure E_STORAGE_IO", async () => {
    const source = pattern(MIB);
    const full = setup(source, ["serve"], new OpfsError("quota", "full"));
    await expect(full.run(ZERO)).rejects.toMatchObject({ name: "ModelError", code: "E_MODEL_STORAGE" });
    const broken = setup(source, ["serve"], new OpfsError("io", "broken"));
    await expect(broken.run(ZERO)).rejects.toMatchObject({ name: "ModelError", code: "E_STORAGE_IO" });
    expect(full.sleeps.concat(broken.sleeps)).toEqual([]);
  });

  it("truncates a part that is larger than the file and starts at 0", async () => {
    const source = pattern(4 * MIB);
    const { run, calls, store } = setup(source);
    await store.api.writeAt(PART, ZERO, pattern(TWO_PARTS).reverse());
    store.truncates.length = 0;
    await run(TWO_PARTS);
    expect(store.truncates).toEqual([{ path: PART, length: 0 }]);
    expect(ranges(calls)).toEqual(["0-4194303"]);
    expect(sameBytes(store.files.get(PART), source)).toBe(true);
  });

  it("asks for the manifest path, with a range and nothing else, on every request", async () => {
    const source = pattern(20 * MIB);
    const file = fileOf(source);
    const { run, calls } = setup(source, ["offline", "serve", "status", "serve"]);
    await run(ZERO, file);
    expect(calls).toHaveLength(5);
    for (const call of calls) {
      expect(call.path).toBe(file.path);
      expect(call.path.includes("?")).toBe(false);
      expect(call.range).toBeDefined();
    }
  });
});

/** Not SHA-256: a digest that changes with any byte and with the order of the bytes. */
function fakeDigest() {
  let a = 0x811c9dc5;
  let b = 0;
  const slices: number[] = [];
  return {
    slices,
    update(chunk: Uint8Array) {
      slices.push(chunk.length);
      for (const byte of chunk) {
        a = Math.imul(a ^ byte, 0x01000193) >>> 0;
        b = (b + a) >>> 0;
      }
    },
    finalizeHex: () => (a.toString(16).padStart(8, "0") + b.toString(16).padStart(8, "0")).repeat(4),
  };
}

function digestOf(bytes: Uint8Array): string {
  const digest = fakeDigest();
  digest.update(bytes);
  return digest.finalizeHex();
}

function verifySetup(onDisk: Uint8Array<ArrayBuffer>) {
  const store = fakeOpfs();
  store.files.set(PART, onDisk);
  const digest = fakeDigest();
  let pauses = 0;
  const deps: VerifyDeps = {
    opfs: store.api,
    newSha256: () => Promise.resolve(digest),
    pause: () => {
      pauses += 1;
      return Promise.resolve();
    },
  };
  return { deps, store, digest, pauses: () => pauses };
}

describe("verifyAndFinalize", () => {
  it("gives a file with the manifest's hash its final name", async () => {
    const source = pattern(MIB);
    const { deps, store } = verifySetup(source);
    await verifyAndFinalize(MODEL_ID, fileOf(source, digestOf(source)), deps);
    expect([...store.files.keys()]).toEqual([FINAL]);
    expect(sameBytes(store.files.get(FINAL), source)).toBe(true);
  });

  it("removes a file with one flipped byte and names the failure E_MODEL_HASH (INV-20)", async () => {
    const source = pattern(MIB);
    const damaged = source.slice();
    damaged[MIB / 2] = (damaged[MIB / 2] ?? 0) ^ 0x01;
    const { deps, store } = verifySetup(damaged);
    const check = verifyAndFinalize(MODEL_ID, fileOf(source, digestOf(source)), deps);
    await expect(check).rejects.toBeInstanceOf(ModelError);
    await expect(check).rejects.toMatchObject({ code: "E_MODEL_HASH" });
    expect([...store.files.keys()]).toEqual([]);
  });

  it("feeds the hash a 9 MiB file in slices of at most 4 MiB, with a pause between them", async () => {
    const source = pattern(9 * MIB);
    const { deps, digest, pauses } = verifySetup(source);
    await verifyAndFinalize(MODEL_ID, fileOf(source, digestOf(source)), deps);
    expect(digest.slices).toEqual([VERIFY_SLICE_BYTES, VERIFY_SLICE_BYTES, MIB]);
    expect(pauses()).toBe(2);
  });
});

describe("localName", () => {
  it("is the last segment of the path without its hash segment (D-62)", () => {
    const file = { path: "models/asr-en-v1/encoder_model.8465fcc35d96d468.onnx", bytes: 1, sha256: "" };
    expect(localName(file)).toBe("encoder_model.onnx");
  });
});

describe("model-manifest.json", () => {
  it("adds up, fits the limit, and holds a hash and a path of the right form for every file", () => {
    const sum = manifest.files.reduce((total, file) => total + file.bytes, 0);
    expect(manifest.totalBytes).toBe(sum);
    expect(manifest.totalBytes).toBeLessThanOrEqual(260_000_000);
    expect(manifest.files.length).toBeGreaterThan(0);
    for (const file of manifest.files) {
      expect(file.sha256).toMatch(/^[0-9a-f]{64}$/);
      expect(file.path.startsWith(`models/${manifest.modelId}/`)).toBe(true);
      expect(localName(file).includes("/")).toBe(false);
    }
  });
});
