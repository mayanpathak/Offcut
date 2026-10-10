// Where an export is written: one file in the origin private file system,
// through one synchronous handle (TS §21.1, §31). The muxer calls `writeAt`
// with the bytes of each box and each sample, at the place they belong.

import type { Bytes } from "../../gen/domain";
import { fileHandle, OpfsError, remove } from "../../persistence/opfs";

/** A failure as the storage layer tags it: the disk is full, or anything else. */
function tagged(error: unknown): OpfsError {
  if (error instanceof OpfsError) {
    return error;
  }
  const quota = error instanceof DOMException && error.name === "QuotaExceededError";
  return new OpfsError(quota ? "quota" : "io", error);
}

export class OpfsSink {
  readonly #path: string;
  /** `undefined` once the sink is closed or aborted. */
  #handle: FileSystemSyncAccessHandle | undefined;

  private constructor(path: string, handle: FileSystemSyncAccessHandle) {
    this.#path = path;
    this.#handle = handle;
  }

  /**
   * Makes the file, and the directories above it, and opens it for writing.
   * A file already at `path` is emptied: it is what an export that did not
   * finish left behind.
   */
  static async open(path: string): Promise<OpfsSink> {
    try {
      const handle = await (await fileHandle(path, { create: true })).createSyncAccessHandle();
      try {
        handle.truncate(0);
      } catch (error) {
        handle.close();
        throw error;
      }
      return new OpfsSink(path, handle);
    } catch (error) {
      throw tagged(error);
    }
  }

  /** Writes all of `data` at `offset`, before it returns. A write that is cut short is a failure. */
  writeAt(offset: Bytes, data: Uint8Array): void {
    if (this.#handle === undefined) {
      throw new OpfsError("io", new Error("the sink is closed"));
    }
    let written: number;
    try {
      written = this.#handle.write(data, { at: offset });
    } catch (error) {
      throw tagged(error);
    }
    if (written !== data.byteLength) {
      // The file system took a part of the bytes and no more: it is full.
      throw new OpfsError("quota", new Error("short write"));
    }
  }

  /** Makes what was written durable and gives the handle back. The file stays. */
  close(): Promise<void> {
    const handle = this.#handle;
    this.#handle = undefined;
    if (handle === undefined) {
      return Promise.resolve();
    }
    try {
      handle.flush();
    } catch (error) {
      handle.close();
      return Promise.reject(tagged(error));
    }
    handle.close();
    return Promise.resolve();
  }

  /** Gives the handle back and removes the file: nothing of a failed export stays. */
  async abort(): Promise<void> {
    const handle = this.#handle;
    this.#handle = undefined;
    handle?.close();
    await remove(this.#path);
  }
}
