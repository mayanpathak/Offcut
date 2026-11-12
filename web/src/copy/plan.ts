// The words of V6.2's AI plan, kept out of messages.ts, which is near its
// line limit (v6.2implementation D-491); messages.ts imports them as it
// imports long.ts. Drafts of the plan's §12: the founder approves the wording
// at checkpoint 3. P184 adds only the five kinds' names, which the visuals
// list's kind names must cover; the rest comes with its screens (P194).
// D-500: wherever §12 says Google's Gemini, DeepSeek. Open item 35: the
// settings line and row say where the words go, and never that they are not
// kept or not used for training (DeepSeek's terms state neither, D-492).

import type { EventKind } from "../gen/domain";

/** V6.2's five kinds, as the visuals list names them (§12's "Kind names in the list"). */
export const planKindNames = {
  emphasis: "Emphasis",
  icon_pop: "Icon",
  punch_in: "Zoom",
  emoji: "Emoji",
  hook_title: "Hook title",
} as const satisfies Partial<Record<EventKind, string>>;

/**
 * The processing feed's line for each of the five kinds. Type completeness
 * only: the feed lists the detector's finds, and the detector never makes
 * these kinds (D-446).
 */
export const planFoundLines = {
  emphasis: (display: string) => `Found: ${display}`,
  icon_pop: (display: string) => `Found: ${display}`,
  punch_in: (display: string) => `Found: ${display}`,
  emoji: (display: string) => `Found: ${display}`,
  hook_title: (display: string) => `Found: ${display}`,
} satisfies Partial<Record<EventKind, (display: string) => string>>;

/** The line told once when a clip asked for an AI plan and uses the rules (§8.2). */
const fallback = {
  unavailable: "AI plan unavailable: using the built-in rules",
  quota: "This month's AI plans are used: using the built-in rules",
  signedOut: "Sign in to get an AI-planned edit. Using the built-in rules for now.",
  offline: "You're offline: using the built-in rules",
} as const;

const regenerateFailed = "Couldn't make a new plan. Your edit is unchanged.";

/** Why "Regenerate plan" did not change the plan (§8.3: "a toast says why"). New drafts beside §12's, for CP 3. */
const regenerateWhy = {
  signed_out: "Couldn't make a new plan: sign in first. Your edit is unchanged.",
  quota: "Couldn't make a new plan: this month's AI plans are used. Your edit is unchanged.",
  offline: "Couldn't make a new plan: you're offline. Your edit is unchanged.",
} as const;

/** §12's drafts (P194). `messages.plan`. */
export const planCopy = {
  /** `plan.processing`: not shown yet; the wait is inside "Finding visual moments" (V6 CL known issue 8). */
  processing: "Planning the edit",
  fallback,
  /** The fallback line for a reason of the clip's `planFallback`: invalid, timeout and catalog read as unavailable. */
  fallbackLine: (why: string): string =>
    why === "signed_out" ? fallback.signedOut : why === "quota" ? fallback.quota : why === "offline" ? fallback.offline : fallback.unavailable,
  regenerate: "Regenerate plan",
  regenerateFailed,
  regenerateWhy,
  /** The toast of a regeneration that changed nothing, for its reason. */
  regenerateLine: (why: string): string =>
    why === "signed_out" || why === "quota" || why === "offline" ? regenerateWhy[why] : regenerateFailed,
  settings: {
    label: "AI edit plan",
    help: "Sends the words of your transcript and their timings to DeepSeek, in China, to plan the edit. Never your audio or video.",
    /** The row of "What leaves your device", with the flag on (open item 35). */
    row: {
      data: "The words of a clip's transcript and their timings, when \"AI edit plan\" is on (signed in only)",
      leaves:
        "Yes: to our server, which sends them to DeepSeek, in China, to plan the edit, and returns the plan. Our server keeps none of them. Turn \"AI edit plan\" off and nothing is sent",
    },
  },
  extras: {
    title: "Text for your post",
    hook: "Hook",
    caption: "Post caption",
    hashtags: "Hashtags",
    copy: "Copy",
    copied: "Copied",
    /** The Copy button's name for a screen reader: what it copies. */
    copyNamed: (what: string) => `Copy ${what.toLowerCase()}`,
  },
  /** §12's Enhance and Glow drafts, for their screens (P201, P208). */
  enhance: {
    title: "Enhance",
    voice: {
      label: "Voice cleanup",
      help: "Removes noise and evens out your voice. Off: your original sound, at the same loudness.",
    },
    motion: {
      label: "Motion",
      help: "A slow camera move on every shot. Off: the shot stays still; the plan's zooms stay.",
    },
    reset: "Back to recommended",
  },
  glow: "Glow",
} as const;
