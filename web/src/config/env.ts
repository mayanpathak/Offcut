// The build-time configuration. This file is the only reader of `import.meta.env`.

/**
 * The asset host's base URL, without a trailing slash. It must be an
 * `https://` URL with no query string and no fragment.
 */
function readAssetBaseUrl(raw: unknown): string {
  if (typeof raw !== "string" || raw === "") {
    throw new Error("VITE_ASSET_BASE_URL is not set");
  }
  if (!URL.canParse(raw)) {
    throw new Error("VITE_ASSET_BASE_URL is not a URL");
  }
  if (new URL(raw).protocol !== "https:") {
    throw new Error("VITE_ASSET_BASE_URL must be an https:// URL");
  }
  if (raw.includes("?") || raw.includes("#")) {
    throw new Error("VITE_ASSET_BASE_URL must have no query string and no fragment");
  }
  return raw.replace(/\/+$/, "");
}

/**
 * The base64 of the test key that also verifies entitlement tokens, or
 * `null`. It is set for the builds the E2E tests and the bench run against,
 * and never for a deployment: CI refuses a bundle that holds it
 * (v2implementation D-24).
 */
function readEntitlementTestPublicKey(raw: unknown): string | null {
  if (raw === undefined || raw === "") {
    return null;
  }
  if (typeof raw !== "string") {
    throw new Error("VITE_ENTITLEMENT_TEST_PUBLIC_KEY is not a string");
  }
  let decoded: string;
  try {
    decoded = atob(raw);
  } catch (cause) {
    throw new Error("VITE_ENTITLEMENT_TEST_PUBLIC_KEY is not base64", { cause });
  }
  if (decoded.length !== 32) {
    throw new Error("VITE_ENTITLEMENT_TEST_PUBLIC_KEY must decode to 32 bytes");
  }
  return raw;
}

// A missing or malformed value throws here, when the module loads, so a
// misconfigured build fails on its first page view and not later.
export const env: {
  readonly assetBaseUrl: string;
  readonly dev: boolean;
  readonly entitlementTestPublicKey: string | null;
} = {
  assetBaseUrl: readAssetBaseUrl(import.meta.env.VITE_ASSET_BASE_URL),
  dev: import.meta.env.DEV,
  entitlementTestPublicKey: readEntitlementTestPublicKey(import.meta.env.VITE_ENTITLEMENT_TEST_PUBLIC_KEY),
};
