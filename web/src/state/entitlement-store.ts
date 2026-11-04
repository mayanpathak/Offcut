// The entitlement token on this page and what it says (v6implementation D-292).
// The claims are read without verifying, for display and blockers only: what an
// export is allowed comes from the worker's verification alone (INV-9).

import { create } from "zustand";

import type { Plan, UnixSecs, UserId } from "../gen/domain";

/** The field names of `EntitlementClaims` in Rust (TS §10.6). */
export type EntitlementClaims = {
  v: number;
  sub: UserId;
  plan: Plan;
  free_exports_remaining: number;
  period_end: UnixSecs;
  iat: UnixSecs;
  exp: UnixSecs;
};

export type EntitlementState = {
  token: string | null;
  claims: EntitlementClaims | null;
  /** The Free rows of the outbox, per account: `receipt-outbox.ts` mirrors them here, so `effective()` can be read while drawing. */
  outboxFree: Readonly<Record<string, number>>;
};

export const useEntitlementStore = create<EntitlementState>(() => ({ token: null, claims: null, outboxFree: {} }));

const isUserId = (value: unknown): value is UserId => typeof value === "string";
const isSecs = (value: unknown): value is UnixSecs => typeof value === "number";
const isPlan = (value: unknown): value is Plan => value === "free" || value === "creator";

function isClaims(value: unknown): value is EntitlementClaims {
  return (
    typeof value === "object" &&
    value !== null &&
    "v" in value &&
    typeof value.v === "number" &&
    "sub" in value &&
    isUserId(value.sub) &&
    "plan" in value &&
    isPlan(value.plan) &&
    "free_exports_remaining" in value &&
    typeof value.free_exports_remaining === "number" &&
    "period_end" in value &&
    isSecs(value.period_end) &&
    "iat" in value &&
    isSecs(value.iat) &&
    "exp" in value &&
    isSecs(value.exp)
  );
}

/** The claims of a token, or `null` if it cannot be read as one. */
export function decodeClaims(token: string): EntitlementClaims | null {
  try {
    const payload = (token.split(".")[0] ?? "").replaceAll("-", "+").replaceAll("_", "/");
    const claims: unknown = JSON.parse(atob(payload));
    return isClaims(claims) ? claims : null;
  } catch {
    // Not base64, or not JSON: as good as no token at all (D-292).
    return null;
  }
}

/** Takes a token, or forgets the one held. A token that cannot be read counts as none (D-292). */
export function setToken(token: string | null): void {
  const claims = token === null ? null : decodeClaims(token);
  useEntitlementStore.setState({ token: claims === null ? null : token, claims });
}

export function setOutboxFree(outboxFree: Readonly<Record<string, number>>): void {
  useEntitlementStore.setState({ outboxFree });
}
