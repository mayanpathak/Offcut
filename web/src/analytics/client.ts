// The analytics client: a queue of allowlisted events, sent in batches
// (TS §32). `track` takes only the generated `AnalyticsEvent` union, so an
// event or a property that is not on the allowlist does not type-check.
//
// This module must stay unable to read user text: it imports no store and
// nothing from persistence/. The anonymous id is handed to it.

import { type AnalyticsEvent, MAX_EVENTS_PER_BATCH } from "../gen/api";
import type { AnonId } from "../gen/domain";
import { postEvents } from "../net/api-client";

export const FLUSH_INTERVAL_MS = 10_000;
export const FLUSH_AT = 20;

const queue: AnalyticsEvent[] = [];
let anonId: AnonId | undefined;

/**
 * Gives the client its anonymous id and starts the flushes: every 10 s and
 * whenever the page is hidden. Events tracked before this call have waited
 * in the queue. A second call does nothing.
 */
export function initAnalytics(o: { anonId: AnonId }): void {
  if (anonId !== undefined) {
    return;
  }
  anonId = o.anonId;
  setInterval(() => void flush(), FLUSH_INTERVAL_MS);
  document.addEventListener("visibilitychange", () => {
    // The last chance to send before the tab may be closed.
    if (document.visibilityState === "hidden") {
      void flush();
    }
  });
  flushIfFull();
}

/** Queues one event. Never throws and never waits. */
export function track(event: AnalyticsEvent): void {
  queue.push(event);
  flushIfFull();
}

function flushIfFull(): void {
  if (queue.length >= FLUSH_AT) {
    void flush();
  }
}

/**
 * Sends the oldest queued events, at most one batch. Does nothing before
 * `initAnalytics`. The events of a batch that fails are dropped, not retried.
 */
export async function flush(): Promise<void> {
  if (anonId === undefined || queue.length === 0) {
    return;
  }
  const events = queue.splice(0, MAX_EVENTS_PER_BATCH);
  try {
    await postEvents({ anon_id: anonId, events });
  } catch {
    // Dropped by design: analytics must never disturb the app (TS §11.3).
  }
}
