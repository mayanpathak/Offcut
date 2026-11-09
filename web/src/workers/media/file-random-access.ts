// Reading a dropped `File` where it is, for the demuxer (v6.1implementation
// D-400, TS §43.3). Nothing is copied: each read is a `FileReaderSync` read
// of one slice, in 1 MiB blocks aligned on the file, with the last eight
// blocks kept (least recently used first out). Workers only.

import type { ByteReader } from "../../wasm/load-core";

/** The size of a block read from the file. */
export const BLOCK_BYTES = 1 << 20;
/** How many blocks are kept: the demuxer reads small boxes often and samples in runs. */
export const CACHE_BLOCKS = 8;

/** Why a read failed: the file moved, was deleted or changed (the browser's `NotReadableError`). */
export class SourceReadError extends Error {
  constructor(cause: unknown) {
    super("the source file cannot be read", { cause });
    this.name = "SourceReadError";
  }
}

/** How one slice of the file is read; the default is `FileReaderSync`. */
export type ReadSlice = (blob: Blob) => ArrayBuffer;

const readWithFileReaderSync: ReadSlice = (blob) => new FileReaderSync().readAsArrayBuffer(blob);

/**
 * A `ByteReader` over `file`. A read that crosses blocks joins them. Any
 * failure of the browser's read throws `SourceReadError` (the media worker
 * turns it into `E_SOURCE_READ`).
 */
export function fileRandomAccess(file: Blob, readSlice: ReadSlice = readWithFileReaderSync): ByteReader {
  const size = file.size;
  // Block index to bytes, oldest first.
  const cache = new Map<number, Uint8Array>();

  function block(index: number): Uint8Array {
    const hit = cache.get(index);
    if (hit !== undefined) {
      cache.delete(index);
      cache.set(index, hit);
      return hit;
    }
    const start = index * BLOCK_BYTES;
    let bytes: Uint8Array;
    try {
      bytes = new Uint8Array(readSlice(file.slice(start, Math.min(start + BLOCK_BYTES, size))));
    } catch (error) {
      throw new SourceReadError(error);
    }
    cache.set(index, bytes);
    if (cache.size > CACHE_BLOCKS) {
      const oldest = cache.keys().next();
      if (oldest.done !== true) {
        cache.delete(oldest.value);
      }
    }
    return bytes;
  }

  return {
    size: () => size,
    readAt(offset: number, length: number): Uint8Array {
      const end = Math.min(offset + length, size);
      const out = new Uint8Array(Math.max(0, end - offset));
      let at = offset;
      while (at < end) {
        const index = Math.floor(at / BLOCK_BYTES);
        const bytes = block(index);
        const from = at - index * BLOCK_BYTES;
        const take = Math.min(bytes.length - from, end - at);
        if (take <= 0) {
          // The file is shorter than it said: it changed under us.
          throw new SourceReadError(new Error("short read"));
        }
        out.set(bytes.subarray(from, from + take), at - offset);
        at += take;
      }
      return out;
    },
  };
}
