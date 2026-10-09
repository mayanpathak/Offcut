// Every word the app shows to a person (INV-15). No component holds a
// sentence of its own, and no sentence here holds a typed-in number: a number
// comes from the generated limits or from the caller.
//
// Wording rules (TS §25.1 P-12): the privacy claim is the factual one, that
// the video is not uploaded. Legal and certification terms are not used.

import type { AnalyticsEvent, Wanted } from "../gen/api";
import { type ErrorCode, LIMITS, type UnsupportedReason } from "../gen/domain";

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

/** "45 seconds", "1 minute", "12 minutes": a wait, rounded up to whole minutes from one minute on. */
function waitText(seconds: number): string {
  if (seconds < SECONDS_PER_MINUTE) {
    const whole = Math.max(Math.ceil(seconds), 1);
    return `${String(whole)} ${whole === 1 ? "second" : "seconds"}`;
  }
  const minutes = Math.ceil(seconds / SECONDS_PER_MINUTE);
  return `${String(minutes)} ${minutes === 1 ? "minute" : "minutes"}`;
}

export const messages = {
  landing: {
    hero: "Turn what you say into a finished short.",
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
      `Drop a clip (up to ${String(limits.MAX_CLIP_DURATION / MS_PER_SECOND)} seconds, English).`,
    sampleButton: "Try with a sample clip",
    notReady: "Offcut cannot process clips yet. Join the waitlist and we will email you when it can.",
  },

  modelDownload: {
    body: (sizeMb: number) =>
      `One-time setup: downloading the speech model (about ${String(sizeMb)} MB). It stays in your browser, and your video is not uploaded.`,
    progress: (etaSecs: number) => `About ${waitText(etaSecs)} left.`,
    verifying: "Checking the download…",
  },

  capability: {
    checking: "Checking your browser…",
    supported: "Your browser can do this.",
  },

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
    wanted: {
      launch: "the launch",
      safari: "Safari",
      firefox: "Firefox",
      mobile: "phones and tablets",
      linux: "Linux",
    } satisfies Record<Wanted, string>,
  },

  api: {
    waking: "Connecting to the account service…",
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
};
