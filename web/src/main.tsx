// Temporary entry point: it proves the toolchain. The real one arrives with
// the pages (v1implementation §11.1).
import { createRoot } from "react-dom/client";

const root = document.getElementById("root");
if (root !== null) {
  createRoot(root).render(<div />);
}
