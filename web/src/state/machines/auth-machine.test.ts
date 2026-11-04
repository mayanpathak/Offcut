// The auth machine (TS §12.2, §12.3; v6implementation D-295): the table,
// written out here, through `transition` and through the store's actions; the
// callback tab's path; what a broadcast's refresh may meet.

import { beforeEach, describe, expect, it } from "vitest";

import * as auth from "../auth-store";
import { AUTH_MACHINE, type AuthEvent, type AuthStatus } from "./auth-machine";
import { IllegalTransitionError, transition } from "./transition";

const STATES: readonly AuthStatus[] = ["unknown", "anonymous", "anonymous_offline", "link_sent", "signed_in"];
const EVENTS: readonly AuthEvent[] = [
  "refresh_ok",
  "refresh_401",
  "refresh_network",
  "verify_ok",
  "link_accepted",
  "dialog_closed",
  "signed_out",
  "session_expired",
  "account_deleted",
];

/** TS §12.2's table with D-295's rows, row by row. */
const LEGAL: readonly [AuthStatus, AuthEvent, AuthStatus][] = [
  ["unknown", "refresh_ok", "signed_in"],
  ["unknown", "refresh_401", "anonymous"],
  ["unknown", "refresh_network", "anonymous_offline"],
  ["unknown", "verify_ok", "signed_in"],
  ["unknown", "link_accepted", "link_sent"],
  ["anonymous", "refresh_ok", "signed_in"],
  ["anonymous", "verify_ok", "signed_in"],
  ["anonymous", "link_accepted", "link_sent"],
  ["anonymous_offline", "refresh_ok", "signed_in"],
  ["anonymous_offline", "refresh_401", "anonymous"],
  ["anonymous_offline", "link_accepted", "link_sent"],
  ["link_sent", "refresh_ok", "signed_in"],
  ["link_sent", "verify_ok", "signed_in"],
  ["link_sent", "link_accepted", "link_sent"],
  ["link_sent", "dialog_closed", "anonymous"],
  ["signed_in", "signed_out", "anonymous"],
  ["signed_in", "session_expired", "anonymous"],
  ["signed_in", "account_deleted", "anonymous"],
];

/** Each event through the store, as the use-cases fire it. */
const FIRE: Record<AuthEvent, () => void> = {
  refresh_ok: () => {
    auth.refreshOk("token");
  },
  refresh_401: auth.refreshRefused,
  refresh_network: auth.refreshUnreachable,
  verify_ok: () => {
    auth.verified("token");
  },
  link_accepted: () => {
    auth.linkAccepted("someone@example.com");
  },
  dialog_closed: auth.dialogClosed,
  signed_out: auth.signedOut,
  session_expired: auth.refreshRefused,
  account_deleted: auth.accountDeleted,
};

function freshIn(status: AuthStatus): void {
  auth.useAuthStore.setState({ ...auth.useAuthStore.getInitialState(), status }, true);
}

describe("auth machine", () => {
  beforeEach(() => {
    auth.useAuthStore.setState(auth.useAuthStore.getInitialState(), true);
  });

  it("has exactly the table of TS §12.2 with D-295's rows", () => {
    for (const from of STATES) {
      for (const event of EVENTS) {
        const legal = LEGAL.find(([f, e]) => f === from && e === event);
        if (legal === undefined) {
          expect(() => transition("auth", AUTH_MACHINE, from, event), `${from} + ${event}`).toThrow(IllegalTransitionError);
        } else {
          expect(transition("auth", AUTH_MACHINE, from, event)).toBe(legal[2]);
        }
      }
    }
  });

  it.each(LEGAL)("%s --%s--> %s through the store", (from, event, to) => {
    freshIn(from);
    FIRE[event]();
    expect(auth.useAuthStore.getState().status).toBe(to);
  });

  it("the callback tab's path: unknown, verify, signed_in, with no throw (D-295, D-296)", () => {
    expect(auth.useAuthStore.getState().status).toBe("unknown");
    expect(() => {
      auth.verified("access");
    }).not.toThrow();
    expect(auth.useAuthStore.getState()).toMatchObject({ status: "signed_in", accessToken: "access" });
  });

  it("TS §12.3: anonymous reaches signed_in only by a verify or a refresh", () => {
    const leads = EVENTS.filter((event) => AUTH_MACHINE.anonymous[event] === "signed_in");
    expect([...leads].sort()).toEqual(["refresh_ok", "verify_ok"]);
    for (const event of ["signed_out", "session_expired", "dialog_closed", "account_deleted"] as const) {
      expect(() => transition("auth", AUTH_MACHINE, "anonymous", event)).toThrow(IllegalTransitionError);
    }
  });

  it("an action that is illegal throws in a test build and changes nothing", () => {
    freshIn("anonymous");
    expect(auth.signedOut).toThrow(IllegalTransitionError);
    expect(auth.useAuthStore.getState().status).toBe("anonymous");
    freshIn("signed_in");
    expect(() => {
      auth.linkAccepted("someone@example.com");
    }).toThrow(IllegalTransitionError);
    expect(auth.useAuthStore.getState().email).toBeNull();
  });
});

describe("what the store keeps", () => {
  beforeEach(() => {
    auth.useAuthStore.setState(auth.useAuthStore.getInitialState(), true);
  });

  it("a resend stays in link_sent and takes the new address", () => {
    auth.linkAccepted("first@example.com");
    auth.linkAccepted("second@example.com");
    expect(auth.useAuthStore.getState()).toMatchObject({ status: "link_sent", email: "second@example.com" });
  });

  it("a refresh in a tab signed in replaces the token and fires nothing", () => {
    auth.verified("first");
    auth.refreshOk("second");
    expect(auth.useAuthStore.getState()).toMatchObject({ status: "signed_in", accessToken: "second" });
  });

  it("a refused refresh leaves anonymous and link_sent as they are, and throws nothing", () => {
    for (const status of ["anonymous", "link_sent"] as const) {
      freshIn(status);
      auth.refreshRefused();
      expect(auth.useAuthStore.getState().status).toBe(status);
    }
  });

  it("an unreachable refresh says something only from unknown", () => {
    for (const status of ["anonymous", "anonymous_offline", "link_sent", "signed_in"] as const) {
      freshIn(status);
      auth.refreshUnreachable();
      expect(auth.useAuthStore.getState().status).toBe(status);
    }
  });

  it("each way out of signed_in forgets the token and the address", () => {
    for (const leave of [auth.signedOut, auth.accountDeleted, auth.refreshRefused]) {
      freshIn("signed_in");
      auth.useAuthStore.setState({ email: "someone@example.com", accessToken: "token" });
      leave();
      expect(auth.useAuthStore.getState()).toEqual({ status: "anonymous", email: null, accessToken: null });
    }
  });
});
