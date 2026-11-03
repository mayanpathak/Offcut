// The account of this tab (TS §12.2, v6implementation D-295): unknown →
// anonymous | anonymous_offline | signed_in. Only a verify or a refresh leads
// to `signed_in`: a broadcast alone does not (TS §12.3), so the machine has no
// event for one.

import type { MachineDef } from "./transition";

export type AuthStatus = "unknown" | "anonymous" | "anonymous_offline" | "link_sent" | "signed_in";

export type AuthEvent =
  | "refresh_ok"
  | "refresh_401"
  | "refresh_network"
  | "verify_ok"
  | "link_accepted"
  | "dialog_closed"
  | "signed_out"
  | "session_expired"
  | "account_deleted";

export const AUTH_MACHINE: MachineDef<AuthStatus, AuthEvent> = {
  unknown: {
    refresh_ok: "signed_in",
    refresh_401: "anonymous",
    refresh_network: "anonymous_offline",
    verify_ok: "signed_in",
    link_accepted: "link_sent",
  },
  anonymous: { refresh_ok: "signed_in", verify_ok: "signed_in", link_accepted: "link_sent" },
  anonymous_offline: { refresh_ok: "signed_in", refresh_401: "anonymous", link_accepted: "link_sent" },
  link_sent: { refresh_ok: "signed_in", verify_ok: "signed_in", link_accepted: "link_sent", dialog_closed: "anonymous" },
  signed_in: { signed_out: "anonymous", session_expired: "anonymous", account_deleted: "anonymous" },
};
