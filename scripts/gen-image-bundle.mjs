// The image bundle (v6.2implementation D-470, D-471, D-495; V6.2 Prompt 195).
// For each name of `EmojiId` and `IconId` (crates/offcut-types/src/assets.rs,
// whose variant docs name the upstream files), fetches the upstream file at
// a pinned version and writes a 160 x 160 PNG, 8-bit RGBA, straight alpha;
// then `manifest.json` and the upstream licence texts in `LICENSES/`.
//
//   node scripts/gen-image-bundle.mjs              writes crates/offcut-scene/assets/images/
//   node scripts/gen-image-bundle.mjs --out <dir>  writes <dir> instead (a dry run, or the determinism check)
//   --size <px>                                    another size than 160 (D-471's fallback is 128)
//
// Emoji: Fluent Emoji's "3D" PNGs (256 px) at commit FLUENT_COMMIT, resized
// with Lanczos-3 on premultiplied colour, here in Node. Icons: lucide-static's
// SVGs at LUCIDE_VERSION, drawn white on transparent by the installed Chrome
// (Playwright, channel "chrome", network blocked), on Lucide's own 24-unit
// grid, which has 1 unit of even padding on every side. Every PNG is encoded
// here, so the bytes depend on this script and Node's zlib, not on a browser's
// encoder. Downloads are kept in a fresh temporary folder, never in the
// repository. Nothing is sent anywhere but the two GETs per file.
//
// A name never changes here (D-495): to swap an image, change the variant's
// doc in assets.rs to another upstream file, run this again, and the manifest
// records the new source.

import { createHash } from "node:crypto";
import { mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync, existsSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { constants, deflateSync, inflateSync } from "node:zlib";

const ROOT = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const ASSETS_RS = join(ROOT, "crates/offcut-types/src/assets.rs");
const DEFAULT_OUT = join(ROOT, "crates/offcut-scene/assets/images");

const DEFAULT_SIZE = 160;
/** The icon keys start here, so that each set can grow without a clash (images.rs, ICON_BASE). */
const ICON_BASE = 1000;
const FLUENT_COMMIT = "1ffb34c752ecf5d402f04cfb4b392c77f57c54bc";
const FLUENT_RAW = `https://raw.githubusercontent.com/microsoft/fluentui-emoji/${FLUENT_COMMIT}`;
const LUCIDE_VERSION = "1.61.0";
const LUCIDE_PKG = `https://unpkg.com/lucide-static@${LUCIDE_VERSION}`;

const LICENCES = [
  { file: "LICENSES/fluent-emoji-MIT.txt", url: `${FLUENT_RAW}/LICENSE` },
  { file: "LICENSES/lucide-ISC.txt", url: `${LUCIDE_PKG}/LICENSE` },
];

// --- The names ------------------------------------------------------------------

/** The two sets of assets.rs, in code order: { set, code, name, upstream }. */
function readNames() {
  const text = readFileSync(ASSETS_RS, "utf8").replaceAll("\r\n", "\n");
  const sets = { EmojiId: "emoji", IconId: "icons" };
  const out = [];
  const block = /^\s*(EmojiId|IconId), (\d+), \[\n([\s\S]*?)^\]\);/gm;
  let found = 0;
  for (let b = block.exec(text); b; b = block.exec(text)) {
    found += 1;
    const entry = /^\s*\w+ = "([a-z0-9_]+)", "(Fluent|Lucide): ([^"]+)";$/gm;
    const entries = [...b[3].matchAll(entry)];
    if (entries.length !== Number(b[2])) {
      throw new Error(`${b[1]}: ${entries.length} entries read, ${b[2]} declared`);
    }
    entries.forEach((e, code) => out.push({ set: sets[b[1]], code, name: e[1], kind: e[2], upstream: e[3] }));
  }
  if (found !== 2) throw new Error(`assets.rs: ${found} closed sets read, 2 expected`);
  return out;
}

/** The pinned URL of an entry's upstream file. */
function sourceOf(entry) {
  if (entry.kind === "Lucide") return `${LUCIDE_PKG}/icons/${entry.upstream}.svg`;
  const tone = entry.upstream.endsWith(" (Default tone)");
  const folder = tone ? entry.upstream.slice(0, -" (Default tone)".length) : entry.upstream;
  const file = folder.toLowerCase().replaceAll(" ", "_");
  const path = tone ? `assets/${folder}/Default/3D/${file}_3d_default.png` : `assets/${folder}/3D/${file}_3d.png`;
  return `${FLUENT_RAW}/${encodeURI(path)}`;
}

// --- PNG ------------------------------------------------------------------------

const SIGNATURE = Buffer.from([137, 80, 78, 71, 13, 10, 26, 10]);

const CRC_TABLE = Array.from({ length: 256 }, (_, n) => {
  let c = n;
  for (let k = 0; k < 8; k += 1) c = c & 1 ? 0xedb88320 ^ (c >>> 1) : c >>> 1;
  return c >>> 0;
});
function crc32(bytes) {
  let c = 0xffffffff;
  for (const byte of bytes) c = CRC_TABLE[(c ^ byte) & 0xff] ^ (c >>> 8);
  return (c ^ 0xffffffff) >>> 0;
}

function paeth(a, b, c) {
  const p = a + b - c;
  const pa = Math.abs(p - a);
  const pb = Math.abs(p - b);
  const pc = Math.abs(p - c);
  return pa <= pb && pa <= pc ? a : pb <= pc ? b : c;
}

/** An 8-bit, non-interlaced PNG of colour type 2, 3 or 6 as RGBA: { width, height, rgba }. */
function decodePng(file) {
  if (!file.subarray(0, 8).equals(SIGNATURE)) throw new Error("not a PNG");
  let pos = 8;
  let header;
  let palette;
  let trns;
  const idat = [];
  while (pos < file.length) {
    const length = file.readUInt32BE(pos);
    const type = file.toString("latin1", pos + 4, pos + 8);
    const data = file.subarray(pos + 8, pos + 8 + length);
    if (type === "IHDR") header = { width: data.readUInt32BE(0), height: data.readUInt32BE(4), depth: data[8], colour: data[9], interlace: data[12] };
    if (type === "PLTE") palette = data;
    if (type === "tRNS") trns = data;
    if (type === "IDAT") idat.push(data);
    pos += 12 + length;
  }
  const channels = { 2: 3, 3: 1, 6: 4 }[header.colour];
  if (header.depth !== 8 || header.interlace !== 0 || !channels) {
    throw new Error(`unsupported PNG: depth ${header.depth}, colour ${header.colour}, interlace ${header.interlace}`);
  }
  const { width, height } = header;
  const raw = inflateSync(Buffer.concat(idat));
  const stride = width * channels;
  const pixels = Buffer.alloc(stride * height);
  for (let y = 0; y < height; y += 1) {
    const filter = raw[y * (stride + 1)];
    for (let x = 0; x < stride; x += 1) {
      const v = raw[y * (stride + 1) + 1 + x];
      const a = x >= channels ? pixels[y * stride + x - channels] : 0;
      const b = y > 0 ? pixels[(y - 1) * stride + x] : 0;
      const c = x >= channels && y > 0 ? pixels[(y - 1) * stride + x - channels] : 0;
      const predictor = [0, a, b, (a + b) >> 1, paeth(a, b, c)][filter];
      if (predictor === undefined) throw new Error(`bad PNG filter ${filter}`);
      pixels[y * stride + x] = (v + predictor) & 0xff;
    }
  }
  const rgba = Buffer.alloc(width * height * 4);
  for (let i = 0; i < width * height; i += 1) {
    if (channels === 4) pixels.copy(rgba, i * 4, i * 4, i * 4 + 4);
    else if (channels === 3) rgba.set([pixels[i * 3], pixels[i * 3 + 1], pixels[i * 3 + 2], 255], i * 4);
    else rgba.set([...palette.subarray(pixels[i] * 3, pixels[i] * 3 + 3), trns && pixels[i] < trns.length ? trns[pixels[i]] : 255], i * 4);
  }
  return { width, height, rgba };
}

/** One filter type applied to every row, or `null` for the per-row choice (least sum of absolute differences). */
function filterRows(rgba, width, height, fixed) {
  const stride = width * 4;
  const out = Buffer.alloc((stride + 1) * height);
  const row = Buffer.alloc(stride);
  for (let y = 0; y < height; y += 1) {
    let best = null;
    for (const filter of fixed === null ? [0, 1, 2, 3, 4] : [fixed]) {
      let cost = 0;
      for (let x = 0; x < stride; x += 1) {
        const v = rgba[y * stride + x];
        const a = x >= 4 ? rgba[y * stride + x - 4] : 0;
        const b = y > 0 ? rgba[(y - 1) * stride + x] : 0;
        const c = x >= 4 && y > 0 ? rgba[(y - 1) * stride + x - 4] : 0;
        const r = (v - [0, a, b, (a + b) >> 1, paeth(a, b, c)][filter]) & 0xff;
        row[x] = r;
        cost += r < 128 ? r : 256 - r;
      }
      if (best === null || cost < best.cost) best = { cost, filter, bytes: Buffer.from(row) };
    }
    out[y * (stride + 1)] = best.filter;
    best.bytes.copy(out, y * (stride + 1) + 1);
  }
  return out;
}

function chunk(type, data) {
  const head = Buffer.alloc(8);
  head.writeUInt32BE(data.length, 0);
  head.write(type, 4, "latin1");
  const crc = Buffer.alloc(4);
  crc.writeUInt32BE(crc32(Buffer.concat([head.subarray(4), data])), 0);
  return Buffer.concat([head, data, crc]);
}

/** RGBA (straight alpha) as the smallest PNG of a few deterministic settings: IHDR, one IDAT, IEND, nothing else. */
function encodePng(rgba, width, height) {
  const ihdr = Buffer.alloc(13);
  ihdr.writeUInt32BE(width, 0);
  ihdr.writeUInt32BE(height, 4);
  ihdr.set([8, 6, 0, 0, 0], 8);
  let idat = null;
  for (const fixed of [null, 0, 1, 2, 4]) {
    const filtered = filterRows(rgba, width, height, fixed);
    for (const strategy of [constants.Z_DEFAULT_STRATEGY, constants.Z_FILTERED]) {
      const z = deflateSync(filtered, { level: 9, memLevel: 9, strategy });
      if (idat === null || z.length < idat.length) idat = z;
    }
  }
  return Buffer.concat([SIGNATURE, chunk("IHDR", ihdr), chunk("IDAT", idat), chunk("IEND", Buffer.alloc(0))]);
}

// --- Resizing (emoji) -----------------------------------------------------------

function lanczos3(x) {
  if (x === 0) return 1;
  if (Math.abs(x) >= 3) return 0;
  const px = Math.PI * x;
  return (3 * Math.sin(px) * Math.sin(px / 3)) / (px * px);
}

/** For each output index, the source indices and normalised weights of a Lanczos-3 resize of n to m. */
function weights(n, m) {
  const scale = m / n;
  const support = 3 / Math.min(scale, 1);
  return Array.from({ length: m }, (_, i) => {
    const centre = (i + 0.5) / scale - 0.5;
    const taps = [];
    for (let j = Math.ceil(centre - support); j <= Math.floor(centre + support); j += 1) {
      const w = lanczos3((j - centre) * Math.min(scale, 1));
      if (w !== 0) taps.push([Math.min(Math.max(j, 0), n - 1), w]);
    }
    const sum = taps.reduce((s, [, w]) => s + w, 0);
    return taps.map(([j, w]) => [j, w / sum]);
  });
}

/** A square RGBA image resized to `size`, filtered on premultiplied colour, returned with straight alpha. */
function resize({ width, height, rgba }, size) {
  if (width !== height) throw new Error(`not square: ${width} x ${height}`);
  const pre = new Float64Array(width * height * 4);
  for (let i = 0; i < width * height; i += 1) {
    const a = rgba[i * 4 + 3] / 255;
    pre.set([rgba[i * 4] * a, rgba[i * 4 + 1] * a, rgba[i * 4 + 2] * a, rgba[i * 4 + 3]], i * 4);
  }
  const w = weights(width, size);
  const across = new Float64Array(size * height * 4);
  for (let y = 0; y < height; y += 1) {
    for (let x = 0; x < size; x += 1) {
      for (const [j, k] of w[x]) {
        for (let c = 0; c < 4; c += 1) across[(y * size + x) * 4 + c] += pre[(y * width + j) * 4 + c] * k;
      }
    }
  }
  const out = Buffer.alloc(size * size * 4);
  for (let y = 0; y < size; y += 1) {
    for (let x = 0; x < size; x += 1) {
      const p = [0, 0, 0, 0];
      for (const [j, k] of w[y]) {
        for (let c = 0; c < 4; c += 1) p[c] += across[(j * size + x) * 4 + c] * k;
      }
      const alpha = Math.min(Math.max(p[3], 0), 255);
      const a = Math.round(alpha);
      const o = (y * size + x) * 4;
      out[o + 3] = a;
      for (let c = 0; c < 3 && a > 0; c += 1) {
        out[o + c] = Math.round(Math.min(Math.max(p[c] / (alpha / 255), 0), 255));
      }
    }
  }
  return out;
}

// --- Drawing (icons) ------------------------------------------------------------

/** The SVG drawn white at `size` px: Lucide's stroke, caps and joins unchanged. */
function whiteSvg(svg, size) {
  const swaps = [
    ['width="24"', `width="${size}"`],
    ['height="24"', `height="${size}"`],
    ['stroke="currentColor"', 'stroke="#ffffff"'],
  ];
  return swaps.reduce((text, [from, to]) => {
    if (text.split(from).length !== 2) throw new Error(`the SVG does not have ${from} exactly once`);
    return text.replace(from, to);
  }, svg);
}

async function drawIcons(svgs, size) {
  const { chromium } = await import("@playwright/test");
  const browser = await chromium.launch({ channel: "chrome", args: ["--disable-gpu", "--force-color-profile=srgb"] });
  try {
    const page = await browser.newPage();
    await page.route("**/*", (route) => route.abort());
    await page.setContent("<!doctype html><title>icons</title>");
    const drawn = [];
    for (const svg of svgs) {
      const base64 = await page.evaluate(
        async ({ svg, size }) => {
          const img = new Image();
          img.src = `data:image/svg+xml;charset=utf-8,${encodeURIComponent(svg)}`;
          await img.decode();
          const canvas = new OffscreenCanvas(size, size);
          const g = canvas.getContext("2d", { willReadFrequently: true });
          g.drawImage(img, 0, 0, size, size);
          const data = g.getImageData(0, 0, size, size).data;
          let s = "";
          for (const byte of data) s += String.fromCharCode(byte);
          return btoa(s);
        },
        { svg: whiteSvg(svg, size), size },
      );
      drawn.push(Buffer.from(base64, "base64"));
    }
    return drawn;
  } finally {
    await browser.close();
  }
}

// --- The run --------------------------------------------------------------------

async function fetchBytes(url) {
  const response = await fetch(url);
  if (!response.ok) throw new Error(`${url}: HTTP ${response.status}`);
  return Buffer.from(await response.arrayBuffer());
}

const sha256 = (bytes) => createHash("sha256").update(bytes).digest("hex");

async function main() {
  const args = process.argv.slice(2);
  const outArg = args.indexOf("--out");
  const out = outArg >= 0 ? resolve(args[outArg + 1]) : DEFAULT_OUT;
  const sizeArg = args.indexOf("--size");
  const SIZE = sizeArg >= 0 ? Number(args[sizeArg + 1]) : DEFAULT_SIZE;
  if (!Number.isInteger(SIZE) || SIZE < 16 || SIZE > 256) throw new Error("--size: a whole number of pixels, 16 to 256");
  const downloads = mkdtempSync(join(tmpdir(), "offcut-images-"));
  console.log(`downloads: ${downloads}`);

  const entries = readNames().map((e) => ({ ...e, source: sourceOf(e) }));
  for (const e of entries) {
    e.upstreamBytes = await fetchBytes(e.source);
    writeFileSync(join(downloads, `${e.set}-${e.name}${e.kind === "Lucide" ? ".svg" : ".png"}`), e.upstreamBytes);
  }

  // A file fetched again must be the file fetched before, unless its source changed (a swap).
  const previousPath = join(out, "manifest.json");
  if (existsSync(previousPath)) {
    const previous = JSON.parse(readFileSync(previousPath, "utf8")).images;
    for (const e of entries) {
      const before = previous.find((p) => p.set === e.set && p.id === e.name);
      if (before && before.source === e.source && before.upstream_sha256 !== sha256(e.upstreamBytes)) {
        throw new Error(`${e.source}: the upstream file changed at a pinned version`);
      }
    }
  }

  const icons = entries.filter((e) => e.kind === "Lucide");
  const drawn = await drawIcons(icons.map((e) => e.upstreamBytes.toString("utf8")), SIZE);
  icons.forEach((e, i) => (e.rgba = drawn[i]));
  for (const e of entries.filter((x) => x.kind === "Fluent")) {
    const image = decodePng(e.upstreamBytes);
    if (image.width !== 256 || image.height !== 256) throw new Error(`${e.source}: ${image.width} x ${image.height}, not 256 px`);
    e.rgba = resize(image, SIZE);
  }

  rmSync(join(out, "emoji"), { recursive: true, force: true });
  rmSync(join(out, "icons"), { recursive: true, force: true });
  for (const dir of ["emoji", "icons", "LICENSES"]) mkdirSync(join(out, dir), { recursive: true });
  const images = entries.map((e) => {
    const png = encodePng(e.rgba, SIZE, SIZE);
    const file = `${e.set}/${e.name}.png`;
    writeFileSync(join(out, file), png);
    const fluent = e.kind === "Fluent";
    return {
      set: e.set,
      id: e.name,
      code: e.code,
      key: fluent ? e.code : ICON_BASE + e.code,
      file,
      bytes: png.length,
      sha256: sha256(png),
      source: e.source,
      upstream_sha256: sha256(e.upstreamBytes),
      licence: fluent ? "MIT" : "ISC",
      licence_file: LICENCES[fluent ? 0 : 1].file,
    };
  });
  for (const { file, url } of LICENCES) writeFileSync(join(out, file), await fetchBytes(url));

  const manifest = {
    generator: "scripts/gen-image-bundle.mjs",
    size: SIZE,
    format: "PNG, 8-bit RGBA, straight alpha",
    images,
  };
  writeFileSync(previousPath, `${JSON.stringify(manifest, null, 2)}\n`);

  const total = (set) => images.filter((i) => set === undefined || i.set === set).reduce((s, i) => s + i.bytes, 0);
  console.log(`images: ${images.length} (${total("emoji")} bytes of emoji, ${total("icons")} of icons, ${total()} in all)`);
  console.log(`written: ${out}`);
}

main().catch((error) => {
  console.error(`gen-image-bundle: ${error.message}`);
  process.exit(1);
});
