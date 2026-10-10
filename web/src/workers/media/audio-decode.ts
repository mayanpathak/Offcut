// Decodes a clip's audio track to mono PCM at the source rate, placed on the
// clip's timeline: sample 0 is video time 0, and the length is the video's.
// The placing reads timestamps only, never the audio itself (INV-5).

import type { ClipInfo } from "../../gen/domain";
import { type CoreDemuxer, loadCore } from "../../wasm/load-core";
import { CANCELLED, WorkerFailure } from "../rpc";

/** Compressed frames handed to the decoder between two looks at the cancel flag. */
const BATCH = 32;
// The decoder is given more work only while it holds fewer frames than this.
// It works through a deep queue several times faster than through a shallow
// one (measured in Prompt 37: 0.5 s against 1.7 s for 75 s of audio), and 256
// frames are about 130 kB of compressed audio.
const QUEUE_HIGH_WATER = 256;

function decodeFailure(detail: string): WorkerFailure {
  return new WorkerFailure("E_DECODE_AUDIO", detail);
}

/**
 * Resolves in the next turn of the event loop: the decoder's outputs are
 * delivered, and a cancel message that has arrived is read.
 */
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

/** Resolves when the decoder has taken work off its queue. */
function dequeued(decoder: AudioDecoder): Promise<void> {
  return new Promise((resolve) => {
    decoder.addEventListener("dequeue", () => {
      resolve();
    }, { once: true });
  });
}

/**
 * Mono `Float32Array` at `info.audio_sample_rate`, of exactly
 * `round(duration x rate / 1000)` samples. Audio that starts after video
 * time 0 is preceded by silence; audio from before it is dropped; the tail
 * is padded with silence or cut to the video's length.
 */
export async function decodeAudio(
  handle: FileSystemSyncAccessHandle,
  info: ClipInfo,
  isCancelled: () => boolean,
): Promise<Float32Array<ArrayBuffer> | typeof CANCELLED> {
  const core = await loadCore();
  const demuxer = core.openDemuxer(handle);
  if ("rejected" in demuxer) {
    throw decodeFailure("NotOpened");
  }
  try {
    return await decodeTrack(demuxer, info, isCancelled);
  } finally {
    demuxer.free();
  }
}

async function decodeTrack(
  demuxer: CoreDemuxer,
  info: ClipInfo,
  isCancelled: () => boolean,
): Promise<Float32Array<ArrayBuffer> | typeof CANCELLED> {
  const rate = info.audio_sample_rate;
  const count = demuxer.audioSampleCount();
  const out = new Float32Array(Math.round((info.duration * rate) / 1000));
  if (count === 0) {
    return out;
  }

  // Where the next decoded sample goes. The first one belongs at the time
  // the container gives audio sample 0, which may be before video time 0.
  let position = Math.round((demuxer.readAudioSample(0).ptsUs * rate) / 1_000_000);
  let plane = new Float32Array(0);
  // The first failure the decoder reports, from either of its callbacks.
  const state: { failed: WorkerFailure | undefined } = { failed: undefined };

  const decoder = new AudioDecoder({
    output: (data) => {
      try {
        if (data.sampleRate !== rate) {
          state.failed ??= decodeFailure("SampleRate");
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
        // Only the part that lies on the clip's timeline is kept.
        const from = Math.max(0, -position);
        const to = Math.min(frames, out.length - position);
        if (from < to) {
          out.set(mono.subarray(from, to), position + from);
        }
        position += frames;
      } finally {
        data.close();
      }
    },
    error: (error) => {
      state.failed ??= decodeFailure(error.name);
    },
  });

  try {
    const description = demuxer.audioDescription();
    decoder.configure({
      codec: info.audio_codec_string,
      sampleRate: rate,
      numberOfChannels: info.audio_channels,
      ...(description === undefined ? {} : { description }),
    });
    for (let index = 0; index < count; index += BATCH) {
      if (isCancelled()) {
        return CANCELLED;
      }
      for (let i = index; i < Math.min(index + BATCH, count); i += 1) {
        const sample = demuxer.readAudioSample(i);
        decoder.decode(
          new EncodedAudioChunk({ type: "key", timestamp: sample.ptsUs, duration: sample.durationUs, data: sample.data }),
        );
      }
      await nextTask();
      while (decoder.decodeQueueSize > QUEUE_HIGH_WATER && state.failed === undefined) {
        await dequeued(decoder);
      }
      if (state.failed !== undefined) {
        throw state.failed;
      }
    }
    await decoder.flush();
  } catch (thrown) {
    // The decoder's own error, when it has one, says more than the throw it caused.
    throw state.failed ?? (thrown instanceof DOMException ? decodeFailure(thrown.name) : thrown);
  } finally {
    if (decoder.state !== "closed") {
      decoder.close();
    }
  }
  if (state.failed !== undefined) {
    throw state.failed;
  }
  return out;
}
