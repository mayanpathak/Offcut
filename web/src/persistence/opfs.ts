// The origin private file system: where each file lives, and the few
// operations the app does on it. This is the only file that builds an OPFS
// path (TS §23.1). A path is made of ids and fixed words; no path ever
// contains a name the user supplied (TS §25.1 P-11).

import type { Bytes, ClipId, ExportId } from "../gen/domain";

/** Why a storage operation failed: the disk is full, or anything else. */
export class OpfsError extends Error {
  readonly kind: "quota" | "io";

  constructor(kind: "quota" | "io", cause: unknown) {
    super(kind === "quota" ? "storage quota exceeded" : "storage operation failed", { cause });
    this.name = "OpfsError";
    this.kind = kind;
  }
}

/** The one place of this file that turns a number into `Bytes` (D-59). */
function toBytes(n: number): Bytes {
  return n as Bytes;
}

/** A model file name is one path segment: it may not leave its directory. */
function modelName(name: string): string {
  if (name === "" || name.includes("/") || name.includes("..")) {
    throw new Error("a model file name is one path segment");
  }
  return name;
}

export const paths = {
  clipsRoot: (): string => "clips",
  clipDir: (id: ClipId): string => `clips/${id}`,
  clipSource: (id: ClipId): string => `clips/${id}/source`,
  clipOut48: (id: ClipId): string => `clips/${id}/out48.f32`,
  exportsRoot: (): string => "exports",
  exportTmp: (id: ExportId): string => `exports/tmp/${id}.mp4`,
  exportFinal: (id: ExportId): string => `exports/${id}.mp4`,
  modelDir: (modelId: string): string => `models/${modelId}`,
  modelPart: (modelId: string, name: string): string => `models/${modelId}/${modelName(name)}.part`,
  modelFile: (modelId: string, name: string): string => `models/${modelId}/${modelName(name)}`,
};

function isNamed(error: unknown, name: string): boolean {
  return error instanceof DOMException && error.name === name;
}

/** Runs one storage operation; a failure is rethrown tagged `quota` or `io`. */
async function guarded<T>(operation: () => Promise<T>): Promise<T> {
  try {
    return await operation();
  } catch (error) {
    if (error instanceof OpfsError) {
      throw error;
    }
    throw new OpfsError(isNamed(error, "QuotaExceededError") ? "quota" : "io", error);
  }
}

/** The directory at `segments`. `undefined` when one of them is absent and may not be made. */
async function directory(segments: readonly string[], create: boolean): Promise<FileSystemDirectoryHandle | undefined> {
  let dir = await navigator.storage.getDirectory();
  for (const segment of segments) {
    try {
      dir = await dir.getDirectoryHandle(segment, { create });
    } catch (error) {
      if (!create && isNamed(error, "NotFoundError")) {
        return undefined;
      }
      throw error;
    }
  }
  return dir;
}

/** The directory that holds `path`, and the name of `path` in it. */
async function locate(
  path: string,
  create: boolean,
): Promise<{ dir: FileSystemDirectoryHandle; name: string } | undefined> {
  const segments = path.split("/");
  const name = segments.pop();
  if (name === undefined || name === "") {
    throw new Error("an OPFS path names a file or a directory");
  }
  const dir = await directory(segments, create);
  return dir === undefined ? undefined : { dir, name };
}

/** The handle of a file that exists, or `undefined`. */
async function existing(path: string): Promise<FileSystemFileHandle | undefined> {
  const at = await locate(path, false);
  if (at === undefined) {
    return undefined;
  }
  try {
    return await at.dir.getFileHandle(at.name);
  } catch (error) {
    if (isNamed(error, "NotFoundError")) {
      return undefined;
    }
    throw error;
  }
}

/** With `create`, the file and the directories above it are made when absent. */
export function fileHandle(path: string, o: { create: boolean }): Promise<FileSystemFileHandle> {
  return guarded(async () => {
    const at = await locate(path, o.create);
    if (at === undefined) {
      throw new DOMException("no such file", "NotFoundError");
    }
    return at.dir.getFileHandle(at.name, { create: o.create });
  });
}

export function getFile(path: string): Promise<File> {
  return guarded(async () => (await fileHandle(path, { create: false })).getFile());
}

/** `null` when the file is absent. */
export function size(path: string): Promise<Bytes | null> {
  return guarded(async () => {
    const handle = await existing(path);
    return handle === undefined ? null : toBytes((await handle.getFile()).size);
  });
}

/** Writes at an offset and closes the file, so the bytes survive a closed tab (D-36). */
export function writeAt(path: string, offset: Bytes, data: Uint8Array<ArrayBuffer>): Promise<void> {
  return guarded(async () => {
    const handle = await fileHandle(path, { create: true });
    const writable = await handle.createWritable({ keepExistingData: true });
    try {
      await writable.write({ type: "write", position: offset, data });
    } finally {
      await writable.close();
    }
  });
}

export function truncate(path: string, length: Bytes): Promise<void> {
  return guarded(async () => {
    const handle = await fileHandle(path, { create: true });
    const writable = await handle.createWritable({ keepExistingData: true });
    try {
      await writable.truncate(length);
    } finally {
      await writable.close();
    }
  });
}

// Chrome moves a file of the origin private file system in place. The
// method is not in the TypeScript library yet.
type Movable = FileSystemFileHandle & { move(dir: FileSystemDirectoryHandle, name: string): Promise<void> };

/** Renames a file, replacing one already at `to`. */
export function move(from: string, to: string): Promise<void> {
  return guarded(async () => {
    const source: Partial<Movable> = await fileHandle(from, { create: false });
    const target = await locate(to, true);
    if (source.move === undefined || target === undefined) {
      throw new Error("this browser cannot move a file in OPFS");
    }
    await source.move(target.dir, target.name);
  });
}

/** A file, or a directory with everything in it. An absent path is not an error. */
export function remove(path: string): Promise<void> {
  return guarded(async () => {
    const at = await locate(path, false);
    if (at === undefined) {
      return;
    }
    try {
      await at.dir.removeEntry(at.name, { recursive: true });
    } catch (error) {
      if (!isNamed(error, "NotFoundError")) {
        throw error;
      }
    }
  });
}

/** The names in a directory. An absent directory gives `[]`. */
export function list(dir: string): Promise<readonly string[]> {
  return guarded(async () => {
    const handle = await directory(dir.split("/"), false);
    if (handle === undefined) {
      return [];
    }
    const names: string[] = [];
    for await (const name of handle.keys()) {
      names.push(name);
    }
    return names;
  });
}

/** Raw little-endian 32-bit floats, 48 kHz mono. */
export function writeAudio(id: ClipId, out48: Float32Array<ArrayBuffer>): Promise<void> {
  return guarded(async () => {
    const handle = await fileHandle(paths.clipOut48(id), { create: true });
    const writable = await handle.createWritable();
    try {
      await writable.write(out48);
    } finally {
      await writable.close();
    }
  });
}

/** What `writeAudio` wrote (D-51). */
export function readAudio(id: ClipId): Promise<Float32Array<ArrayBuffer>> {
  return guarded(async () => new Float32Array(await (await getFile(paths.clipOut48(id))).arrayBuffer()));
}
