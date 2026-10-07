// The asset host: model files, the demo video, the sample clip (TS §24.1).
// This is the only file that may `fetch` from it. V1 has no such request yet:
// the demo video is loaded by a `<video>` element from `assetUrl(...)`.

import { env } from "../config/env";

/**
 * The demo video of the landing page (E-1), relative to the asset host's
 * base URL. The file is uploaded in the deploy phase; until then this path
 * does not load.
 */
export const DEMO_VIDEO_PATH = "media/demo.mp4";

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
