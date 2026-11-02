// The routes of the app (TS §13.1). Four are mounted; any other path, including
// the two that arrive in V6, goes to the landing page.

import { createBrowserRouter, generatePath, Navigate } from "react-router";

import { CapabilityGate } from "./ui/components/CapabilityGate";
import { EditorPage } from "./ui/pages/EditorPage";
import { LandingPage } from "./ui/pages/LandingPage";
import { LegalPage } from "./ui/pages/LegalPage";
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
  { path: ROUTES.landing, element: <LandingPage settingsPath={ROUTES.settings} appPath={ROUTES.app} legalPath={(doc) => generatePath(ROUTES.legal, { doc })} /> },
  {
    path: ROUTES.app,
    element: (
      <CapabilityGate>
        <EditorPage />
      </CapabilityGate>
    ),
  },
  { path: ROUTES.settings, element: <SettingsPage homePath={ROUTES.landing} /> },
  { path: ROUTES.legal, element: <LegalPage homePath={ROUTES.landing} /> },
  { path: "*", element: <Navigate to={ROUTES.landing} replace /> },
]);
