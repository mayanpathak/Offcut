// Temporary entry point: it proves the toolchain. The real one arrives with
// the pages (v1implementation §11.1).
import { createRoot } from "react-dom/client";

import { preload } from "./workers/pool";

// TEMPORARY (Prompt 20): proves in the browser that the WASM bundle is
// fetched and compiled under the production CSP. Prompt 21 removes this
// call; `startApp()` makes it from then on.
void preload();

const root = document.getElementById("root");
if (root !== null) {
  createRoot(root).render(<div />);
}
