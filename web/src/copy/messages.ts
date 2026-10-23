// Every word the app shows to a person (INV-15). No component holds a
// sentence of its own, and no sentence here holds a typed-in number: a number
// comes from the generated limits or from the caller.
//
// Wording rules (TS §25.1 P-12): the privacy claim is the factual one, that
// the video is not uploaded. Legal and certification terms are not used.

import type { AnalyticsEvent, Wanted } from "../gen/api";
import { type ErrorCode, type EventKind, LIMITS, type RejectReason, type UnsupportedReason } from "../gen/domain";

type EventName = AnalyticsEvent["name"];
type ErrorCopy = { title: string; body: string; action: string };
/**
 * The error codes that can reach a person: the six of V1 and, since V2, the
 * three of the model download. The others get copy with the flows they belong to.
 */
type ErrorCodeWithCopy = Extract<
  ErrorCode,
  | "E_NET_OFFLINE"
  | "E_NET_TIMEOUT"
  | "E_API_5XX"
  | "E_API_RATE_LIMITED"
  | "E_INTERNAL"
  | "E_WORKER_CRASH"
  | "E_MODEL_DOWNLOAD"
  | "E_MODEL_HASH"
  | "E_MODEL_STORAGE"
>;

const MS_PER_SECOND = 1000;
const SECONDS_PER_MINUTE = 60;
/** MB is 10^6 bytes, as `limits.rs` reads PS §9.4. */
const BYTES_PER_MB = 1_000_000;
/** `MAX_INPUT_FPS` is in thousandths of a frame per second. */
const FPS_MILLI_PER_FPS = 1000;
const MAX_FPS = LIMITS.MAX_INPUT_FPS / FPS_MILLI_PER_FPS;
const PERCENT = 100;

/** "45 seconds", "1 minute", "12 minutes": a wait, rounded up to whole minutes from one minute on. */
function waitText(seconds: number): string {
  if (seconds < SECONDS_PER_MINUTE) {
    const whole = Math.max(Math.ceil(seconds), 1);
    return `${String(whole)} ${whole === 1 ? "second" : "seconds"}`;
  }
  const minutes = Math.ceil(seconds / SECONDS_PER_MINUTE);
  return `${String(minutes)} ${minutes === 1 ? "minute" : "minutes"}`;
}

/** "0:42", "1:05": a time in the video, as a player shows it. */
function clockTime(ms: number): string {
  const total = Math.max(0, Math.floor(ms / MS_PER_SECOND));
  const seconds = total % SECONDS_PER_MINUTE;
  return `${String(Math.floor(total / SECONDS_PER_MINUTE))}:${seconds < 10 ? "0" : ""}${String(seconds)}`;
}

export const messages = {
  landing: {
    hero: "Turn what you say into a finished vertical short.",
    subhead:
      "Drop in a raw talking-head clip. Get back a vertical short with a cleaned voice, animated captions, and visuals for the numbers and lists you spoke.",
    supportedBrowsers: "Works in Chrome or Edge on Windows and Mac.",
    privacyLine: "Your video stays on your computer.",
    whatLeavesLink: "What leaves your device",
    demoCaption: "A demo, made by hand, of the kind of short Offcut will make.",
    waitlistHeading: "Offcut is not open yet. Leave your email and we will tell you when it is.",
  },

  dropZone: {
    prompt: (limits: Pick<typeof LIMITS, "MAX_CLIP_DURATION">) =>
      `Drop a video (up to ${String(limits.MAX_CLIP_DURATION / MS_PER_SECOND)} seconds, in English)`,
    or: "or", sampleButton: "Try a sample",
    fetchingSample: "Getting the sample video…",
  },

  modelDownload: {
    title: "Getting ready",
    body: (sizeMb: number) =>
      `Offcut is downloading what it needs to write your captions (about ${String(sizeMb)} MB). This happens only the first time, and your video is not uploaded.`,
    progress: (etaSecs: number) => `About ${waitText(etaSecs)} left.`,
    verifying: "Checking the download…",
  },

  // The processing card (v4implementation D-188): four steps in plain words.
  processing: {
    title: "Making your short",
    steps: { audio: "Preparing audio", voice: "Cleaning your voice", captions: "Writing captions", visuals: "Finding visual moments" },
    reassure: "Everything runs on your computer. This usually takes a couple of minutes.",
  },
  cancel: { button: "Cancel", cancelled: "Cancelled.", processAgain: "Process again" },

  // The processing feed (PS §10 J6). A line is shown for something that
  // happened, and for nothing else. `display` is the event's own figure; a
  // FromTo has two, its start and its end (v3implementation D-83).
  feed: {
    transcribing: "Transcribing…",
    cleaningVoice: "Cleaning voice",
    eventFound: {
      number_reveal: (display: string) => `Found: ${display}`,
      list_reveal: (display: string) => `Found: ${display}-item list`,
      // An empty `displayTo` is for type completeness only: no line is made without one.
      from_to: (display: string, displayTo: string) =>
        displayTo === "" ? `Found: ${display}` : `Found: ${display} to ${displayTo}`,
      keyword_pop: (display: string) => `Found: ${display}`,
    } satisfies Record<Exclude<EventKind, "from_to">, (display: string) => string> & {
      from_to: (display: string, displayTo: string) => string;
    },
  },

  preview: {
    play: "Play", pause: "Pause", replay: "Play again",
    canvas: "Preview of your video", seek: "Position",
    loadingFrame: "Loading the frame…",
    clock: (ms: number) => clockTime(ms),
    time: (positionMs: number, lengthMs: number) => `${clockTime(positionMs)} / ${clockTime(lengthMs)}`,
    watermarkBadge: "Free exports include a small watermark",
  },

  // `button` is the top bar's Export (v4implementation §17.5's `editor.export`).
  export: {
    button: "Export",
    unavailable: "Exporting needs an account, and accounts are not open yet.",
    stage: { rendering: "Rendering video", muxing: "Finishing file", saving: "Saving" },
    elapsed: (seconds: number) => `${String(seconds)} ${seconds === 1 ? "second" : "seconds"} so far.`,
    renderingLocally: "Everything is rendering on your computer.",
    done: "Your video is ready. Your browser has downloaded it.",
    percent: (percent: number) => `${String(percent)}%`,
    timeLeft: (seconds: number) => `About ${waitText(seconds)} left`, downloadAgain: "Download again",
  },

  // The editor's top bar and side panel (v4implementation D-178, D-184, §17.5).
  editor: {
    failedStub: "Something went wrong on this device.",
    startOver: "Start over",
    mark: "Offcut",
    lengthLabel: (ms: number) => `Video length ${clockTime(ms)}`,
    saved: "Saved on this device", saving: "Saving…",
    undo: "Undo", redo: "Redo",
    availableAfterExport: "Available when the export finishes",
  },
  panel: { label: "Edit your video", captions: "Captions", style: "Style", visuals: "Visuals", framing: "Framing" },
  captions: {
    hint: "Click any word to fix it.",
    follow: "Follow playback", followHint: "The text scrolls with the video.",
    hide: "Hide word", restore: "Restore original", show: "Show", wordMenu: "Word options",
    replaceAll: (n: number, from: string, to: string) => `Change all ${String(n)} “${from}” to “${to}”?`,
  },
  style: {
    clean: "Clean", bold: "Bold", mono: "Mono",
    cleanLine: "Soft and premium: the spoken word on a highlight.",
    boldLine: "Big and loud: capitals, and each word pops as it is said.",
    monoLine: "Calm and precise: a typewriter look on a dark panel.",
    highlightNumbers: "Highlight numbers",
    highlightNumbersHint: "Numbers in your captions get the accent colour.",
  },
  visuals: {
    count: (on: number, total: number) => `${String(on)} of ${String(total)} visuals on`,
    kind: { number_reveal: "Big number", list_reveal: "List", from_to: "Before → after", keyword_pop: "Emphasis" } satisfies
      Record<EventKind, string>,
    // PS §10 J7.
    noEventsFound: "No numbers or lists found in this clip. Captions and cleanup applied.",
    whatWeLookFor: "Offcut looks for numbers, lists and before-and-after comparisons you say out loud.",
  },
  framing: {
    captionPosition: "Caption position",
    top: "Top", middle: "Middle", bottom: "Bottom",
    dragCaptions: "Or drag the captions on the video.", crop: "Framing",
    cropValue: (offset: number) =>
      offset === 0 ? "Centre" : `${String(Math.round(Math.abs(offset) * PERCENT))}% ${offset < 0 ? "left" : "right"}`,
    dragVideo: "Or drag the video sideways.",
  },

  // The shortcuts sheet (v4implementation D-185); undo and redo are `editor`'s.
  shortcuts: {
    title: "Keyboard shortcuts", open: "Show keyboard shortcuts",
    playPause: "Play or pause", stepFrame: "One frame back or forward", stepSecond: "One second back or forward",
    close: "Close a field or a menu", help: "Show this list",
  },
  // How a shortcut names its keys (`Kbd`, v4implementation D-181).
  keys: {
    modMac: "⌘", modOther: "Ctrl", shift: "Shift", space: "Space", escape: "Esc",
    left: "←", right: "→", z: "Z", y: "Y", question: "?",
  },

  capability: { checking: "Checking your browser…", supported: "Your browser can do this." },

  unsupported: {
    title: "Offcut cannot run in this browser yet",
    supportedList: "Offcut works in Chrome or Edge on Windows or macOS.",
    demoHeading: "See what Offcut makes",
    reasons: {
      UNSUPPORTED_MOBILE: "Offcut does not work on phones and tablets yet.",
      UNSUPPORTED_WEBCODECS: "Your browser cannot encode video here.",
      UNSUPPORTED_THREADS: "Your browser cannot run code on several processor cores here, which Offcut needs.",
      UNSUPPORTED_WASM_SIMD: "Your browser cannot run the fast processing code Offcut needs.",
      UNSUPPORTED_STORAGE:
        "Your browser will not let Offcut keep files on this device. A private window often blocks this.",
      UNSUPPORTED_LOW_MEMORY: `Your device reports less than ${String(LIMITS.MIN_DEVICE_MEMORY_GB)} GB of memory, and Offcut needs at least that much.`,
      UNSUPPORTED_WEBGPU: "Your browser cannot use the graphics processor here.",
      UNSUPPORTED_H264_DECODE: "Your browser cannot decode H.264 video here.",
      UNSUPPORTED_H264_ENCODE: "Your browser cannot encode H.264 video here.",
      UNSUPPORTED_AAC_DECODE: "Your browser cannot decode AAC audio here.",
      UNSUPPORTED_AAC_ENCODE: "Your browser cannot encode AAC audio here.",
    } satisfies Record<UnsupportedReason, string>,
  },

  notifyMe: {
    label: "Email me when Offcut opens",
    labelUnsupported: "Email me when Offcut supports",
    emailLabel: "Email address",
    submit: "Notify me",
    sending: "Sending…",
    success: "Thanks. We will send you one email, when it is ready.",
    invalidEmail: "That does not look like an email address.",
    rateLimited: (seconds: number | undefined) =>
      seconds === undefined
        ? "Too many tries from this network. Please try again later."
        : `Too many tries from this network. Please try again in ${waitText(seconds)}.`,
    offline: "You appear to be offline. Check your connection and try again.",
    unavailable: "We could not save that just now. Please try again in a minute.",
    wanted: { launch: "the launch", safari: "Safari", firefox: "Firefox", mobile: "phones and tablets", linux: "Linux" } satisfies
      Record<Wanted, string>,
  },

  api: { waking: "Connecting to the account service…" },

  // Why an action cannot start now (TS §12.5): the four a person can meet in
  // V2. The others come with the room check and with accounts.
  blockers: {
    B_UNSUPPORTED: "This browser cannot run Offcut, so it cannot take a clip.",
    B_PIPELINE_BUSY: "Offcut is still working on a clip. Add another when it has finished.",
    B_EXPORT_IN_PROGRESS: "An export is running. Wait until it has finished.",
    B_PIPELINE_NOT_READY: "The clip is not ready to export yet.",
  },

  settings: {
    title: "Settings",
    backLink: "Back to Offcut",
    whatLeavesHeading: "What leaves your device",
    columns: { data: "Data", leaves: "Leaves your device?" },
    rows: [
      { data: "Video, audio, transcript, captions, project edits", leaves: "No" },
      { data: "Your email and plan status", leaves: "Yes, to our servers" },
      { data: "Count of exports (no content)", leaves: "Yes" },
      {
        data: "Anonymous usage events from a fixed list (for example stage timings and error codes)",
        leaves: "Yes, and the whole list is shown below",
      },
    ],
    eventListHeading: "The anonymous usage events, in full",
    eventColumns: { event: "Event", description: "What it tells us" },
    events: {
      landing_view: "Landing page opened",
      capability_check: "Browser check",
      clip_accepted: "Clip accepted",
      clip_rejected: "Clip not usable",
      model_download: "Speech model download",
      stage_timing: "Processing stage timing",
      pipeline_done: "Processing finished",
      preview_played: "Preview played",
      review_action: "Change made in review",
      export_started: "Export started",
      export_done: "Export finished",
      export_failed: "Export failed",
      job_cancelled: "Job cancelled",
      post_export_answer: "Answer after an export",
      upgrade_prompt: "Upgrade prompt",
      signin_step: "Sign-in step",
      checkout_step: "Checkout step",
      client_error: "Error in the app",
      local_data_cleared: "Local data cleared",
    } satisfies Record<EventName, string>,
  },

  errors: {
    E_NET_OFFLINE: {
      title: "You are offline",
      body: "Offcut could not reach the account service because this device has no connection.",
      action: "Check your connection and try again.",
    },
    E_NET_TIMEOUT: {
      title: "The account service did not answer",
      body: "It can take about a minute to start when nobody has used it for a while.",
      action: "Try again.",
    },
    E_API_5XX: {
      title: "The account service had a problem",
      body: "The request failed on our side, not yours.",
      action: "Try again in a minute.",
    },
    E_API_RATE_LIMITED: {
      title: "Too many requests",
      body: "You have sent this several times in a short while.",
      action: "Wait a little and try again.",
    },
    E_INTERNAL: {
      title: "Something went wrong in Offcut",
      body: "This is a fault in Offcut, not in your clip or your browser.",
      action: "Reload the page.",
    },
    E_WORKER_CRASH: {
      title: "Part of Offcut stopped working",
      body: "A background task could not start, or stopped unexpectedly.",
      action: "Reload the page and try again.",
    },
    E_MODEL_DOWNLOAD: {
      title: "The speech model could not be downloaded",
      body: "The download stopped after several tries. What has arrived is kept, so the next try goes on from there.",
      action: "Check your connection and try again.",
    },
    E_MODEL_HASH: {
      title: "The speech model did not arrive intact",
      body: "A downloaded file was damaged on the way, so Offcut removed it.",
      action: "Try again, and Offcut downloads it once more.",
    },
    E_MODEL_STORAGE: {
      title: "There is no room for the speech model",
      body: "Your browser has no space left on this device for the model's files.",
      action: "Free some disk space and try again.",
    },
  } satisfies Record<ErrorCodeWithCopy, ErrorCopy>,

  // A file Offcut cannot use (TS §15.3, PS §9.4): what the limit is and what
  // to do, never the measured value (v3implementation D-80).
  rejections: {
    REJECT_FILE_SIZE: {
      title: "This file is too large",
      body: `Offcut takes files up to ${String(LIMITS.MAX_FILE_SIZE / BYTES_PER_MB)} MB.`,
      action: "Trim the clip or export it at a lower resolution, then drop it again.",
    },
    REJECT_CONTAINER: {
      title: "Offcut cannot open this kind of file",
      body: "Offcut takes MP4 and MOV files with H.264 video and AAC audio.",
      action: "Save or export the clip as an MP4, then drop it again.",
    },
    REJECT_CORRUPT: {
      title: "This file looks damaged",
      body: "Offcut could not read the inside of this file. It may be incomplete.",
      action: "Export the clip again from your camera or editor, then drop the new file.",
    },
    REJECT_NO_VIDEO: {
      title: "This file has no video",
      body: "Offcut makes a short from a video of you talking, and this file holds sound only.",
      action: "Drop the video file instead.",
    },
    REJECT_HEVC: {
      title: "This video uses HEVC",
      body: "This file uses HEVC, which this browser cannot decode here.",
      action: 'Export it as H.264, or choose "Most Compatible" in your camera settings and record again.',
    },
    REJECT_VIDEO_CODEC: {
      title: "This video format is not supported",
      body: "Offcut reads H.264 video, and this file uses another format.",
      action: "Export the clip as H.264, then drop it again.",
    },
    REJECT_NO_AUDIO: {
      title: "This video has no sound",
      body: "Offcut needs your voice to write the captions, and this file has no audio track.",
      action: "Record with the microphone on, then drop the new file.",
    },
    REJECT_MULTI_AUDIO_TRACK: {
      title: "This video has more than one audio track",
      body: "Offcut takes clips with one audio track.",
      action: "Export the clip with a single audio track, then drop it again.",
    },
    REJECT_AUDIO_CODEC: {
      title: "This audio format is not supported",
      body: "Offcut reads AAC audio, and this file uses another format.",
      action: "Export the clip with AAC audio, then drop it again.",
    },
    REJECT_DURATION: {
      title: "This clip is too long",
      body: `Offcut handles clips up to ${String(LIMITS.MAX_CLIP_DURATION / MS_PER_SECOND)} seconds right now.`,
      action: "Trim it in your phone's gallery or in your editor, then drop it again.",
    },
    REJECT_RESOLUTION: {
      title: "This video is larger than Offcut handles",
      body: `Offcut handles video up to ${String(LIMITS.MAX_LONG_SIDE)} pixels on its long side.`,
      action: "Record or export at a lower resolution, then drop it again.",
    },
    REJECT_FRAME_RATE: {
      title: "This video has too many frames per second",
      body: `Offcut handles up to ${String(MAX_FPS)} frames per second.`,
      action: `Record or export at ${String(MAX_FPS)} frames per second or fewer, then drop it again.`,
    },
    REJECT_DECODE_UNSUPPORTED: {
      title: "This browser cannot play this video",
      body: "The video uses settings this browser cannot decode.",
      action: "Export it again with your editor's standard H.264 settings, then drop it again.",
    },
    REJECT_NO_SPEECH: {
      title: "No English speech was found",
      body: "Offcut could not hear enough English words in this clip to write captions.",
      action: "Drop a clip of you talking in English.",
    },
  } satisfies Record<RejectReason, ErrorCopy>,
};
