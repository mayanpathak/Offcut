import { defineConfig } from "vitest/config";

export default defineConfig({
  test: {
    // A test file that touches the DOM asks for jsdom itself:
    // `// @vitest-environment jsdom` on its first line.
    environment: "node",
    include: ["src/**/*.test.ts"],
  },
});
