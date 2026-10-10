// Getting one model file from the asset host into OPFS, and checking it
// (TS §16.3). A file arrives in parts of 8 MiB under the name `<name>.part`,
// so that a closed tab loses at most one part. It gets its final name only
// after its SHA-256 matched the manifest's (INV-20).

import { fetchAsset } from "../net/asset-fetch";
import type { Bytes, ErrorCode } from "../gen/domain";
import * as opfs from "../persistence/opfs";

export const MODEL_PART_BYTES = 8 * 1024 * 1024;
export const MODEL_RETRIES = 3;
/** The wait before the first, second and third retry of one request. */
export const MODEL_BACKOFF_MS = [1_000, 3_000, 9_000] as const;
/** How much of a downloaded file is hashed in one go, between two turns of the event loop (D-36). */
export const VERIFY_SLICE_BYTES = 4 * 1024 * 1024;

/** One entry of `config/model-manifest.json` (TS §16.1). */
export type ManifestFile = { path: string; bytes: number; sha256: string };

export type ModelErrorCode = Extract<ErrorCode, "E_MODEL_DOWNLOAD" | "E_MODEL_HASH" | "E_MODEL_STORAGE" | "E_STORAGE_IO">;

/** Why a file could not be downloaded or did not pass its check. */
export class ModelError extends Error {
  readonly code: ModelErrorCode;

  constructor(code: ModelErrorCode, cause?: unknown) {
    super(code, { cause });
    this.name = "ModelError";
    this.code = code;
  }
}

export type DownloadDeps = {
  fetchAsset: typeof fetchAsset;
  opfs: Pick<typeof opfs, "size" | "writeAt" | "truncate">;
  sleep: (ms: number) => Promise<void>;
};

export type VerifyDeps = {
  opfs: Pick<typeof opfs, "getFile" | "move" | "remove">;
  /** A new SHA-256 over the chunks it is given, with the digest in lower-case hex. */
  newSha256: () => Promise<{ update(chunk: Uint8Array): void; finalizeHex(): string }>;
  /** Gives the event loop one turn. */
  pause: () => Promise<void>;
};

/** The one place of this file that turns a number into `Bytes` (D-59). */
function toBytes(n: number): Bytes {
  return n as Bytes;
}

const STORED_NAME = /^(?<stem>.+)\.[0-9a-f]{16}\.(?<extension>[^.]+)$/;

/**
 * The name of a model file in OPFS: the last segment of its path, without
 * the hash segment the upload gave it (D-62). The runtime asks for a file
 * by this name.
 */
export function localName(file: ManifestFile): string {
  const stored = file.path.slice(file.path.lastIndexOf("/") + 1);
  const groups = STORED_NAME.exec(stored)?.groups;
  if (groups?.stem === undefined || groups.extension === undefined) {
    throw new Error("a manifest path ends in <name>.<hash>.<extension>");
  }
  return `${groups.stem}.${groups.extension}`;
}

/** The model a file belongs to: the second segment of `models/<modelId>/<file>`. */
function modelIdOf(file: ManifestFile): string {
  const [root, modelId, stored, ...rest] = file.path.split("/");
  if (root !== "models" || modelId === undefined || stored === undefined || rest.length > 0) {
    throw new Error("a manifest path is models/<modelId>/<file>");
  }
  return modelId;
}

/** A storage failure, as the code a person is shown for it. Anything else goes on as it was. */
function storageFailure(error: unknown): unknown {
  if (error instanceof opfs.OpfsError) {
    return new ModelError(error.kind === "quota" ? "E_MODEL_STORAGE" : "E_STORAGE_IO", error);
  }
  return error;
}

function isAbort(error: unknown): boolean {
  return error instanceof DOMException && error.name === "AbortError";
}

function aborted(signal: AbortSignal): never {
  signal.throwIfAborted();
  throw new DOMException("The download was aborted.", "AbortError");
}

/** What one request did: how far the file is now, or that the attempt failed. */
type Attempt = { have: number } | { failed: true; have: number };

export function createDownloader(deps: DownloadDeps): { fetchRanged: typeof fetchRanged } {
  /** Writes a body the host sent whole, from byte 0, in parts of 8 MiB. */
  async function writeWhole(part: string, file: ManifestFile, body: ReadableStream<Uint8Array>): Promise<Attempt> {
    await deps.opfs.truncate(part, toBytes(0));
    const reader = body.getReader();
    let chunks: Uint8Array[] = [];
    let buffered = 0;
    let written = 0;
    const flush = async (): Promise<void> => {
      if (buffered === 0) {
        return;
      }
      const data = new Uint8Array(new ArrayBuffer(buffered));
      let at = 0;
      for (const chunk of chunks) {
        data.set(chunk, at);
        at += chunk.length;
      }
      await deps.opfs.writeAt(part, toBytes(written), data);
      written += buffered;
      chunks = [];
      buffered = 0;
    };
    for (;;) {
      const { done, value } = await reader.read();
      if (done) {
        break;
      }
      chunks.push(value);
      buffered += value.length;
      if (written + buffered > file.bytes) {
        // Longer than the file: not the file.
        await reader.cancel();
        break;
      }
      if (buffered >= MODEL_PART_BYTES) {
        await flush();
      }
    }
    if (written + buffered !== file.bytes) {
      await deps.opfs.truncate(part, toBytes(0));
      return { failed: true, have: 0 };
    }
    await flush();
    return { have: written };
  }

  /** One request for the bytes from `have` on, and what came of it. */
  async function attempt(
    part: string,
    file: ManifestFile,
    have: number,
    o: { signal: AbortSignal; onBytes: (n: Bytes) => void },
  ): Promise<Attempt> {
    const endInclusive = Math.min(have + MODEL_PART_BYTES, file.bytes) - 1;
    const result = await deps.fetchAsset(file.path, {
      range: { start: toBytes(have), endInclusive: toBytes(endInclusive) },
      signal: o.signal,
    });
    if (!result.ok) {
      if (result.cause === "aborted") {
        aborted(o.signal);
      }
      return { failed: true, have };
    }

    if (result.status === 200) {
      // The host ignored the range and sent the file from its first byte.
      if (result.response.body === null) {
        return { failed: true, have };
      }
      const whole = await writeWhole(part, file, result.response.body);
      if (!("failed" in whole)) {
        o.onBytes(toBytes(whole.have));
      }
      return whole;
    }

    const body = new Uint8Array(await result.response.arrayBuffer());
    if (body.length === 0 || body.length > endInclusive - have + 1) {
      return { failed: true, have };
    }
    await deps.opfs.writeAt(part, toBytes(have), body);
    o.onBytes(toBytes(body.length));
    return { have: have + body.length };
  }

  return {
    async fetchRanged(file, o) {
      const part = opfs.paths.modelPart(modelIdOf(file), localName(file));
      try {
        let have: number = o.resumeFrom;
        if (have > file.bytes) {
          await deps.opfs.truncate(part, toBytes(0));
          have = 0;
        }
        let failures = 0;
        while (have < file.bytes) {
          if (o.signal.aborted) {
            aborted(o.signal);
          }
          let outcome: Attempt;
          try {
            outcome = await attempt(part, file, have, o);
          } catch (error) {
            if (o.signal.aborted || isAbort(error) || error instanceof opfs.OpfsError) {
              throw error;
            }
            // The connection broke while the body was being read.
            outcome = { failed: true, have };
          }
          have = outcome.have;
          if (!("failed" in outcome)) {
            failures = 0;
            continue;
          }
          const wait = MODEL_BACKOFF_MS[failures];
          if (failures >= MODEL_RETRIES || wait === undefined) {
            throw new ModelError("E_MODEL_DOWNLOAD");
          }
          failures += 1;
          await deps.sleep(wait);
        }
      } catch (error) {
        throw storageFailure(error);
      }
    },
  };
}

const downloader = createDownloader({
  fetchAsset,
  opfs,
  sleep: (ms) =>
    new Promise((resolve) => {
      setTimeout(resolve, ms);
    }),
});

/**
 * Downloads what `file` still lacks into its part file, from byte
 * `resumeFrom` on, which is the size the part file has. A request that
 * fails is made again after 1, 3 and 9 s; a fourth failure in a row ends
 * the download with `E_MODEL_DOWNLOAD`. The part file is kept in every case
 * but one: a host that sends the whole file makes it start again from 0.
 */
export function fetchRanged(
  file: ManifestFile,
  o: { resumeFrom: Bytes; signal: AbortSignal; onBytes: (n: Bytes) => void },
): Promise<void> {
  return downloader.fetchRanged(file, o);
}

/**
 * Hashes the part file of `file` and, if the hash is the manifest's, gives
 * the file its final name. A file with another hash is removed, and the
 * failure is `E_MODEL_HASH`. No file therefore has a final name whose bytes
 * were not checked (INV-20).
 */
export async function verifyAndFinalize(modelId: string, file: ManifestFile, deps: VerifyDeps): Promise<void> {
  const name = localName(file);
  const part = opfs.paths.modelPart(modelId, name);
  try {
    const blob = await deps.opfs.getFile(part);
    const sha256 = await deps.newSha256();
    for (let offset = 0; offset < blob.size; offset += VERIFY_SLICE_BYTES) {
      if (offset > 0) {
        await deps.pause();
      }
      sha256.update(new Uint8Array(await blob.slice(offset, offset + VERIFY_SLICE_BYTES).arrayBuffer()));
    }
    if (sha256.finalizeHex() !== file.sha256) {
      await deps.opfs.remove(part);
      throw new ModelError("E_MODEL_HASH");
    }
    await deps.opfs.move(part, opfs.paths.modelFile(modelId, name));
  } catch (error) {
    throw storageFailure(error);
  }
}
