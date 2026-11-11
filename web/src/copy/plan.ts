// The words of V6.2's AI plan, kept out of messages.ts, which is near its
// line limit (v6.2implementation D-491); messages.ts imports them as it
// imports long.ts. Drafts of the plan's §12: the founder approves the wording
// at checkpoint 3. P184 adds only the five kinds' names, which the visuals
// list's kind names must cover; the rest comes with its screens (P194).

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
