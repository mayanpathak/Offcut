// The entry point: the global styles, the start of the app, the first render.

import "./ui/styles/tokens.css";

import { createRoot } from "react-dom/client";

import { App } from "./App";
import { startApp } from "./usecases/start-app";

// Not awaited: the page renders at once and fills in as the start proceeds.
void startApp();

const root = document.getElementById("root");
if (root !== null) {
  createRoot(root).render(<App />);
}
