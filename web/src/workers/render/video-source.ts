// The decoded frames of a clip, by time: the demuxer's samples through one
// `VideoDecoder` into a short queue (TS §20.1). The preview asks for the
// frame of a time without waiting; the export waits for it.
//
// Who closes a frame: this file, and nothing else. A frame that was handed
// out stays open until a newer one takes its place, so the loop that holds
// it may draw it again, and must not close it (TS §21.4).

import type { ClipInfo, TimeMs } from "../../gen/domain";
import { WorkerFailure } from "../rpc";

/** What the decoder is fed through. `RenderSession` of the render bundle is one (D-30). */
export interface DemuxerHandle {
  /** The raw `avcC` payload. */
  video_description(): Uint8Array | undefined;
  video_sample_count(): number;
  /** Sample `index` in decoding order, with its place on the clip's timeline in microseconds. */
  read_video_sample(index: number): { data: Uint8Array; ptsUs: number; durationUs: number; isKeyframe: boolean };
  /** The number of the last keyframe at or before `tMs`. */
  keyframe_at_or_before(tMs: number): number;
  /** Whether the clip's video says it uses the whole brightness range, 0 to 255. */
  video_full_range(): boolean;
}

/** What the decoder of a clip had to be told of its colours (D-70): kept for later sources of the same clip. */
const toldColours = new WeakMap<DemuxerHandle, VideoColorSpaceInit>();

/**
 * The `VideoFrame`s that are open. Every code path that obtains one adds 1,
 * here for a decoder's output, and every `close()` takes 1 away. 0 at the end
 * of an export and after a preview stops, or a frame leaked (INV-11, D-45).
 */
export const liveFrames: { count: number } = { count: 0 };

/** The preview keeps up to this many decoded frames ahead of the time it shows. */
export const PREVIEW_QUEUE = 6;
/** The export keeps up to this many (TS §31). */
const EXPORT_QUEUE = 8;
/** The decoder is given more work only while it holds fewer samples than this. */
const DECODE_QUEUE_MAX = 3;
/** A time this far past everything fed so far is a jump: decoding starts again at its keyframe. */
const JUMP_AHEAD_US = 1_000_000;
/**
 * A time this little before the oldest frame held is not a jump back: the
 * clock that drives a preview may step back by a frame when it is corrected.
 */
const STEP_BACK_US = 100_000;

function decodeFailure(detail: string): WorkerFailure {
  return new WorkerFailure("E_DECODE_VIDEO", detail);
}

/** What was thrown, as the failure of this file. The demuxer throws `{ code, detail }`. */
function asFailure(thrown: unknown): WorkerFailure {
  if (thrown instanceof WorkerFailure) {
    return thrown;
  }
  if (typeof thrown === "object" && thrown !== null && "detail" in thrown && typeof thrown.detail === "string") {
    return decodeFailure(thrown.detail);
  }
  return decodeFailure(thrown instanceof Error ? thrown.name : "Error");
}

export class VideoSource {
  readonly #demuxer: DemuxerHandle;
  #config: VideoDecoderConfig;
  readonly #count: number;
  readonly #fullRange: boolean;

  #decoder: VideoDecoder | undefined;
  /** Goes up with every new decoder: an output of an old one is closed unseen. */
  #generation = 0;
  /** Decoded frames that were not handed out yet, in the order of their time. */
  #queue: VideoFrame[] = [];
  /** The frame that was handed out last. */
  #shown: VideoFrame | null = null;
  /** `#shown` is from before a jump: still open for the loop that holds it, and not handed out again. */
  #shownIsOld = false;

  /** The next sample to give the decoder, in decoding order. */
  #next = 0;
  /** The time of the keyframe decoding last started at, and the latest time given to the decoder since. */
  #startUs = Number.POSITIVE_INFINITY;
  #fedUntilUs = Number.NEGATIVE_INFINITY;
  /** The time last asked for and how many frames to keep ahead of it: what feeding goes by between two calls. */
  #wantUs = 0;
  #ahead = PREVIEW_QUEUE;

  #draining = false;
  /** Every sample was decoded and every frame of it is out. */
  #drained = false;
  #failed: WorkerFailure | undefined;
  #closed = false;
  #waiters: (() => void)[] = [];

  constructor(demuxer: DemuxerHandle, info: ClipInfo) {
    this.#demuxer = demuxer;
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

  /**
   * The latest decoded frame at or before `t`, or `null` while it is still
   * being decoded. Never waits. The frame stays this object's: do not close it.
   */
  frameAt(t: TimeMs): VideoFrame | null {
    const tUs = t * 1000;
    this.#ask(tUs, PREVIEW_QUEUE);
    return this.#take(tUs);
  }

  /**
   * The frame of `t`: the latest at or before it, known to be the latest
   * because the one after it is decoded too, or the clip has ended. Waits for
   * the decoder; never returns the frame of an earlier time in its place. For
   * the export. The frame stays this object's: do not close it.
   */
  async frameAtBlocking(t: TimeMs): Promise<VideoFrame> {
    const tUs = t * 1000;
    for (;;) {
      this.#ask(tUs, EXPORT_QUEUE);
      if (this.#drained || this.#queue.some((frame) => frame.timestamp > tUs)) {
        // A clip whose first frame is after `t` starts with that frame.
        const frame = this.#take(tUs) ?? this.#takeFirst();
        if (frame === null) {
          throw decodeFailure("NoFrame");
        }
        return frame;
      }
      await new Promise<void>((resolve) => {
        this.#waiters.push(resolve);
      });
    }
  }

  /** V4: starts decoding at `from` before a frame of it is asked for. */
  prefetch(from: TimeMs): void {
    void from;
  }

  /** Closes every frame this object holds, the one handed out last too, and the decoder. */
  close(): void {
    if (this.#closed) {
      return;
    }
    this.#closed = true;
    this.#generation += 1;
    this.#dropQueue();
    if (this.#shown !== null) {
      this.#release(this.#shown);
      this.#shown = null;
    }
    if (this.#decoder !== undefined && this.#decoder.state !== "closed") {
      this.#decoder.close();
    }
    this.#wake();
  }

  #release(frame: VideoFrame): void {
    frame.close();
    liveFrames.count -= 1;
  }

  #dropQueue(): void {
    for (const frame of this.#queue) {
      this.#release(frame);
    }
    this.#queue = [];
  }

  #wake(): void {
    const waiters = this.#waiters;
    this.#waiters = [];
    for (const resolve of waiters) {
      resolve();
    }
  }

  #fail(thrown: unknown): void {
    this.#failed ??= asFailure(thrown);
    this.#wake();
  }

  /** Notes what is asked for, starts decoding somewhere else when the time jumped, and feeds the decoder. */
  #ask(tUs: number, ahead: number): void {
    if (this.#closed) {
      throw decodeFailure("Closed");
    }
    if (this.#failed !== undefined) {
      throw this.#failed;
    }
    this.#wantUs = tUs;
    this.#ahead = ahead;
    try {
      if (this.#jumped(tUs)) {
        this.#restart(tUs);
      }
      this.#feed();
    } catch (thrown) {
      this.#failed ??= asFailure(thrown);
      throw this.#failed;
    }
  }

  /** Whether `tUs` is somewhere the decoder cannot get to by going on. */
  #jumped(tUs: number): boolean {
    if (this.#decoder === undefined) {
      return true;
    }
    if (this.#count === 0) {
      return false;
    }
    // Back: before the oldest frame that is still held, or, while none is
    // held yet, before the keyframe decoding started at. What lies between
    // that keyframe and a frame that is held was decoded and closed.
    const held = this.#shown !== null && !this.#shownIsOld ? this.#shown.timestamp : this.#queue[0]?.timestamp;
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

  /** A new decoder, fed from the keyframe at or before `tUs`. What was decoded for another place is closed. */
  #restart(tUs: number): void {
    if (this.#decoder !== undefined && this.#decoder.state !== "closed") {
      this.#decoder.close();
    }
    this.#dropQueue();
    this.#shownIsOld = this.#shown !== null;
    this.#next = this.#count === 0 ? 0 : this.#demuxer.keyframe_at_or_before(tUs / 1000);
    this.#startUs = Number.POSITIVE_INFINITY;
    this.#fedUntilUs = Number.NEGATIVE_INFINITY;
    this.#draining = false;
    this.#drained = false;

    this.#generation += 1;
    const generation = this.#generation;
    const decoder = new VideoDecoder({
      output: (frame) => {
        liveFrames.count += 1;
        if (generation !== this.#generation || this.#toldFullRange(frame)) {
          this.#release(frame);
          return;
        }
        this.#keep(frame);
        this.#wake();
      },
      error: (error) => {
        if (generation === this.#generation) {
          this.#fail(decodeFailure(error.name));
        }
      },
    });
    decoder.addEventListener("dequeue", () => {
      if (generation !== this.#generation) {
        return;
      }
      try {
        this.#feed();
      } catch (thrown) {
        this.#fail(thrown);
      }
      this.#wake();
    });
    decoder.configure(this.#config);
    this.#decoder = decoder;
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
    try {
      this.#restart(this.#wantUs);
      this.#feed();
    } catch (thrown) {
      this.#fail(thrown);
    }
    return true;
  }

  /** Gives the decoder samples while it has room and too few frames are decoded ahead. */
  #feed(): void {
    const decoder = this.#decoder;
    if (decoder === undefined || decoder.state !== "configured" || this.#failed !== undefined) {
      return;
    }
    while (this.#next < this.#count && decoder.decodeQueueSize < DECODE_QUEUE_MAX && this.#framesAhead() < this.#ahead) {
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
            this.#wake();
          }
        },
        (error: unknown) => {
          if (generation === this.#generation && !this.#closed) {
            this.#fail(error);
          }
        },
      );
    }
  }

  #framesAhead(): number {
    return this.#queue.reduce((count, frame) => count + (frame.timestamp > this.#wantUs ? 1 : 0), 0);
  }

  /** Puts a decoded frame in its place. Of the frames at or before the time asked for, only the latest is kept. */
  #keep(frame: VideoFrame): void {
    const after = this.#queue.findIndex((queued) => queued.timestamp > frame.timestamp);
    this.#queue.splice(after === -1 ? this.#queue.length : after, 0, frame);
    const due = this.#queue.filter((queued) => queued.timestamp <= this.#wantUs).length;
    for (const old of this.#queue.splice(0, Math.max(0, due - 1))) {
      this.#release(old);
    }
  }

  /** Hands out the latest frame at or before `tUs`, closing the ones before it; or the one handed out last, if it still is that frame. */
  #take(tUs: number): VideoFrame | null {
    const due = this.#queue.filter((frame) => frame.timestamp <= tUs).length;
    if (due > 0) {
      const taken = this.#queue.splice(0, due);
      const chosen = taken.pop();
      for (const old of taken) {
        this.#release(old);
      }
      if (chosen !== undefined) {
        return this.#show(chosen);
      }
    }
    return this.#shown !== null && !this.#shownIsOld && this.#shown.timestamp <= tUs ? this.#shown : null;
  }

  /** Hands out the earliest decoded frame, when none is at or before the time asked for. */
  #takeFirst(): VideoFrame | null {
    const first = this.#queue.shift();
    return first === undefined ? null : this.#show(first);
  }

  #show(frame: VideoFrame): VideoFrame {
    if (this.#shown !== null) {
      this.#release(this.#shown);
    }
    this.#shown = frame;
    this.#shownIsOld = false;
    return frame;
  }
}
