// The Ed25519 public keys an entitlement token may be signed with
// (technicalspec.md §24.2). Public values: this file never holds a private
// key or a seed.

import { env } from "./env";

// The public half of the server's ENTITLEMENT_SIGNING_KEY, as base64. The
// secret scan of CI takes the literal for a secret; the comment tells it not to.
const PRODUCTION_PUBLIC_KEY = "KnENazU2ypDUgG3mibKKiP0g4L5zgrgiWTCeCLNjeLg="; // gitleaks:allow

function decode(base64: string): Uint8Array {
  return Uint8Array.from(atob(base64), (char) => char.charCodeAt(0));
}

/**
 * An array, so that a rotation can ship the new key before the server
 * switches. The test key is in it only in a build made for the tests
 * (v2implementation D-24).
 */
export const ENTITLEMENT_PUBLIC_KEYS: readonly Uint8Array[] = [
  decode(PRODUCTION_PUBLIC_KEY),
  ...(env.entitlementTestPublicKey === null ? [] : [decode(env.entitlementTestPublicKey)]),
];
