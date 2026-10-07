// Temporary entry point: it starts the app and renders nothing yet. The real
// one arrives with the pages (v1implementation §11.1).
import { createRoot } from "react-dom/client";

import { startApp } from "./usecases/start-app";

void startApp();

const root = document.getElementById("root");
if (root !== null) {
  createRoot(root).render(<div />);
}
