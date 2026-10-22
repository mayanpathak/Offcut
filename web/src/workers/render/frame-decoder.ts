// The `VideoDecoder` of one `VideoSource` (TS §20.1): created and configured
// with the colours the clip needs (D-70), fed the demuxer's samples, drained
// after the last one, started again at a keyframe after a jump, and closed.
// Moved from `video-source.ts` (v4implementation D-150); the source keeps
// the queue of decoded frames and decides when to feed and when to restart.

import type { ClipInfo } from "../../gen/domain";
import { WorkerFailure } from "../rpc-worker";
import type { DemuxerHandle } from "./video-source";

/** What the decoder of a clip had to be told of its colours (D-70): kept for later sources of the same clip. */
const toldColours = new WeakMap<DemuxerHandle, VideoColorSpaceInit>();

/** The decoder is given more work only while it holds fewer samples than this. */
const DECODE_QUEUE_MAX = 3;
/** A time this far past everything fed so far is a jump: decoding starts again at its keyframe. */
const JUMP_AHEAD_US = 1_000_000;
/**
 * A time this little before the oldest frame held is not a jump back: the
 * clock that drives a preview may step back by a frame when it is corrected.
 */
const STEP_BACK_US = 100_000;

export function decodeFailure(detail: string): WorkerFailure {
  return new WorkerFailure("E_DECODE_VIDEO", detail);
}

/** What a `FrameDecoder` tells its source, and asks of it. */
export type FrameSink = {
  /** A decoder gave out a frame: it is open, and counted. */
  opened(): void;
  /** A frame of the decoder that runs now, for the queue. */
  keep(frame: VideoFrame): void;
  /** A frame nobody will see: closed, and no longer counted. */
  release(frame: VideoFrame): void;
  /** Whoever waits for a frame looks again. */
  wake(): void;
  fail(thrown: unknown): void;
  /** Whether the source failed already: then nothing more is fed. */
  failed(): boolean;
  /** Whether too few frames are decoded ahead of the time asked for. */
  wantsMore(): boolean;
  /** The decoder must be configured anew (D-70): decoding starts again at the time asked for last. */
  startAgain(): void;
};

export class FrameDecoder {
  readonly #demuxer: DemuxerHandle;
  readonly #sink: FrameSink;
  #config: VideoDecoderConfig;
  readonly #count: number;
  readonly #fullRange: boolean;

  #decoder: VideoDecoder | undefined;
  /** Goes up with every new decoder: an output of an old one is closed unseen. */
  #generation = 0;

  /** The next sample to give the decoder, in decoding order. */
  #next = 0;
  /** The time of the keyframe decoding last started at, and the latest time given to the decoder since. */
  #startUs = Number.POSITIVE_INFINITY;
  #fedUntilUs = Number.NEGATIVE_INFINITY;

  #draining = false;
  /** Every sample was decoded and every frame of it is out. */
  #drained = false;
  #closed = false;

  constructor(demuxer: DemuxerHandle, info: ClipInfo, sink: FrameSink) {
    this.#demuxer = demuxer;
    this.#sink = sink;
    this.#count = demuxer.video_sample_count();
    // The frames are stored unturned: the display size with its sides
    // exchanged for a clip that is shown turned a quarter.
    const turned = info.rotation === "r90" || info.rotation === "r270";
    const description = demuxer.video_description();
    const colorSpace = toldColours.get(demuxer);
    this.#fullRange = demuxer.video_full_range();
    this.#config = {
      codec: info.video_codec_string,
      codedWidth: turned ? info.display_height : info.display_width,
      codedHeight: turned ? info.display_width : info.display_height,
      ...(description === undefined ? {} : { description }),
      ...(colorSpace === undefined ? {} : { colorSpace }),
    };
  }

  /** Every sample was decoded and every frame of it is out. */
  get drained(): boolean {
    return this.#drained;
  }

  /**
   * Whether `tUs` is somewhere the decoder cannot get to by going on. `held`
   * is the time of the oldest frame the source still holds, if any.
   */
  jumped(tUs: number, held: number | undefined): boolean {
    if (this.#decoder === undefined) {
      return true;
    }
    if (this.#count === 0) {
      return false;
    }
    // Back: before the oldest frame that is still held, or, while none is
    // held yet, before the keyframe decoding started at. What lies between
    // that keyframe and a frame that is held was decoded and closed.
    if (tUs < (held ?? this.#startUs) - STEP_BACK_US) {
      return true;
    }
    // Ahead: only when a keyframe that was not fed yet is nearer to it. Going
    // back to a keyframe that was fed already would decode the same again.
    return (
      tUs > this.#fedUntilUs + JUMP_AHEAD_US &&
      this.#fedUntilUs !== Number.NEGATIVE_INFINITY &&
      this.#demuxer.keyframe_at_or_before(tUs / 1000) >= this.#next
    );
  }

  /** Closes the decoder that runs, if one does: the first half of a restart. */
  stop(): void {
    if (this.#decoder !== undefined && this.#decoder.state !== "closed") {
      this.#decoder.close();
    }
  }

  /** A new decoder, fed from the keyframe at or before `tUs`: the second half of a restart. */
  start(tUs: number): void {
    this.#next = this.#count === 0 ? 0 : this.#demuxer.keyframe_at_or_before(tUs / 1000);
    this.#startUs = Number.POSITIVE_INFINITY;
    this.#fedUntilUs = Number.NEGATIVE_INFINITY;
    this.#draining = false;
    this.#drained = false;

    this.#generation += 1;
    const generation = this.#generation;
    const decoder = new VideoDecoder({
      output: (frame) => {
        this.#sink.opened();
        if (generation !== this.#generation || this.#toldFullRange(frame)) {
          this.#sink.release(frame);
          return;
        }
        this.#sink.keep(frame);
        this.#sink.wake();
      },
      error: (error) => {
        if (generation === this.#generation) {
          this.#sink.fail(decodeFailure(error.name));
        }
      },
    });
    decoder.addEventListener("dequeue", () => {
      if (generation !== this.#generation) {
        return;
      }
      try {
        this.feed();
      } catch (thrown) {
        this.#sink.fail(thrown);
      }
      this.#sink.wake();
    });
    decoder.configure(this.#config);
    this.#decoder = decoder;
  }

  /** Closes the decoder for good: what it gives out from now on is closed unseen. */
  close(): void {
    this.#closed = true;
    this.#generation += 1;
    this.stop();
  }

  /**
   * A clip that says it is full-range is drawn so (D-70). The browser may
   * decode it as limited-range, and obeys only a whole colour space: so the
   * decoder is told the range with the colours the browser assumed, and
   * decoding starts again, once for a clip. Returns whether it did.
   */
  #toldFullRange(frame: VideoFrame): boolean {
    const assumed = frame.colorSpace;
    if (!this.#fullRange || assumed.fullRange === true || this.#config.colorSpace !== undefined) {
      return false;
    }
    const colorSpace: VideoColorSpaceInit = {
      fullRange: true,
      matrix: assumed.matrix ?? "bt709",
      primaries: assumed.primaries ?? "bt709",
      transfer: assumed.transfer ?? "bt709",
    };
    toldColours.set(this.#demuxer, colorSpace);
    this.#config = { ...this.#config, colorSpace };
    this.#sink.startAgain();
    return true;
  }

  /** Gives the decoder samples while it has room and too few frames are decoded ahead. */
  feed(): void {
    const decoder = this.#decoder;
    if (decoder === undefined || decoder.state !== "configured" || this.#sink.failed()) {
      return;
    }
    while (this.#next < this.#count && decoder.decodeQueueSize < DECODE_QUEUE_MAX && this.#sink.wantsMore()) {
      const sample = this.#demuxer.read_video_sample(this.#next);
      this.#startUs = Math.min(this.#startUs, sample.ptsUs);
      this.#fedUntilUs = Math.max(this.#fedUntilUs, sample.ptsUs);
      this.#next += 1;
      decoder.decode(
        new EncodedVideoChunk({
          type: sample.isKeyframe ? "key" : "delta",
          timestamp: sample.ptsUs,
          duration: sample.durationUs,
          data: sample.data,
        }),
      );
    }
    // After the last sample the decoder is told to give out what it holds.
    if (this.#next >= this.#count && !this.#draining) {
      this.#draining = true;
      const generation = this.#generation;
      decoder.flush().then(
        () => {
          if (generation === this.#generation) {
            this.#drained = true;
            this.#sink.wake();
          }
        },
        (error: unknown) => {
          if (generation === this.#generation && !this.#closed) {
            this.#sink.fail(error);
          }
        },
      );
    }
  }
}
