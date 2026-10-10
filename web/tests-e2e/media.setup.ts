// The setup of the media suites and of the bench: the files the asset host
// would serve are put into `fixtures/.cache/`, once, and checked against
// their hashes. It is a project of its own, which only `media` and `bench`
// depend on, so that `pnpm e2e` never downloads a model.

import { test as setup } from "@playwright/test";

import { fillAssetCache } from "./helpers/fixtures";

// The model is up to 260 MB, and the first run fetches all of it.
const DOWNLOAD_TIMEOUT_MS = 30 * 60_000;

setup("the asset cache holds the model and the reference clip", async () => {
  setup.setTimeout(DOWNLOAD_TIMEOUT_MS);
  await fillAssetCache();
});
