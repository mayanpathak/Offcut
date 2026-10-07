import { defineConfig } from "vitest/config";

export default defineConfig({
  test: {
    // A test file that touches the DOM asks for jsdom itself:
    // `// @vitest-environment jsdom` on its first line.
    environment: "node",
    include: ["src/**/*.test.ts"],
    // `config/env.ts` refuses to load without this. `.env.local` is not in
    // the repository, so the tests bring their own value.
    env: { VITE_ASSET_BASE_URL: "https://assets.example.invalid" },
  },
});
