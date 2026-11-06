// The landing page's words and its prices' (designimplementation D-321).
// `messages.ts` imports both, as `messages.landing` and `messages.pricing`, so
// every caller and test reads them as before. The rules at the top of
// `messages.ts` hold here too (INV-15): no number is typed into a sentence; a
// count or a price comes from `LIMITS`, `PRICES_USD` or a named constant.
//
// Every claim is true on the day it ships (rule 2.4): what works today is
// said plainly, and what comes after V6 carries "Coming soon", with no date.

import type { LIMITS } from "../gen/domain";

const MS_PER_SECOND = 1000;
const SECONDS_PER_MINUTE = 60;
const TWO_DIGITS = 10;

// The long path's planned limits (PS §20.2A), named for the "Coming soon" lines.
const LONG_VIDEO_MINUTES = 20;
const SHORTS_FROM_LONG_MIN = 5;
const SHORTS_FROM_LONG_MAX = 10;

/** "00:07": a section's place on the page, written as a short's timecode (D-317). */
function timecode(seconds: number): string {
  const minutes = Math.floor(seconds / SECONDS_PER_MINUTE);
  const rest = seconds % SECONDS_PER_MINUTE;
  const pad = (n: number) => `${n < TWO_DIGITS ? "0" : ""}${String(n)}`;
  return `${pad(minutes)}:${pad(rest)}`;
}

/** A section's eyebrow: its timecode and its label. */
function eyebrow(seconds: number, label: string): { time: string; label: string } {
  return { time: timecode(seconds), label };
}

export const landingCopy = {
  // --- Kept from V1-V6, word for word (the tests read them) ---------------------
  hero: "Say it once. Post it finished.",
  subhead:
    "Drop in a raw talking-head clip. Offcut cleans your voice, writes word-by-word captions and animates the numbers and lists you say, then exports a vertical short ready to post.",
  supportedBrowsers: "Works in Chrome or Edge on Windows and Mac.",
  privacyLine: "Your video stays on your computer.",
  whatLeavesLink: "What leaves your device",
  // The unsupported page's caption under the hero clip (D-324).
  demoCaption: "What Offcut makes: a raw take in, a finished short out.",
  // The update list (D-328): the same form and wire as the old waitlist.
  updatesHeading: "Hear when the next features open.",

  // --- The page (designimplementation §6) ---------------------------------------
  /** The word of `hero` the caption pill rests on (M1). */
  heroRestsOn: "finished.",
  wordmark: "Offcut",
  homeLink: "Offcut, back to the top",
  nav: { label: "Sections", features: "Features", how: "How it works", pricing: "Pricing", soon: "Coming soon" },
  cta: { startFree: "Start free", howItWorks: "See how it works" },
  eyebrows: {
    hero: eyebrow(0, "Raw take in"),
    beforeAfter: eyebrow(4, "Before → after"),
    features: eyebrow(7, "What it does"),
    how: eyebrow(21, "How it works"),
    try: eyebrow(30, "Your clip"),
    privacy: eyebrow(38, "On your computer"),
    pricing: eyebrow(44, "Pricing"),
    soon: eyebrow(50, "Coming soon"),
    updates: eyebrow(56, "Stay close"),
    final: eyebrow(60, "Your turn"),
  },
  heroChips: ["Voice cleaned", "Captions, word by word", "Numbers counted up"],
  marquee: [
    "Word-by-word captions",
    "Voice cleanup",
    "Big numbers",
    "Lists",
    "Before → after",
    "Emphasis",
    "9:16 framing",
    "1080p export",
    "Runs in your browser",
  ],
  beforeAfter: {
    heading: "Raw take in. Finished short out.",
    lead: "The same take, the same length: only better to watch.",
    steps: ["Transcribed in seconds", "Voice cleaned", "Captions and visuals added"],
  },
  features: {
    heading: "Everything a talking-head short needs. Nothing it doesn't.",
    lead: "Every card below works today.",
    cards: {
      captions: {
        title: "Captions that follow your voice",
        body: "Word by word, timed to your speech, written in seconds in the cloud or on your computer if you prefer. Three styles: Clean, Bold and Mono. Numbers can take the accent colour.",
        chips: ["Clean", "Bold", "Mono"],
      },
      visuals: {
        title: "Visuals for what you say",
        body: "Say a number, a list or a before-and-after, and Offcut animates it. Turn any of them off.",
        chips: ["Big number", "List", "Before → after", "Emphasis"],
      },
      voice: {
        title: "A cleaner voice, same length",
        body: "Noise reduced, levels evened out and loudness set for phones. Nothing is cut, so your timing stays yours.",
        chips: [],
      },
      framing: {
        title: "Vertical, framed your way",
        body: "Portrait clips pass straight through. Landscape clips are cropped to 9:16, and you pick the crop. Captions go top, middle or bottom, or wherever you drag them.",
        chips: [],
      },
      editor: {
        title: "Fix any word in a click",
        body: "Correct a word, change every copy of it at once, or hide it. Live preview with sound, undo and redo, and keyboard shortcuts.",
        chips: [],
      },
      transcription: {
        title: "Transcribed on your computer",
        body: "A speech model runs in your browser. It downloads once, the first time.",
        chips: [],
      },
      export: {
        title: "Ready to post",
        body: "An MP4 in 1080p on Creator, or 720p with a small watermark on Free, rendered on your computer. Then a short summary of what we changed.",
        chips: ["1080p", "720p", "MP4"],
      },
      account: {
        title: "Sign in with a link",
        body: "No password: we email you a link. See your plan, take your data, or delete your account at any time.",
        chips: [],
      },
    },
    // What the small illustrations of the cards show (decoration, hidden from readers).
    editorMock: { line: "Post it", before: "fished.", after: "finished." },
    transcriptMock: ["say", "it", "once"],
    mailChip: "Your sign-in link",
    fileChip: "short.mp4",
  },
  how: {
    heading: "No timeline to learn.",
    lead: "Offcut makes the edit. You check it, then download it.",
    steps: [
      { title: "Drop", body: "Drop a clip, or try the sample." },
      { title: "Review", body: "Fix a word, pick a style, move the captions." },
      { title: "Export", body: "Download an MP4 ready to post." },
    ],
  },
  try: {
    heading: "Try it now.",
    limits: (limits: Pick<typeof LIMITS, "MAX_CLIP_DURATION">) =>
      `Up to ${String(limits.MAX_CLIP_DURATION / MS_PER_SECOND)} seconds, in English, one speaker. MP4 or MOV.`,
  },
  privacy: {
    heading: "Your video is never uploaded.",
    facts: [
      "It is edited, rendered and exported in your browser, on your computer.",
      "No third-party trackers.",
      "Every anonymous event we count is listed.",
    ],
    fullList: "See the full list",
  },
  soon: {
    heading: "Where Offcut is going.",
    lead: "Built in the open, one release at a time.",
    chip: "Coming soon",
    next: {
      title: "Next",
      items: [
        {
          title: "Long videos",
          line: `Up to ${String(LONG_VIDEO_MINUTES)} minutes and 4K, with transcription that carries on after a closed tab.`,
        },
      ],
    },
    later: {
      title: "On the way",
      items: [
        {
          title: "One long video, many shorts",
          line: `Offcut picks ${String(SHORTS_FROM_LONG_MIN)} to ${String(SHORTS_FROM_LONG_MAX)} shorts from a long video, each with the reason it was picked. Export the ones you keep.`,
        },
        {
          title: "An edit plan from your words",
          line: "Emphasis, emoji, a hook title, a post caption and hashtags, planned from your transcript.",
        },
        { title: "Pauses trimmed", line: "Dead air shortened, never your words. Every cut is shown and can be restored." },
        { title: "Framing that follows you", line: "A speaker who moves stays in frame." },
        {
          title: "The finished look",
          line: "Cards for quotes, comparisons, steps and checklists; a hook title, progress bar and end card; caption presets and shape transitions.",
        },
        { title: "Music", line: "Your own track, lowered automatically under your voice, and sound cues." },
        { title: "B-roll and layouts", line: "Your own B-roll placed for you, transitions at joins, screen-and-face layouts." },
        { title: "Edit by asking", line: "Ask for a change in words. No timeline." },
        {
          title: "Brand kit and memory",
          line: "Your logo, colours and fonts on every short. Offcut remembers what you prefer, and works through your back catalogue.",
        },
        {
          title: "Text behind you, new backgrounds",
          line: "Text behind the speaker and background changes, released only if they meet our quality bar.",
        },
      ],
    },
    footnote: "In development. What ships, and when, may change.",
  },
  final: { line: "Your next short is one take away.", restsOn: "away." },
  footer: { line: "Speech-driven vertical shorts, made in your browser.", legal: "Legal", settingsLink: "Settings" },
  playClip: "Play the clip",
  // One description per clip, for a reader that cannot see it (D-326).
  clipLabels: {
    hero: "A raw talking-head take becomes a vertical short with clean captions.",
    beforeAfter: "A landscape take on the left, the finished vertical short on the right.",
    captionStyles: "The same caption in the three styles: Clean, Bold and Mono.",
    visuals: "A spoken number, a list and a before-and-after, each animated on screen.",
    howItWorks: "The Offcut editor: a clip dropped in, a word fixed, an MP4 downloaded.",
    privacy: "The video stays inside the browser.",
    comingSoon: "The features that are coming, each marked Coming soon.",
  },
};

// The landing page's prices (PS §11). Accounts are open, so Free is; Creator
// goes on sale with the founding release (v6implementation D-303). Until then
// the section has no way to buy, only a way in (D-329, option (a)).
export const pricingCopy = {
  heading: "Pricing",
  notOnSale: "Start on Free with a free account. Creator goes on sale soon.",
  free: {
    name: "Free",
    price: (usd: number) => `$${String(usd)}`,
    features: (limits: Pick<typeof LIMITS, "FREE_EXPORTS_PER_MONTH">) => [
      `${String(limits.FREE_EXPORTS_PER_MONTH)} exports a month`,
      "720p, with a small watermark",
    ],
    everything: "Every feature above",
  },
  creator: {
    name: "Creator",
    price: (monthlyUsd: number, annualUsd: number) => `$${String(monthlyUsd)}/month or $${String(annualUsd)}/year`,
    features: ["Unlimited exports", "1080p, no watermark"],
    tag: "For regular posting",
    soon: "On sale soon",
  },
  merchant: "Prices are in US dollars. Dodo Payments, our merchant of record, will handle payment and tax.",
};
