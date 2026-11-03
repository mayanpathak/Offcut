// The token on the page and its claims (v6implementation D-292).

import { beforeEach, describe, expect, it } from "vitest";

import { decodeClaims, setOutboxFree, setToken, useEntitlementStore } from "./entitlement-store";

const CLAIMS = {
  v: 1,
  sub: "0190f3a2-7b1c-7def-8a55-0123456789ab",
  plan: "free",
  free_exports_remaining: 2,
  period_end: 1_900_000_000,
  iat: 1_800_000_000,
  exp: 1_800_600_000,
};

/** As `sign_token` writes it: base64url without padding, then the signature. */
function tokenOf(claims: unknown): string {
  const payload = btoa(JSON.stringify(claims)).replaceAll("+", "-").replaceAll("/", "_").replaceAll("=", "");
  return `${payload}.signature`;
}

describe("decodeClaims", () => {
  it("reads the seven fields of a token", () => {
    expect(decodeClaims(tokenOf(CLAIMS))).toEqual(CLAIMS);
  });

  it("reads a payload whose base64url uses - and _", () => {
    // These bytes encode to a form holding both characters of the URL alphabet.
    const token = tokenOf({ ...CLAIMS, sub: "???>>>~~~" });
    expect(token.split(".")[0]).toMatch(/[-_]/);
    expect(decodeClaims(token)?.sub).toBe("???>>>~~~");
  });

  it("is null for what cannot be read as claims, and never throws", () => {
    const withoutExp: Partial<typeof CLAIMS> = { ...CLAIMS };
    delete withoutExp.exp;
    for (const token of [
      "",
      ".",
      "not a token",
      "%%%.sig",
      tokenOf("a string"),
      tokenOf(null),
      tokenOf(withoutExp),
      tokenOf({ ...CLAIMS, plan: "pro" }),
      tokenOf({ ...CLAIMS, free_exports_remaining: "2" }),
      tokenOf({ ...CLAIMS, sub: 7 }),
    ]) {
      expect(decodeClaims(token), token).toBeNull();
    }
  });
});

describe("the store", () => {
  beforeEach(() => {
    useEntitlementStore.setState(useEntitlementStore.getInitialState(), true);
  });

  it("holds a token with its claims", () => {
    const token = tokenOf(CLAIMS);
    setToken(token);
    expect(useEntitlementStore.getState()).toMatchObject({ token, claims: CLAIMS });
  });

  it("an undecodable token counts as none (D-292), and replaces the one held", () => {
    setToken(tokenOf(CLAIMS));
    setToken("garbage");
    expect(useEntitlementStore.getState()).toMatchObject({ token: null, claims: null });
  });

  it("forgets the token on null, and keeps the outbox's mirror", () => {
    setOutboxFree({ someone: 1 });
    setToken(tokenOf(CLAIMS));
    setToken(null);
    expect(useEntitlementStore.getState()).toEqual({ token: null, claims: null, outboxFree: { someone: 1 } });
  });
});
