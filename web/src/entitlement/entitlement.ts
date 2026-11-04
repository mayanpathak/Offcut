// What a person may do now: the token, less the exports it does not know of yet
// (TS §12.1, v6implementation D-267). This is the only place the two meet. It
// waits on nothing, so it can be read while drawing.

import type { Plan, UnixSecs, UserId } from "../gen/domain";

/** The claims `effective` reads, as `entitlement-store` holds them. */
export type EffectiveInput = {
  sub: UserId;
  plan: Plan;
  free_exports_remaining: number;
  exp: UnixSecs;
};

export type Effective = {
  plan: Plan;
  exp: UnixSecs;
  freeExportsRemaining: number;
};

/**
 * `outboxFree` is the number of unsent Free rows per account. Only the token's
 * own account counts: another's rows wait for that account, and a row exported
 * under Creator never spends a Free count, so it is not among them. `null`
 * without a token.
 */
export function effective(claims: EffectiveInput | null, outboxFree: Readonly<Record<string, number>>): Effective | null {
  if (claims === null) {
    return null;
  }
  return {
    plan: claims.plan,
    exp: claims.exp,
    freeExportsRemaining: Math.max(0, claims.free_exports_remaining - (outboxFree[claims.sub] ?? 0)),
  };
}
