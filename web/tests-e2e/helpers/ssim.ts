// The structural similarity of the golden frames, for a frame the browser
// drew (v6.2implementation D-474, TE-30): a line-for-line copy of
// `crates/offcut-render/tests/common/ssim.rs`, so the browser's frame is
// judged by the same measure as the native one. On BT.709 luma, over the
// 8 x 8 windows that tile the image, with the standard constants, the mean
// over windows. Change one, change both.

/** An image in memory, as `offcut_render::RgbaImage`: rows from the top-left, four bytes a pixel. */
export type Rgba = { width: number; height: number; data: Uint8Array };

/** The side of a window, in pixels. */
const WINDOW = 8;
/** `(0.01 x 255)²` and `(0.03 x 255)²`. */
const C1 = 0.01 * 255.0 * (0.01 * 255.0);
const C2 = 0.03 * 255.0 * (0.03 * 255.0);

/** The SSIM of `a` and `b`, which are the same size: 1.0 for equal images. Pixels past the last whole window are not compared. */
export function ssim(a: Rgba, b: Rgba): number {
  if (a.width !== b.width || a.height !== b.height) {
    throw new Error(`two sizes: ${String(a.width)} x ${String(a.height)} and ${String(b.width)} x ${String(b.height)}`);
  }
  const columns = Math.floor(a.width / WINDOW);
  const rows = Math.floor(a.height / WINDOW);
  if (columns === 0 || rows === 0) {
    throw new Error("an image smaller than a window");
  }
  const y = (image: Rgba, x: number, row: number): number => {
    const at = (row * image.width + x) * 4;
    return 0.2126 * (image.data[at] ?? 0) + 0.7152 * (image.data[at + 1] ?? 0) + 0.0722 * (image.data[at + 2] ?? 0);
  };
  const n = WINDOW * WINDOW;
  let total = 0.0;
  for (let row = 0; row < rows; row += 1) {
    for (let column = 0; column < columns; column += 1) {
      let [sa, sb, saa, sbb, sab] = [0.0, 0.0, 0.0, 0.0, 0.0];
      for (let dy = 0; dy < WINDOW; dy += 1) {
        for (let dx = 0; dx < WINDOW; dx += 1) {
          const [px, py] = [column * WINDOW + dx, row * WINDOW + dy];
          const [va, vb] = [y(a, px, py), y(b, px, py)];
          sa += va;
          sb += vb;
          saa += va * va;
          sbb += vb * vb;
          sab += va * vb;
        }
      }
      const [ma, mb] = [sa / n, sb / n];
      const [va, vb] = [saa / n - ma * ma, sbb / n - mb * mb];
      const cov = sab / n - ma * mb;
      total += ((2.0 * ma * mb + C1) * (2.0 * cov + C2)) / ((ma * ma + mb * mb + C1) * (va + vb + C2));
    }
  }
  return total / (columns * rows);
}
