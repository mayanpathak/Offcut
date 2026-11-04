// The entry point: the global styles, the start of the app, the first render.

import "./ui/styles/tokens.css";

import { createRoot } from "react-dom/client";

import { App } from "./App";
import { ROUTES } from "./routes";
import { startApp } from "./usecases/start-app";

// Not awaited: the page renders at once and fills in as the start proceeds.
void startApp({ restoreSession: window.location.pathname !== ROUTES.authCallback });

const root = document.getElementById("root");
if (root !== null) {
  createRoot(root).render(<App />);
}
