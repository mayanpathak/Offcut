// The image check of TE-30 (v6.2implementation D-474, §9.1 step 3), in a
// development build only: `render.worker.ts` serves it behind `DEV_FAULTS`,
// so a production build drops this file. The render bundle draws the frame
// of the `image_basic` golden, from the same definition the native golden
// asserts is its own (`offcut_render::check`), over a frame of the goldens'
// synthetic source; this reads the canvas back for the page to compare.

import { loadRender } from "../../wasm/load-render";
import type { ImageCheckResult } from "../protocol";

/** The source and the canvas of the check: `offcut_render::check::SOURCE` and `CANVAS`. */
const SOURCE = { width: 1080, height: 1920 };
const CANVAS = { width: 540, height: 960 };

export async function imageCheck(): Promise<ImageCheckResult> {
  const api = await loadRender();
  const frame = new VideoFrame(api.checkSource(), {
    format: "RGBA",
    codedWidth: SOURCE.width,
    codedHeight: SOURCE.height,
    timestamp: 0,
  });
  const canvas = new OffscreenCanvas(CANVAS.width, CANVAS.height);
  let check: Awaited<ReturnType<typeof api.drawImageCheck>> | undefined;
  try {
    check = await api.drawImageCheck(canvas, frame);
    const bitmap = canvas.transferToImageBitmap();
    const read = new OffscreenCanvas(CANVAS.width, CANVAS.height).getContext("2d", { willReadFrequently: true });
    if (read === null) {
      throw new Error("no 2D context to read the check back");
    }
    read.drawImage(bitmap, 0, 0);
    bitmap.close();
    const { data } = read.getImageData(0, 0, CANVAS.width, CANVAS.height);
    return { ...CANVAS, pixels: new Uint8Array(data.buffer), decodeMs: check.decode_ms() };
  } finally {
    frame.close();
    check?.free();
  }
}
