import { describe, expect, it } from "vitest";

import type { SourceInfo } from "../../gen/domain";
import { CANCELLED } from "../rpc-worker";
import {
  type DecodedAudio,
  type DecodingDeps,
  decodeAudioRegion,
  LevelHistogram,
  type PcmFile,
  type RegionDemuxer,
  type ResamplerFactory,
  SourceAudioError,
  writeWholePcm16,
} from "./audio-region";

const FRAMES = 1024;

/** The fake source's sound at global frame g (the same on both channels). */
function wave(g: number): number {
  return Math.round(Math.sin(g * 0.0031) * 0.6 * 1e6) / 1e6;
}

/** A demuxer of `packets` AAC packets at `rate`, the first presented at `firstUs`. */
function demuxer(packets: number, rate: number, firstUs = 0): RegionDemuxer {
  const time = (i: number) => firstUs + (i * FRAMES * 1e6) / rate;
  return {
    audioSampleCount: () => packets,
    audioSampleTimeUs: (i) => (i >= 0 && i < packets ? time(i) : undefined),
    audioDescription: () => new Uint8Array([0x11, 0x90]),
    readAudioSample: (i) => ({ data: new Uint8Array(new Uint32Array([i]).buffer), ptsUs: time(i), durationUs: (FRAMES * 1e6) / rate }),
  };
}

/** A decoder that gives packet i as 1024 frames of `wave`, at `rateOf(i)`. */
function decoding(rateOf: (i: number) => number, failAt = -1): DecodingDeps {
  return {
    decoder(output, error) {
      let closed = false;
      return {
        configure: () => undefined,
        decode(chunk) {
          const i = new Uint32Array(chunk.data.slice().buffer)[0] ?? 0;
          if (i === failAt) {
            error({ name: "EncodingError" });
            return;
          }
          const data: DecodedAudio = {
            sampleRate: rateOf(i),
            numberOfFrames: FRAMES,
            numberOfChannels: 2,
            copyTo(dest) {
              for (let k = 0; k < FRAMES; k += 1) {
                dest[k] = wave(i * FRAMES + k);
              }
            },
            close: () => undefined,
          };
          output(data);
        },
        flush: () => Promise.resolve(),
        close: () => {
          closed = true;
        },
        decodeQueueSize: 0,
        get state() {
          return closed ? "closed" : "configured";
        },
      };
    },
  };
}

/** Equal rates pass through; 48 → 16 kHz keeps every third input sample. Both keep `round(n × to ÷ from)`. */
const fakeResampler = (made: number[] = []): ResamplerFactory => (from, to) => {
  made.push(from);
  let seen = 0;
  let given = 0;
  return {
    push(input) {
      if (from === to) {
        seen += input.length;
        given += input.length;
        return Float32Array.from(input);
      }
      const out: number[] = [];
      for (const s of input) {
        if (seen % (from / to) === 0) {
          out.push(s);
        }
        seen += 1;
      }
      given += out.length;
      return Float32Array.from(out);
    },
    finish(total) {
      return new Float32Array(Math.max(0, Math.round((total * to) / from) - given));
    },
    free: () => undefined,
  };
};

/** An in-memory `pcm16.f32`. */
function memoryFile(): PcmFile & { samples(): Float32Array } {
  let data = new Float32Array(0);
  return {
    getSize: () => data.length * 4,
    truncate(bytes) {
      const next = new Float32Array(bytes / 4);
      next.set(data.subarray(0, next.length));
      data = next;
    },
    write(samples, at) {
      const end = at / 4 + samples.length;
      if (end > data.length) {
        const next = new Float32Array(end);
        next.set(data);
        data = next;
      }
      data.set(samples, at / 4);
    },
    read(into, at) {
      const part = data.subarray(at / 4, at / 4 + into.length);
      into.set(part);
      return part.length * 4;
    },
    flush: () => undefined,
    samples: () => data,
  };
}

function info(durationMs: number, rate: number): SourceInfo {
  return {
    duration: durationMs,
    display_width: 320,
    display_height: 180,
    rotation: "r0",
    orientation: "landscape",
    is_vfr: false,
    is_hdr: false,
    avg_fps: 30_000,
    video_codec: "h264",
    video_codec_string: "avc1.640028",
    audio_codec_string: "mp4a.40.2",
    audio_sample_rate: rate,
    audio_channels: 2,
    audio_track: 0,
    audio_tracks: 1,
  } as SourceInfo;
}

const never = () => false;

async function written(durationMs: number, from: number, file = memoryFile(), firstUs = 0) {
  const packets = Math.ceil((durationMs * 16) / FRAMES) + 2;
  const seconds: number[] = [];
  const result = await writeWholePcm16(
    file,
    demuxer(packets, 16_000, firstUs),
    info(durationMs, 16_000),
    from,
    () => undefined,
    (s) => seconds.push(s),
    never,
    { decoding: decoding(() => 16_000), resampler: fakeResampler() },
  );
  return { file, seconds, result };
}

function same(a: Float32Array, b: Float32Array): boolean {
  if (a.length !== b.length) {
    return false;
  }
  const x = new Uint32Array(a.buffer, a.byteOffset, a.length);
  const y = new Uint32Array(b.buffer, b.byteOffset, b.length);
  for (let i = 0; i < x.length; i += 1) {
    if (x[i] !== y[i]) {
      return false;
    }
  }
  return true;
}

describe("writeWholePcm16", () => {
  it("writes round(duration × 16) samples placed on the source timeline, with a checkpoint every 10 s", async () => {
    const { file, seconds } = await written(61_234, 0);
    const pcm = file.samples();
    expect(pcm.length).toBe(Math.round(61_234 * 16));
    expect(pcm[0]).toBe(Math.fround(wave(0)));
    expect(pcm[123_456]).toBe(Math.fround(wave(123_456)));
    expect(seconds).toEqual([10, 20, 30, 40, 50, 60, 61]);
  });

  it("drops the audio before time 0", async () => {
    // The first packet is presented 1,024 samples early: its samples are before 0.
    const { file } = await written(20_000, 0, memoryFile(), -64_000);
    expect(file.samples()[0]).toBe(Math.fround(wave(1024)));
  });

  it("resumed from 30 s gives the same file as a run from 0", async () => {
    const whole = (await written(75_000, 0)).file.samples();
    const partial = memoryFile();
    // A first run that stopped after 37 s: the file holds more than the checkpoint.
    partial.write(whole.subarray(0, 37 * 16_000), 0);
    const { file, seconds } = await written(75_000, 30, partial);
    expect(same(file.samples(), whole)).toBe(true);
    expect(seconds).toEqual([40, 50, 60, 70, 75]);
  });

  it("rebuilds the resampler when the decoded rate changes, and keeps the length rule", async () => {
    const made: number[] = [];
    const file = memoryFile();
    // 20 s: the first 100 packets decode at 48 kHz, the rest at 16 kHz.
    const packets = 400;
    await writeWholePcm16(file, demuxer(packets, 48_000), info(20_000, 48_000), 0, () => undefined, () => undefined, never, {
      decoding: decoding((i) => (i < 100 ? 48_000 : 16_000)),
      resampler: fakeResampler(made),
    });
    expect(made).toEqual([48_000, 16_000]);
    expect(file.samples().length).toBe(320_000);
  });

  it("measures the source-wide level as cloud-audio.ts does, within 0.1 dB", async () => {
    const { file, result } = await written(30_000, 0);
    if (typeof result === "symbol") {
      throw new Error("cancelled");
    }
    const sorted = Float32Array.from(file.samples(), Math.abs).sort();
    const exact = 20 * Math.log10(sorted[Math.floor(sorted.length * 0.95)] ?? 0);
    expect(Math.abs(result.levelP95 - exact)).toBeLessThanOrEqual(0.1);
  });

  it("stops when cancelled, and a decoder failure is SourceAudioError", async () => {
    let calls = 0;
    const cancelled = await writeWholePcm16(memoryFile(), demuxer(200, 16_000), info(12_000, 16_000), 0, () => undefined, () => undefined, () => ++calls > 1, {
      decoding: decoding(() => 16_000),
      resampler: fakeResampler(),
    });
    expect(cancelled).toBe(CANCELLED);
    await expect(
      decodeAudioRegion(demuxer(200, 16_000), info(12_000, 16_000), 0, 12_000, () => undefined, never, decoding(() => 16_000, 40)),
    ).rejects.toBeInstanceOf(SourceAudioError);
  });
});

describe("LevelHistogram", () => {
  it("gives -Infinity for silence and the 95th percentile otherwise", () => {
    const h = new LevelHistogram();
    expect(h.p95()).toBe(Number.NEGATIVE_INFINITY);
    h.add(new Float32Array(100));
    expect(h.p95()).toBe(Number.NEGATIVE_INFINITY);
    const loud = new LevelHistogram();
    loud.add(Float32Array.from({ length: 1000 }, (_, i) => (i < 960 ? 0.01 : 0.5)));
    // Sample 950 of 1,000 sorted is 0.01: -40 dBFS, to one 0.1 dB bin.
    expect(Math.abs(loud.p95() - -40)).toBeLessThanOrEqual(0.11);
  });
});
