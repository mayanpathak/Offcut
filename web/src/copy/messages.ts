// Every word the app shows to a person (INV-15). No component holds a
// sentence of its own, and no sentence here holds a typed-in number: a number
// comes from the generated limits or from the caller.
//
// Wording rules (TS §25.1 P-12): the privacy claim is the factual one, that
// the video is not uploaded. Legal and certification terms are not used.

import type { AnalyticsEvent, Wanted } from "../gen/api";
import { type ErrorCode, type EventKind, LIMITS, type RejectReason, type UnsupportedReason } from "../gen/domain";

type EventName = AnalyticsEvent["name"];

/** The documents of `/legal/:doc`, in the footer's order. */
export const LEGAL_DOCS = ["terms", "privacy", "refunds"] as const;
export type LegalDoc = (typeof LEGAL_DOCS)[number];

type ErrorCopy = { title: string; body: string; action: string };
/**
 * The error codes that can reach a person: the six of V1, the three of the
 * model download (V2), and the twelve more a person can meet on this device
 * since v4implementation D-159, and the account's four (V6). The billing and
 * entitlement codes get copy with their flows.
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
  | "E_STORAGE_QUOTA"
  | "E_STORAGE_IO"
  | "E_DECODE_AUDIO"
  | "E_DECODE_VIDEO"
  | "E_ASR_RUNTIME"
  | "E_ASR_OOM"
  | "E_DSP"
  | "E_GPU_INIT"
  | "E_GPU_LOST"
  | "E_ENCODE_VIDEO"
  | "E_ENCODE_AUDIO"
  | "E_MUX"
  | "E_AUTH_LINK_INVALID"
  | "E_AUTH_LINK_EXPIRED"
  | "E_AUTH_SESSION_EXPIRED"
  | "E_AUTH_EMAIL_UNAVAILABLE"
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
    canvas: "Preview of your video", canvasWords: "Its words are in the Captions tab, where each can be fixed.", seek: "Position",
    loadingFrame: "Loading the frame…",
    clock: (ms: number) => clockTime(ms),
    time: (positionMs: number, lengthMs: number) => `${clockTime(positionMs)} / ${clockTime(lengthMs)}`,
    watermarkBadge: "Free exports include a small watermark",
    captionHandle: "Caption position", captionAt: (percent: number) => `${String(percent)}% of the way down`,
  },

  // `button` is the top bar's Export (v4implementation §17.5's `editor.export`).
  export: {
    button: "Export",
    unavailable: "Exporting needs an account, and accounts are not open yet.",
    stage: { rendering: "Rendering video", muxing: "Finishing file", saving: "Saving" },
    renderingLocally: "Everything is rendering on your computer.",
    done: "Your video is downloaded.",
    percent: (percent: number) => `${String(percent)}%`,
    timeLeft: (seconds: number) => `About ${waitText(seconds)} left`, downloadAgain: "Download again",
    // The pill's name for assistive technology (D-189).
    label: "Export",
    // What the person is told once the file is theirs (PS §10 J11; v5implementation §14).
    summary: {
      title: "What we changed", close: "Close", voice: "Voice cleaned",
      emphasized: (n: number) => (n === 1 ? "1 word emphasized" : `${String(n)} words emphasized`),
      visuals: (n: number) => (n === 1 ? "1 visual moment" : `${String(n)} visual moments`),
    },
    question: { title: "Would you post this as is?", yes: "Yes", smallEdits: "With small edits", no: "No", thanks: "Thanks for telling us." },
  },

  // The one action of an error card, chosen by where the failure happened (D-190).
  recovery: {
    tryAgain: "Try again", reloadPreview: "Reload preview", undoChange: "Undo this change", close: "Close",
    details: "Details",
  },

  // The editor's top bar and side panel (v4implementation D-178, D-184, §17.5).
  editor: {
    failedStub: "Something went wrong on this device.",
    startOver: "Start over",
    mark: "Offcut",
    lengthLabel: (ms: number) => `Video length ${clockTime(ms)}`,
    saved: "Saved on this device", saving: "Saving…",
    undo: "Undo", redo: "Redo",
    // The toast after an undo or a redo (D-184), with the other one as its action.
    undone: "Change undone.", redone: "Change redone.",
    availableAfterExport: "Available when the export finishes",
  },
  panel: { label: "Edit your video", captions: "Captions", style: "Style", visuals: "Visuals", framing: "Framing" },
  captions: {
    hint: "Click any word to fix it.", changeAll: "Change all",
    follow: "Follow playback", followHint: "The text scrolls with the video.",
    hide: "Hide word", restore: "Restore original", show: "Show", wordMenu: "Word options", wordField: "Fix this word",
    replaceAll: (n: number, from: string, to: string) => `Change all ${String(n)} “${from}” to “${to}”?`,
    hidden: (word: string) => `“${word}” hidden.`,
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
    count: (on: number, total: number) => `${String(on)} of ${String(total)} visuals on`, goTo: (time: string) => `Go to ${time}`,
    fromTo: (from: string, to: string) => `${from} → ${to}`, points: (n: number) => `${String(n)} ${n === 1 ? "point" : "points"}`,
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
    close: "Close a field or a menu", help: "Show this list", dismiss: "Close",
  },
  // How a shortcut names its keys (`Kbd`, v4implementation D-181).
  keys: {
    modMac: "⌘", modOther: "Ctrl", shift: "Shift", space: "Space", escape: "Esc",
    left: "←", right: "→", z: "Z", y: "Y", question: "?", or: "or",
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

  // Signing in (PS §10 J9, TS C-8). A link opened on a phone would leave this tab waiting, so the sent state says where to open it.
  signIn: {
    title: "Sign in to export", close: "Close", emailLabel: "Email address", submit: "Send me a sign-in link", sending: "Sending…",
    limits: (freePerMonth: number, creatorMonthlyUsd: number) =>
      `Create a free account to export. Free: ${String(freePerMonth)} exports a month, 720p, small watermark. Creator ($${String(creatorMonthlyUsd)}/month): no watermark, 1080p.`,
    sentTitle: "Check your email",
    sentBody: (email: string) => `We sent a sign-in link to ${email}. Open it on this computer, in this browser.`,
    spamHint: "Not there after a minute? Look in your spam folder.", resend: "Send it again",
  },
  callback: {
    verifying: "Signing you in…", signedIn: "You are signed in",
    returnToTab: "If you have a video open in another tab, go back to it: that tab is signed in too.", openApp: "Go to Offcut",
  },
  account: { title: "Account", signedIn: "You are signed in.", signOut: "Sign out" },

  // Why an action cannot start now (TS §12.5): the four a person can meet in
  // V2. The others come with the room check and with accounts.
  blockers: {
    B_UNSUPPORTED: "This browser cannot run Offcut, so it cannot take a clip.",
    B_PIPELINE_BUSY: "Offcut is still working on a clip. Add another when it has finished.",
    B_EXPORT_IN_PROGRESS: "An export is running. Wait until it has finished.",
    B_PIPELINE_NOT_READY: "The clip is not ready to export yet.",
    B_STORAGE_LOW: "There is not enough room on this device for this video. Free some disk space, then try again.",
  },

  // The landing page's prices (PS §11). Nothing is on sale until accounts open,
  // so the section says so and has no way to buy. It lists only what the plans
  // will hold when they do.
  pricing: {
    heading: "Pricing",
    notOnSale: "Not on sale yet. These plans start when accounts open: leave your email below to hear when they do.",
    free: {
      name: "Free",
      price: (usd: number) => `$${String(usd)}`,
      features: (limits: Pick<typeof LIMITS, "FREE_EXPORTS_PER_MONTH">) => [
        `${String(limits.FREE_EXPORTS_PER_MONTH)} exports a month`,
        "720p, with a small watermark",
      ],
    },
    creator: {
      name: "Creator",
      price: (monthlyUsd: number, annualUsd: number) => `$${String(monthlyUsd)}/month or $${String(annualUsd)}/year`,
      features: ["Unlimited exports", "1080p, no watermark"],
    },
    merchant: "Prices are in US dollars. Dodo Payments, our merchant of record, will handle payment and tax.",
  },

  // Written to match what the app does today, and read by the founder before
  // each change (buildplan.md §10.3). Not reviewed by a lawyer. The wording rules
  // at the top of this file apply.
  legal: {
    updated: "Last updated 2 November 2026.",
    contactLead: "Questions? Write to",
    contactEmail: "ranupathak1986@gmail.com",
    footerContact: "Contact",
    docs: {
      terms: {
        title: "Terms of Service",
        linkLabel: "Terms",
        sections: [
          {
            heading: "These terms",
            body: ["These terms cover your use of Offcut, a tool that turns a talking-head video into a finished vertical short. By using Offcut you agree to them. Offcut is in development, and parts of it are not open yet."],
          },
          {
            heading: "Your videos",
            body: ["Your videos and the shorts you make stay yours. Offcut processes them on your computer. You are responsible for having the right to use what you put in and what you publish."],
          },
          {
            heading: "Plans and payment",
            body: [
              "The plans and prices are on the home page. They start when accounts open, and until then nothing can be bought.",
              "Dodo Payments will be the merchant of record: it sells the subscription to you, takes the payment and handles tax. A subscription renews at the end of each period until you cancel it. The Refund and Cancellation Policy says what happens then.",
              "The limits of a plan are part of how it is priced. We may change plans or prices for the future, and we will tell subscribers before a change reaches them.",
            ],
          },
          {
            heading: "Fair use",
            body: ["Do not use Offcut to break the law, to harm others, or to attack or overload the service. We may suspend an account that does."],
          },
          {
            heading: "No guarantee",
            body: ["Offcut is provided as it is. Captions and visuals are made automatically and can be wrong, so check a short before you publish it. As far as the law allows, we are not liable for indirect or consequential loss, and our total liability is limited to what you paid us in the 12 months before the claim."],
          },
          {
            heading: "Changes",
            body: ["We may update these terms, and this page will show the date of the latest version. If you keep using Offcut after a change, the new terms apply."],
          },
        ],
      },
      privacy: {
        title: "Privacy Policy",
        linkLabel: "Privacy",
        sections: [
          {
            heading: "Your video",
            body: ["Your video is not uploaded to our servers. Offcut processes it in your browser, on your computer, and keeps the video, audio, transcript, captions and edits in that browser's storage. You can remove them by clearing this site's data in your browser."],
          },
          {
            heading: "What we collect today",
            body: [
              "If you join the waitlist: your email address and what you are waiting for. We use it only to tell you when Offcut opens. Write to us to have it removed.",
              "Anonymous usage events from a fixed list, such as how long a stage took or which error occurred, with a random id kept in your browser. They hold no file name, no words from your video and no account link, and we delete them after 90 days. The whole list is on the \"What leaves your device\" page.",
              "To limit abuse, the server reads your IP address while it answers a request. We do not store it with these events or write it to our logs. Our hosting providers may keep their own short-lived logs.",
            ],
          },
          {
            heading: "What we will collect when accounts open",
            body: ["Your email address, your plan and subscription status, and a count of your exports, with no content. You will be able to export this data or delete your account from your account page. Dodo Payments will handle payment, so we will not see your card details, and it may keep its own billing records for tax reasons."],
          },
          {
            heading: "Who handles it",
            body: ["We use Vercel to serve the site, Render to run the server, Neon for the database and Cloudflare to serve the files the app downloads. When accounts open, Brevo will send sign-in emails and Dodo Payments will handle payment. We do not sell your data, show ads or use third-party trackers."],
          },
          {
            heading: "Changes",
            body: ["We will update this page before any new kind of data leaves your computer."],
          },
        ],
      },
      refunds: {
        title: "Refund and Cancellation Policy",
        linkLabel: "Refunds",
        sections: [
          {
            heading: "Before accounts open",
            body: ["Nothing can be bought yet, so nothing can be charged. This policy applies once plans go on sale."],
          },
          {
            heading: "Refunds",
            body: ["If Offcut is not right for you, write to us within 14 days of a payment and we will refund that payment in full. A refund ends your paid plan. Dodo Payments, the merchant of record, returns the money to your original payment method. After 14 days a payment is not refundable, except where the law says otherwise."],
          },
          {
            heading: "Cancelling",
            body: ["You can cancel at any time from your account page. You keep your paid plan until the end of the period you paid for, then your account moves to the Free plan. Nothing is charged after that."],
          },
        ],
      },
    },
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
    // The fourteen of D-159 below, in the words of D-190: what happened in
    // one sentence, then what to do. The card's button is the action itself.
    E_INTERNAL: {
      title: "Something went wrong in Offcut",
      body: "This is a fault in Offcut, not in your clip or your browser.",
      action: "If it happens again, reload the page.",
    },
    E_WORKER_CRASH: {
      title: "Part of Offcut stopped working",
      body: "A task running in the background stopped unexpectedly.",
      action: "If it happens again, reload the page.",
    },
    E_STORAGE_QUOTA: {
      title: "Your browser is out of space",
      body: "Offcut keeps your clip on this device, and the browser has no room left for it.",
      action: "Free some disk space, then try again.",
    },
    E_STORAGE_IO: {
      title: "Offcut could not save on this device",
      body: "Your browser did not let Offcut read or write its files just now.",
      action: "If it happens again, reload the page.",
    },
    E_DECODE_AUDIO: {
      title: "The sound of this clip could not be read",
      body: "Part of the audio in this file could not be played back in this browser.",
      action: "If it happens again, export the clip again from your editor and drop the new file.",
    },
    E_DECODE_VIDEO: {
      title: "The picture of this clip could not be read",
      body: "Part of the video in this file could not be played back in this browser.",
      action: "If it happens again, export the clip again from your editor and drop the new file.",
    },
    E_ASR_RUNTIME: {
      title: "The captions could not be written",
      body: "The speech model stopped while it was listening to your clip.",
      action: "If it happens again, reload the page.",
    },
    E_ASR_OOM: {
      title: "Your computer ran out of memory",
      body: "Writing the captions needed more memory than the browser could give.",
      action: "Close other tabs and apps, then try again.",
    },
    E_DSP: {
      title: "Your voice could not be cleaned",
      body: "Something went wrong while the sound of your clip was being prepared.",
      action: "If it happens again, reload the page.",
    },
    E_GPU_INIT: {
      title: "The preview could not start",
      body: "Offcut could not use your graphics card to draw the video.",
      action: "Close other apps that show video or games, then try again.",
    },
    E_GPU_LOST: {
      title: "The preview stopped",
      body: "Your graphics card was reset, which can happen when another app uses it.",
      action: "Reload preview",
    },
    E_ENCODE_VIDEO: {
      title: "The video file could not be made",
      body: "Your browser stopped while it was turning your edit into a video.",
      action: "Close other tabs, then try again.",
    },
    E_ENCODE_AUDIO: {
      title: "The sound could not be added to the file",
      body: "Your browser stopped while it was writing the audio of your video.",
      action: "If it happens again, reload the page.",
    },
    E_MUX: {
      title: "The video file could not be finished",
      body: "The picture and the sound were ready, but joining them into one file failed.",
      action: "If it happens again, reload the page.",
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
    E_AUTH_LINK_INVALID: { title: "This sign-in link does not work", body: "It has been used already, or a newer link replaced it.", action: "Ask for a new link below." },
    E_AUTH_LINK_EXPIRED: { title: "This sign-in link has expired", body: "A link works for a short while after it is sent.", action: "Ask for a new link below." },
    E_AUTH_SESSION_EXPIRED: { title: "You have been signed out", body: "Your session has ended.", action: "Sign in again." },
    E_AUTH_EMAIL_UNAVAILABLE: { title: "We could not send the email", body: "The email service is not taking messages just now.", action: "Try again in a little while." },
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
