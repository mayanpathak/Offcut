// The asset host: model files, the demo video, the sample clip (TS §24.1).
// This is the only file that may `fetch` from it. The demo video is not
// fetched here: a `<video>` element loads it from `assetUrl(...)`.

import { env } from "../config/env";
import type { Bytes } from "../gen/domain";

/**
 * The demo video of the landing page (E-1), relative to the asset host's
 * base URL, as `scripts/upload-assets.sh` printed it. The name holds a hash
 * of the content: a new video is a new upload and a new path here.
 */
export const DEMO_VIDEO_PATH = "media/demo.81cafa494d9c2b68.mp4";

/**
 * The sample clip that the landing page offers in place of a dropped file,
 * as `scripts/upload-assets.sh` printed it (v2implementation D-39).
 */
export const SAMPLE_CLIP_PATH = "media/speech_scriptA_landscape_720p.7da948cecc39438a.mp4";

/**
 * The URL of a file on the asset host. `path` is relative to its base URL.
 * A request to the asset host carries no query string, and a path may not
 * climb out of the base.
 */
export function assetUrl(path: string): string {
  if (path.includes("?") || path.includes("..")) {
    throw new Error('An asset path may not contain "?" or "..".');
  }
  return `${env.assetBaseUrl}/${path.replace(/^\/+/, "")}`;
}

/** What a request to the asset host gave. The body of a response is not read here. */
export type AssetResult =
  | { ok: true; status: 200 | 206; response: Response }
  | { ok: false; cause: "offline" | "status" | "aborted"; status?: number };

/**
 * One `GET` to the asset host, for the whole file or for the bytes of
 * `range`. It carries no credential, no cookie and no query string, and
 * `Range` is its only header (TS §24.1). It is made once: the caller knows
 * whether to try again, and what a failure means where it happened.
 */
export async function fetchAsset(
  path: string,
  o: { range?: { start: Bytes; endInclusive: Bytes }; signal: AbortSignal },
): Promise<AssetResult> {
  if (o.signal.aborted) {
    return { ok: false, cause: "aborted" };
  }
  if (!navigator.onLine) {
    return { ok: false, cause: "offline" };
  }
  const init: RequestInit = {
    method: "GET",
    mode: "cors",
    credentials: "omit",
    cache: "no-store",
    signal: o.signal,
  };
  if (o.range !== undefined) {
    init.headers = { Range: `bytes=${String(o.range.start)}-${String(o.range.endInclusive)}` };
  }

  let response: Response;
  try {
    response = await fetch(assetUrl(path), init);
  } catch (error) {
    if (o.signal.aborted) {
      return { ok: false, cause: "aborted" };
    }
    // A `TypeError` is how `fetch` reports that the network cannot be reached.
    if (error instanceof TypeError) {
      return { ok: false, cause: "offline" };
    }
    throw error;
  }

  const status = response.status;
  if (status === 200 || status === 206) {
    return { ok: true, status, response };
  }
  return { ok: false, cause: "status", status };
}
