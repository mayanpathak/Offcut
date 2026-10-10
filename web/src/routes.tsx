// The routes of the app (TS §13.1). V1 mounts three of them; any other path,
// including the three that arrive in V6 and V8, goes to the landing page.

import { createBrowserRouter, Navigate } from "react-router";

import { CapabilityGate } from "./ui/components/CapabilityGate";
import { EditorPage } from "./ui/pages/EditorPage";
import { LandingPage } from "./ui/pages/LandingPage";
import { SettingsPage } from "./ui/pages/SettingsPage";

export const ROUTES = {
  landing: "/",
  app: "/app",
  authCallback: "/auth/callback",
  account: "/account",
  settings: "/settings",
  legal: "/legal/:doc",
} as const;

export const router = createBrowserRouter([
  { path: ROUTES.landing, element: <LandingPage settingsPath={ROUTES.settings} appPath={ROUTES.app} /> },
  {
    path: ROUTES.app,
    element: (
      <CapabilityGate>
        <EditorPage />
      </CapabilityGate>
    ),
  },
  { path: ROUTES.settings, element: <SettingsPage homePath={ROUTES.landing} /> },
  { path: "*", element: <Navigate to={ROUTES.landing} replace /> },
]);
