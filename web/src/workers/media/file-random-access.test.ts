import { describe, expect, it } from "vitest";

import { BLOCK_BYTES, CACHE_BLOCKS, fileRandomAccess, SourceReadError } from "./file-random-access";

// Blob has no synchronous read in Node; the test keeps the bytes beside each slice.
const kept = new WeakMap<Blob, Uint8Array>();
function syncBytes(blob: Blob): ArrayBuffer {
  const bytes = kept.get(blob);
  if (bytes === undefined) {
    throw new Error("unknown slice");
  }
  return bytes.slice().buffer;
}

/** A Blob whose slices remember their bytes, so `syncBytes` can read them. */
class MemoryBlob extends Blob {
  constructor(private readonly data: Uint8Array<ArrayBuffer>) {
    super([data]);
    kept.set(this, data);
  }
  override slice(start = 0, end = this.data.length): Blob {
    return new MemoryBlob(this.data.subarray(start, end));
  }
}

/** Byte equality without a deep compare: Vitest's is slow on megabytes. */
function same(a: Uint8Array, b: Uint8Array): boolean {
  if (a.length !== b.length) {
    return false;
  }
  for (let i = 0; i < a.length; i += 1) {
    if (a[i] !== b[i]) {
      return false;
    }
  }
  return true;
}

function memory(size: number) {
  const bytes = Uint8Array.from({ length: size }, (_, i) => i % 251);
  const reads: number[] = [];
  const reader = fileRandomAccess(new MemoryBlob(bytes), (slice) => {
    reads.push(slice.size);
    return syncBytes(slice);
  });
  return { bytes, reader, reads };
}

describe("fileRandomAccess", () => {
  it("reads what is in the file, within a block and across blocks", () => {
    const { bytes, reader } = memory(3 * BLOCK_BYTES + 1234);
    expect(reader.size()).toBe(3 * BLOCK_BYTES + 1234);
    expect(same(reader.readAt(10, 8), bytes.subarray(10, 18))).toBe(true);
    const across = reader.readAt(BLOCK_BYTES - 5, 2 * BLOCK_BYTES + 10);
    expect(across.length).toBe(2 * BLOCK_BYTES + 10);
    expect(same(across, bytes.subarray(BLOCK_BYTES - 5, 3 * BLOCK_BYTES + 5))).toBe(true);
    // The last, short block, and a read past the end, which gives what there is.
    expect(same(reader.readAt(3 * BLOCK_BYTES + 1200, 100), bytes.subarray(3 * BLOCK_BYTES + 1200))).toBe(true);
  });

  it("reads each block once while it is in the cache, and forgets the oldest", () => {
    const { reader, reads } = memory((CACHE_BLOCKS + 2) * BLOCK_BYTES);
    reader.readAt(0, 4);
    reader.readAt(100, 4);
    expect(reads).toHaveLength(1);
    for (let b = 1; b <= CACHE_BLOCKS; b += 1) {
      reader.readAt(b * BLOCK_BYTES, 4);
    }
    expect(reads).toHaveLength(CACHE_BLOCKS + 1);
    // Block 0 was the oldest: it is read again.
    reader.readAt(0, 4);
    expect(reads).toHaveLength(CACHE_BLOCKS + 2);
    // Block 2 is still kept.
    reader.readAt(2 * BLOCK_BYTES, 4);
    expect(reads).toHaveLength(CACHE_BLOCKS + 2);
    expect(reads.every((n) => n === BLOCK_BYTES)).toBe(true);
  });

  it("turns the browser's NotReadableError into SourceReadError", () => {
    const reader = fileRandomAccess(new MemoryBlob(new Uint8Array(100)), () => {
      throw new DOMException("gone", "NotReadableError");
    });
    expect(() => reader.readAt(0, 4)).toThrow(SourceReadError);
  });

  it("refuses a file that turns out shorter than it said", () => {
    const reader = fileRandomAccess(new MemoryBlob(new Uint8Array(100)), () => new ArrayBuffer(0));
    expect(() => reader.readAt(0, 4)).toThrow(SourceReadError);
  });
});
