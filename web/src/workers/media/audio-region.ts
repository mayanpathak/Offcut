// A long source's sound, decoded in windows and written as 16 kHz mono PCM
// (v6.1implementation §7.2, D-402 to D-404, TS §43.5). The model is
// `audio-decode.ts`'s `decodeTrack` (batches of 32, the 256-frame queue,
// sample 0 at the first sample's time, audio before time 0 dropped), which is
// not changed: the clip path keeps it. Here no buffer ever holds the track:
// at most one 10 s chunk at the decoded rate and the resampler's state.

import type { SourceInfo } from "../../gen/domain";
import type { CoreSourceDemuxer, CoreStreamResampler } from "../../wasm/load-core";
import { CANCELLED } from "../rpc-worker";

/** Compressed frames handed to the decoder between two looks at the cancel flag. */
const BATCH = 32;
/** The decoder is given more work only while it holds fewer frames than this. */
const QUEUE_HIGH_WATER = 256;
/** How much earlier than a region's start decoding begins, so AAC's overlap is primed (D-403). */
export const AAC_PREROLL_MS = 100;
/** The most decoded audio one chunk holds (D-402). */
export const CHUNK_SECONDS = 10;
/** The rate of `pcm16.f32`. */
export const PCM_RATE = 16_000;
/** A checkpoint every this many seconds of written sound (D-403). */
export const CHECKPOINT_SECONDS = 10;

/** The demuxer calls a region needs. */
export type RegionDemuxer = Pick<
  CoreSourceDemuxer,
  "audioSampleCount" | "audioSampleTimeUs" | "readAudioSample" | "audioDescription"
>;

/** What a decoded chunk is handed to: mono samples, the source time of the first in µs, their rate. */
export type ChunkSink = (mono: Float32Array, atUs: number, rate: number) => void;

/** The decoded audio's shape, as WebCodecs' `AudioData` has it. */
export type DecodedAudio = {
  readonly sampleRate: number;
  readonly numberOfFrames: number;
  readonly numberOfChannels: number;
  copyTo(destination: Float32Array, options: { planeIndex: number; format: "f32-planar" }): void;
  close(): void;
};

/** The decoder calls used, as WebCodecs' `AudioDecoder` has them. */
export type RegionDecoder = {
  configure(config: AudioDecoderConfig): void;
  decode(chunk: { type: "key"; timestamp: number; duration: number; data: Uint8Array }): void;
  flush(): Promise<void>;
  close(): void;
  readonly decodeQueueSize: number;
  readonly state: string;
};

export type DecodingDeps = {
  decoder(output: (data: DecodedAudio) => void, error: (e: { name: string }) => void): RegionDecoder;
};

/** The browser's decoder. */
export const browserDecoding: DecodingDeps = {
  decoder(output, error) {
    const decoder = new AudioDecoder({ output, error });
    return {
      configure: (config) => {
        decoder.configure(config);
      },
      decode: (chunk) => {
        decoder.decode(new EncodedAudioChunk(chunk));
      },
      flush: () => decoder.flush(),
      close: () => {
        decoder.close();
      },
      get decodeQueueSize() {
        return decoder.decodeQueueSize;
      },
      get state() {
        return decoder.state;
      },
    };
  },
};

/** Why decoding failed: the source's sound cannot be read (`E_SOURCE_AUDIO`). */
export class SourceAudioError extends Error {
  readonly detail: string;

  constructor(detail: string) {
    super("the source's sound cannot be decoded");
    this.name = "SourceAudioError";
    this.detail = detail;
  }
}

/** Resolves in the next turn of the event loop, so outputs and a cancel are read. */
function nextTask(): Promise<void> {
  return new Promise((resolve) => {
    const channel = new MessageChannel();
    channel.port1.onmessage = () => {
      channel.port1.close();
      resolve();
    };
    channel.port2.postMessage(null);
  });
}

/** The index of the last audio sample presented at or before `us`; 0 when none is. */
export function sampleAtOrBefore(demuxer: Pick<RegionDemuxer, "audioSampleCount" | "audioSampleTimeUs">, us: number): number {
  let lo = 0;
  let hi = demuxer.audioSampleCount() - 1;
  let best = 0;
  while (lo <= hi) {
    const mid = (lo + hi) >> 1;
    const at = demuxer.audioSampleTimeUs(mid);
    if (at !== undefined && at <= us) {
      best = mid;
      lo = mid + 1;
    } else {
      hi = mid - 1;
    }
  }
  return best;
}

/**
 * Decodes the first audio track over `[fromMs, toMs)` of the source's
 * timeline, from the sample at or before `fromMs − AAC_PREROLL_MS`, handing
 * mono chunks of at most 10 s to `onChunk`. Audio before time 0 and from
 * `toMs` on is dropped. A decoded rate that differs from the container's is
 * reported with its chunk, not refused (TS §43.5).
 */
export async function decodeAudioRegion(
  demuxer: RegionDemuxer,
  info: SourceInfo,
  fromMs: number,
  toMs: number,
  onChunk: ChunkSink,
  isCancelled: () => boolean,
  deps: DecodingDeps = browserDecoding,
): Promise<undefined | typeof CANCELLED> {
  const count = demuxer.audioSampleCount();
  if (count === 0 || toMs <= fromMs) {
    return undefined;
  }
  const first = sampleAtOrBefore(demuxer, (fromMs - AAC_PREROLL_MS) * 1000);
  const toUs = toMs * 1000;
  let positionUs = demuxer.audioSampleTimeUs(first) ?? 0;
  let pending: { mono: Float32Array; filled: number; atUs: number; rate: number } | undefined;
  let plane = new Float32Array(0);
  const state: { failed: SourceAudioError | undefined; ended: boolean } = { failed: undefined, ended: false };

  const emit = () => {
    if (pending !== undefined && pending.filled > 0) {
      onChunk(pending.mono.subarray(0, pending.filled), pending.atUs, pending.rate);
    }
    pending = undefined;
  };

  const keep = (mono: Float32Array, atUs: number, rate: number) => {
    // Only what lies on the source's timeline before `toMs` is kept.
    const usPerSample = 1_000_000 / rate;
    const from = Math.min(mono.length, Math.max(0, Math.ceil(-atUs / usPerSample - 1e-9)));
    const to = Math.max(from, Math.min(mono.length, Math.ceil((toUs - atUs) / usPerSample - 1e-9)));
    let at = from;
    while (at < to) {
      if (pending !== undefined && (pending.rate !== rate || pending.filled === pending.mono.length)) {
        emit();
      }
      pending ??= { mono: new Float32Array(rate * CHUNK_SECONDS), filled: 0, atUs: atUs + at * usPerSample, rate };
      const take = Math.min(to - at, pending.mono.length - pending.filled);
      pending.mono.set(mono.subarray(at, at + take), pending.filled);
      pending.filled += take;
      at += take;
    }
    if (to < mono.length) {
      state.ended = true;
    }
  };

  const decoder = deps.decoder(
    (data) => {
      try {
        if (state.failed !== undefined || state.ended) {
          return;
        }
        const frames = data.numberOfFrames;
        const channels = data.numberOfChannels;
        if (plane.length < frames) {
          plane = new Float32Array(frames);
        }
        const mono = new Float32Array(frames);
        for (let channel = 0; channel < channels; channel += 1) {
          data.copyTo(plane, { planeIndex: channel, format: "f32-planar" });
          for (let i = 0; i < frames; i += 1) {
            mono[i] = (mono[i] ?? 0) + (plane[i] ?? 0) / channels;
          }
        }
        const atUs = positionUs;
        positionUs += (frames * 1_000_000) / data.sampleRate;
        keep(mono, atUs, data.sampleRate);
      } finally {
        data.close();
      }
    },
    (error) => {
      state.failed ??= new SourceAudioError(error.name);
    },
  );

  try {
    const description = demuxer.audioDescription();
    decoder.configure({
      codec: info.audio_codec_string,
      sampleRate: info.audio_sample_rate,
      numberOfChannels: info.audio_channels,
      ...(description === undefined ? {} : { description }),
    });
    for (let index = first; index < count && !state.ended; index += BATCH) {
      if (isCancelled()) {
        return CANCELLED;
      }
      for (let i = index; i < Math.min(index + BATCH, count); i += 1) {
        const sample = demuxer.readAudioSample(i);
        if (sample.ptsUs >= toUs) {
          break;
        }
        decoder.decode({ type: "key", timestamp: sample.ptsUs, duration: sample.durationUs, data: sample.data });
      }
      await nextTask();
      while (decoder.decodeQueueSize > QUEUE_HIGH_WATER && state.failed === undefined) {
        await nextTask();
      }
      if (state.failed !== undefined) {
        throw state.failed;
      }
      // The next batch starts at or after the region's end: nothing more is needed.
      const nextIndex = index + BATCH;
      if (nextIndex < count && (demuxer.audioSampleTimeUs(nextIndex) ?? 0) >= toUs) {
        break;
      }
    }
    await decoder.flush();
  } catch (thrown) {
    throw state.failed ?? (thrown instanceof DOMException ? new SourceAudioError(thrown.name) : thrown);
  } finally {
    if (decoder.state !== "closed") {
      decoder.close();
    }
  }
  if (state.failed !== undefined) {
    throw state.failed;
  }
  emit();
  return undefined;
}

// --- pcm16.f32 ---------------------------------------------------------------

/** The file the samples are written to, as an OPFS sync handle offers it. */
export type PcmFile = {
  getSize(): number;
  truncate(bytes: number): void;
  write(data: Float32Array, at: number): void;
  read(into: Float32Array, at: number): number;
  flush(): void;
};

export type ResamplerFactory = (fromHz: number, toHz: number) => Pick<CoreStreamResampler, "push" | "finish" | "free">;

/** The greatest common divisor of two rates. */
function gcd(a: number, b: number): number {
  return b === 0 ? a : gcd(b, a % b);
}

/** 0.1 dB bins of |sample| from −120 dBFS to 0 (D-404). */
const BINS = 1201;
const FLOOR_DB = -120;

export class LevelHistogram {
  private readonly counts = new Float64Array(BINS);
  private total = 0;

  add(samples: Float32Array): void {
    for (const sample of samples) {
      const level = Math.abs(sample);
      const db = level > 0 ? 20 * Math.log10(level) : FLOOR_DB;
      const bin = Math.min(BINS - 1, Math.max(0, Math.floor((db - FLOOR_DB) * 10)));
      this.counts[bin] = (this.counts[bin] ?? 0) + 1;
    }
    this.total += samples.length;
  }

  /** The level 95% of the samples are under, in dBFS (as `loudLevelDb` of `cloud-audio.ts`). */
  p95(): number {
    if (this.total === 0) {
      return Number.NEGATIVE_INFINITY;
    }
    const wanted = Math.floor(this.total * 0.95) + 1;
    let seen = 0;
    for (let bin = 0; bin < BINS; bin += 1) {
      seen += this.counts[bin] ?? 0;
      if (seen >= wanted) {
        return bin === 0 ? Number.NEGATIVE_INFINITY : FLOOR_DB + bin / 10;
      }
    }
    return 0;
  }
}

/**
 * Writes the source's sound as `round(duration × 16)` samples of 16 kHz mono
 * to `file`, from `fromSeconds` on (D-403): the file is cut to that point,
 * decoding starts a little earlier, and what lies before it is dropped.
 * `onSecond(s)` follows every 10 s written and flushed; `onProgress` the
 * milliseconds done. The level histogram covers the whole source (D-404).
 */
export async function writeWholePcm16(
  file: PcmFile,
  demuxer: RegionDemuxer,
  info: SourceInfo,
  fromSeconds: number,
  onProgress: (doneMs: number) => void,
  onSecond: (seconds: number) => void,
  isCancelled: () => boolean,
  deps: { decoding?: DecodingDeps; resampler: ResamplerFactory },
): Promise<{ levelP95: number } | typeof CANCELLED> {
  const total16 = Math.round(info.duration * (PCM_RATE / 1000));
  const start16 = Math.min(Math.max(0, Math.floor(fromSeconds)) * PCM_RATE, total16);
  file.truncate(start16 * 4);
  const histogram = new LevelHistogram();
  // The part already written still counts in the level.
  const reread = new Float32Array(PCM_RATE * CHUNK_SECONDS);
  for (let at = 0; at < start16; at += reread.length) {
    const n = Math.min(reread.length, start16 - at);
    file.read(reread.subarray(0, n), at * 4);
    histogram.add(reread.subarray(0, n));
  }

  let next16 = start16;
  let lastCheckpoint = Math.floor(start16 / (PCM_RATE * CHECKPOINT_SECONDS));
  let current: { r: ReturnType<ResamplerFactory>; rate: number; base16: number; produced: number; input: number } | undefined;

  const place = (samples: Float32Array, at16: number) => {
    // Silence fills a gap; what lies before the next sample to write is already there.
    if (at16 > next16 && next16 < total16) {
      const gap = new Float32Array(Math.min(at16, total16) - next16);
      file.write(gap, next16 * 4);
      histogram.add(gap);
      next16 += gap.length;
    }
    const from = Math.max(0, next16 - at16);
    const to = Math.min(samples.length, total16 - at16);
    if (to > from) {
      const part = samples.subarray(from, to);
      file.write(part, next16 * 4);
      histogram.add(part);
      next16 += part.length;
    }
    const reached = Math.floor(next16 / (PCM_RATE * CHECKPOINT_SECONDS));
    if (reached > lastCheckpoint) {
      file.flush();
      lastCheckpoint = reached;
      onSecond(reached * CHECKPOINT_SECONDS);
    }
    onProgress(next16 / (PCM_RATE / 1000));
  };

  const finishCurrent = () => {
    if (current === undefined) {
      return;
    }
    const c = current;
    current = undefined;
    const tail = c.r.finish(c.input);
    place(tail, c.base16 + c.produced);
  };

  try {
    const result = await decodeAudioRegion(
      demuxer,
      info,
      (start16 / PCM_RATE) * 1000,
      info.duration,
      (mono, atUs, rate) => {
        let input = mono;
        if (current === undefined || current.rate !== rate) {
          // A rate the decoder changed to rebuilds the resampler there (D-402).
          finishCurrent();
          // The resampler starts on an input sample that lies exactly on the 16 kHz grid, as
          // a run from 0 does: otherwise a resumed run is shifted by a fraction of a sample
          // (P174's smoke read 0.31 apart). The few samples skipped are in the pre-roll.
          const step = rate / gcd(rate, PCM_RATE);
          const n0 = Math.round((atUs * rate) / 1_000_000);
          const skip = (((step - (n0 % step)) % step) + step) % step;
          input = mono.subarray(Math.min(skip, mono.length));
          current = { r: deps.resampler(rate, PCM_RATE), rate, base16: ((n0 + skip) * PCM_RATE) / rate, produced: 0, input: 0 };
        }
        const out = current.r.push(input);
        current.input += input.length;
        place(out, current.base16 + current.produced);
        current.produced += out.length;
      },
      isCancelled,
      deps.decoding,
    );
    if (result === CANCELLED) {
      current?.r.free();
      current = undefined;
      return CANCELLED;
    }
    finishCurrent();
  } catch (thrown) {
    current?.r.free();
    throw thrown;
  }
  if (next16 < total16) {
    place(new Float32Array(0), total16);
  }
  file.flush();
  onSecond(Math.floor(total16 / PCM_RATE));
  return { levelP95: histogram.p95() };
}
