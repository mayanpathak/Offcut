import { describe, expect, it } from "vitest";

import type { CloudWord } from "../../gen/api";
import { CLOUD_OPUS_CONFIG, dropSilentWords, type EncoderDeps, encodeOpus, preSkipOf } from "./cloud-audio";
import { DEFAULT_PRE_SKIP } from "./ogg-opus";

const RATE = 16_000;

/** `seconds` of a tone at `dbfs` (an RMS level), at 16 kHz. */
function tone(seconds: number, dbfs: number): Float32Array {
  const amplitude = Math.SQRT2 * 10 ** (dbfs / 20);
  return Float32Array.from({ length: Math.round(seconds * RATE) }, (_, i) => amplitude * Math.sin((2 * Math.PI * 220 * i) / RATE));
}

function join(...parts: Float32Array[]): Float32Array {
  const out = new Float32Array(parts.reduce((n, p) => n + p.length, 0));
  let at = 0;
  for (const p of parts) {
    out.set(p, at);
    at += p.length;
  }
  return out;
}

function word(text: string, startMs: number, endMs: number): CloudWord {
  return { text, start_ms: startMs, end_ms: endMs };
}

function opusHeadBytes(preSkip: number): Uint8Array {
  const head = new Uint8Array(19);
  head.set(Array.from("OpusHead", (c) => c.charCodeAt(0)));
  head[8] = 1;
  head[9] = 1;
  new DataView(head.buffer).setUint16(10, preSkip, true);
  return head;
}

/** An encoder that gives one 20 ms packet of 4 bytes per 320 input samples, and its description on the first output. */
function fakeEncoder(options: { supported?: boolean; description?: Uint8Array; fail?: boolean } = {}): EncoderDeps & { configured: AudioEncoderConfig[]; frames: number[] } {
  const configured: AudioEncoderConfig[] = [];
  const frames: number[] = [];
  return {
    configured,
    frames,
    isConfigSupported: (config) => Promise.resolve({ supported: options.supported ?? true, config }),
    createEncoder: (init) => {
      let pending = 0;
      let sent = 0;
      const emit = (): void => {
        const chunk = { byteLength: 4, duration: 20_000, copyTo: (dest: Uint8Array) => dest.set([sent & 0xff, 1, 2, 3]) } as unknown as EncodedAudioChunk;
        const meta = sent === 0 && options.description !== undefined ? { decoderConfig: { codec: "opus", sampleRate: RATE, numberOfChannels: 1, description: options.description } } : undefined;
        init.output(chunk, meta);
        sent += 1;
      };
      return {
        configure: (config) => {
          configured.push(config);
        },
        encode: (data) => {
          frames.push(data.numberOfFrames);
          if (options.fail === true) {
            init.error(new DOMException("encode failed"));
          }
          pending += data.numberOfFrames;
          while (pending >= 320) {
            emit();
            pending -= 320;
          }
        },
        flush: () => {
          if (pending > 0) {
            emit();
            pending = 0;
          }
          return Promise.resolve();
        },
        close: () => undefined,
      };
    },
    createData: (init) => ({ numberOfFrames: init.numberOfFrames, close: () => undefined }) as unknown as AudioData,
    serial: () => 7,
  };
}

/** The last page's granule position. */
function lastGranule(ogg: Uint8Array): bigint {
  for (let at = ogg.length - 27; at >= 0; at -= 1) {
    if (ogg[at] === 0x4f && ogg[at + 1] === 0x67 && ogg[at + 2] === 0x67 && ogg[at + 3] === 0x53 && ((ogg[at + 5] ?? 0) & 0x04) !== 0) {
      return new DataView(ogg.buffer, ogg.byteOffset).getBigInt64(at + 6, true);
    }
  }
  throw new Error("no EOS page");
}

describe("encodeOpus", () => {
  it("feeds one-second frames, asks for TE-C1's config, and trims the end to the clip", async () => {
    const pcm = tone(2.5, -20);
    const deps = fakeEncoder({ description: opusHeadBytes(104) });
    const result = await encodeOpus(pcm, deps);
    if (!("ogg" in result)) {
      throw new Error("unsupported");
    }
    expect(deps.configured).toEqual([CLOUD_OPUS_CONFIG]);
    expect(deps.frames).toEqual([16_000, 16_000, 8_000]);
    expect(result.durationMs).toBe(2_500);
    expect(lastGranule(result.ogg)).toBe(BigInt(104 + 2.5 * 48_000));
    expect(String.fromCharCode(...result.ogg.subarray(0, 4))).toBe("OggS");
  });

  it("takes the pre-skip from the description, and the default without one", async () => {
    const without = await encodeOpus(tone(1, -20), fakeEncoder());
    if (!("ogg" in without)) {
      throw new Error("unsupported");
    }
    expect(lastGranule(without.ogg)).toBe(BigInt(DEFAULT_PRE_SKIP + 48_000));
    expect(preSkipOf(opusHeadBytes(104))).toBe(104);
    expect(preSkipOf(new Uint8Array(19))).toBeUndefined();
    expect(preSkipOf(undefined)).toBeUndefined();
  });

  it("answers unsupported when the browser says so, and throws when the encoder fails", async () => {
    expect(await encodeOpus(tone(1, -20), fakeEncoder({ supported: false }))).toEqual({ unsupported: true });
    await expect(encodeOpus(tone(1, -20), fakeEncoder({ fail: true }))).rejects.toThrow();
  });
});

describe("dropSilentWords", () => {
  // Speech at the reference clip's levels (TE-C2): a median of -26 dBFS, the quietest word -37.
  const clip = join(tone(1, -26), tone(1, -37), new Float32Array(RATE));

  it("keeps a quiet real word and drops a word over silence", () => {
    const words = [word("Loud", 100, 900), word("quiet.", 1_100, 1_900), word("Thank", 2_100, 2_600)];
    expect(dropSilentWords(words, clip).map((w) => w.text)).toEqual(["Loud", "quiet."]);
  });

  it("drops a word that starts at or after the end, and judges one that straddles it on the part inside", () => {
    const words = [word("end.", 2_950, 3_200), word("after", 3_000, 3_300), word("Last", 1_900, 2_100)];
    // "end." starts over silence; "Last" is half over the quiet word, enough to keep.
    expect(dropSilentWords(words, clip).map((w) => w.text)).toEqual(["Last"]);
  });

  it("measures a word too short to measure over 40 ms", () => {
    expect(dropSilentWords([word("a", 500, 500)], clip).map((w) => w.text)).toEqual(["a"]);
    expect(dropSilentWords([word("a", 2_500, 2_500)], clip)).toEqual([]);
  });

  it("drops everything in digital silence and keeps everything in a steady clip", () => {
    expect(dropSilentWords([word("x", 0, 500)], new Float32Array(RATE))).toEqual([]);
    const steady = tone(2, -30);
    const words = [word("one", 0, 500), word("two", 600, 1_000), word("three.", 1_200, 1_900)];
    expect(dropSilentWords(words, steady)).toEqual(words);
  });
});
