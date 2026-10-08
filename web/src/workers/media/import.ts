// Copies a dropped file into the origin private file system, where the
// demuxer can read it at any offset. The file's name is never read.

import type { Bytes, ClipId } from "../../gen/domain";
import { fileHandle, OpfsError, paths } from "../../persistence/opfs";
import { CANCELLED, type JobContext, WorkerFailure } from "../rpc";

/** One chunk is all of the file that is in memory at a time. */
const CHUNK_BYTES = 4 * 1024 * 1024;

/** A storage failure as the error a handler throws: the disk is full, or anything else (TS §8). */
function storageFailure(error: unknown): WorkerFailure {
  const quota =
    (error instanceof OpfsError && error.kind === "quota") ||
    (error instanceof DOMException && error.name === "QuotaExceededError");
  return new WorkerFailure(quota ? "E_STORAGE_QUOTA" : "E_STORAGE_IO", error instanceof Error ? error.name : "Error");
}

/** The synchronous handle of a clip's source file. The caller closes it. */
export async function openSource(clipId: ClipId, o: { create: boolean }): Promise<FileSystemSyncAccessHandle> {
  try {
    return await (await fileHandle(paths.clipSource(clipId), o)).createSyncAccessHandle();
  } catch (error) {
    throw storageFailure(error);
  }
}

/**
 * Writes the file to `clips/<clipId>/source`, byte for byte, and returns the
 * open handle. The cancel flag is read before each chunk; when it is set the
 * handle is closed and the sentinel returned. Progress is in bytes, once per
 * chunk.
 */
export async function importToOpfs(
  clipId: ClipId,
  file: File,
  isCancelled: () => boolean,
  progress?: JobContext["progress"],
): Promise<FileSystemSyncAccessHandle | typeof CANCELLED> {
  const handle = await openSource(clipId, { create: true });
  try {
    handle.truncate(0);
    for (let offset = 0; offset < file.size; offset += CHUNK_BYTES) {
      if (isCancelled()) {
        handle.close();
        return CANCELLED;
      }
      const chunk = new Uint8Array(await file.slice(offset, offset + CHUNK_BYTES).arrayBuffer());
      if (handle.write(chunk, { at: offset }) !== chunk.length) {
        throw new Error("short write");
      }
      progress?.({ stage: "import", done: offset + chunk.length, total: file.size });
    }
    handle.flush();
    return handle;
  } catch (error) {
    handle.close();
    throw storageFailure(error);
  }
}

/** The size of a file as the unit the demuxer takes. */
export function byteSize(handle: FileSystemSyncAccessHandle): Bytes {
  return handle.getSize() as Bytes;
}
