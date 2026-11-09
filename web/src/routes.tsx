// The routes of the app (TS §13.1). Any other path goes to the landing page.

import { createBrowserRouter, generatePath, Navigate } from "react-router";

import { FEATURES } from "./config/features";
import { CapabilityGate } from "./ui/components/CapabilityGate";
import { AccountPage } from "./ui/pages/AccountPage";
import { AuthCallbackPage } from "./ui/pages/AuthCallbackPage";
import { EditorPage } from "./ui/pages/EditorPage";
import { LandingPage } from "./ui/pages/LandingPage";
import { LegalPage } from "./ui/pages/LegalPage";
import { LongPage } from "./ui/pages/LongPage";
import { SettingsPage } from "./ui/pages/SettingsPage";

export const ROUTES = {
  landing: "/",
  app: "/app",
  authCallback: "/auth/callback",
  account: "/account",
  settings: "/settings",
  legal: "/legal/:doc",
  /** V6.1 (D-422): "Make shorts from a long video"; with `long_video` off it leads to `/app`. */
  long: "/app/long",
} as const;

export const router = createBrowserRouter([
  { path: ROUTES.landing, element: <LandingPage settingsPath={ROUTES.settings} appPath={ROUTES.app} legalPath={(doc) => generatePath(ROUTES.legal, { doc })} /> },
  {
    path: ROUTES.app,
    element: (
      <CapabilityGate>
        <EditorPage accountPath={ROUTES.account} />
      </CapabilityGate>
    ),
  },
  {
    path: ROUTES.long,
    element: FEATURES.long_video ? (
      <CapabilityGate>
        <LongPage appPath={ROUTES.app} accountPath={ROUTES.account} />
      </CapabilityGate>
    ) : (
      <Navigate to={ROUTES.app} replace />
    ),
  },
  { path: ROUTES.authCallback, element: <AuthCallbackPage appPath={ROUTES.app} /> },
  { path: ROUTES.account, element: <AccountPage homePath={ROUTES.landing} /> },
  { path: ROUTES.settings, element: <SettingsPage homePath={ROUTES.landing} /> },
  { path: ROUTES.legal, element: <LegalPage homePath={ROUTES.landing} /> },
  { path: "*", element: <Navigate to={ROUTES.landing} replace /> },
]);
