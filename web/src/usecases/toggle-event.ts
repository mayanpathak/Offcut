// A suggested visual on or off (v4implementation D-145, D-187): the user's
// choice for the plan item `id`, kept in `event_overrides` and through the
// update rule in mode `scene`. `review_action` carries the kind and the
// choice, never the words (INV-18).

import type { EventId } from "../gen/domain";
import { useClipStore } from "../state/clip-store";
import { update } from "./review-update";

/** Shows (`on`) or hides the item `id`. The choice the clip has already changes nothing. */
export function toggleEvent(id: EventId, on: boolean): Promise<void> {
  const { plan, edit } = useClipStore.getState();
  const item = plan?.items.find((candidate) => candidate.id === id);
  // Every item of a rules plan is shown until the user says otherwise (D-145).
  if (item === undefined || (edit.event_overrides[id] ?? true) === on) {
    return Promise.resolve();
  }
  return update(
    (current) => ({ ...current, event_overrides: { ...current.event_overrides, [id]: on } }),
    "scene",
    "event_toggle",
    { event_kind: item.kind, enabled: on },
  );
}
