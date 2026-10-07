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

// A missing or malformed value throws here, when the module loads, so a
// misconfigured build fails on its first page view and not later.
export const env: { readonly assetBaseUrl: string; readonly dev: boolean } = {
  assetBaseUrl: readAssetBaseUrl(import.meta.env.VITE_ASSET_BASE_URL),
  dev: import.meta.env.DEV,
};
