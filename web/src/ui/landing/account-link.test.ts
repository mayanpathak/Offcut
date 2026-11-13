// V6.2 (P206a, D-501): the landing top bar's link reads "Sign in" until the
// tab is signed in, then "Account". Its place and its target are read in
// `tests-e2e/landing.spec.ts`.

import { describe, expect, it } from "vitest";

import { messages } from "../../copy/messages";
import type { AuthStatus } from "../../state/machines/auth-machine";
import { accountLinkLabel } from "./account-link";

const NOT_SIGNED_IN: readonly AuthStatus[] = ["unknown", "anonymous", "anonymous_offline", "link_sent"];

describe("the landing page's account link", () => {
  it("reads Account when this tab is signed in", () => {
    expect(accountLinkLabel("signed_in")).toBe(messages.landing.account.open);
  });

  it("reads Sign in in every other state, a link sent and an offline start included", () => {
    for (const status of NOT_SIGNED_IN) {
      expect(accountLinkLabel(status), status).toBe(messages.landing.account.signIn);
    }
  });
});
