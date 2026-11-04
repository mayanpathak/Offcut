// `effective()`: the token less this account's unsent Free rows (v6implementation D-267).

import { describe, expect, it } from "vitest";

import type { UnixSecs, UserId } from "../gen/domain";
import { effective, type EffectiveInput } from "./entitlement";

const ME: UserId = "me" as never;
const EXP: UnixSecs = 1_800_000_000 as never;

const claims = (over: Partial<EffectiveInput> = {}): EffectiveInput => ({
  sub: ME,
  plan: "free",
  free_exports_remaining: 3,
  exp: EXP,
  ...over,
});

describe("effective", () => {
  it("is null without a token", () => {
    expect(effective(null, { me: 2 })).toBeNull();
  });

  it("is the token as it is when nothing waits", () => {
    expect(effective(claims(), {})).toEqual({ plan: "free", exp: EXP, freeExportsRemaining: 3 });
  });

  it("subtracts this account's rows", () => {
    expect(effective(claims(), { me: 1 })?.freeExportsRemaining).toBe(2);
    expect(effective(claims(), { me: 3 })?.freeExportsRemaining).toBe(0);
  });

  it("does not subtract another account's rows", () => {
    expect(effective(claims(), { someone_else: 2 })?.freeExportsRemaining).toBe(3);
    expect(effective(claims(), { me: 1, someone_else: 2 })?.freeExportsRemaining).toBe(2);
  });

  it("floors at zero", () => {
    expect(effective(claims({ free_exports_remaining: 1 }), { me: 5 })?.freeExportsRemaining).toBe(0);
    expect(effective(claims({ free_exports_remaining: 0 }), {})?.freeExportsRemaining).toBe(0);
  });

  it("keeps the plan and the expiry of the token", () => {
    expect(effective(claims({ plan: "creator" }), {})).toMatchObject({ plan: "creator", exp: EXP });
  });
});
