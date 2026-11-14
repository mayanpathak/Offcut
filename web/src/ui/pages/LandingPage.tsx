// J1: the landing page. What Offcut does and what comes next, the way in,
// where it works, that the video is not uploaded, and whether this browser
// can run it (J3). A clip dropped here is taken, and the page moves on to the
// editor with it (v2implementation D-48).
//
// The design release (designimplementation D-316 to D-335): the page is made
// of the sections in ui/landing/, on the editor's dark tokens. This file keeps
// its props and its navigation effect; the sections hold the rest.

import { useEffect, useRef } from "react";
import { useNavigate } from "react-router";

import type { LegalDoc } from "../../copy/messages";
import { useClipStore } from "../../state/clip-store";
import { BeforeAfter } from "../landing/BeforeAfter";
import { ComingSoon } from "../landing/ComingSoon";
import { Features } from "../landing/Features";
import { FinalCall } from "../landing/FinalCall";
import { Hero } from "../landing/Hero";
import { HowItWorks } from "../landing/HowItWorks";
import { Marquee } from "../landing/Marquee";
import { MotionScope } from "../landing/parts";
import { Pricing } from "../landing/Pricing";
import { Privacy } from "../landing/Privacy";
import { SiteFooter } from "../landing/SiteFooter";
import { Playhead, TopBar } from "../landing/TopBar";
import { TryIt } from "../landing/TryIt";
import { Updates } from "../landing/Updates";
import styles from "../styles/landing.module.css";

export function LandingPage({
  settingsPath,
  appPath,
  accountPath,
  legalPath,
}: {
  settingsPath: string;
  appPath: string;
  /** V6.2 (P206a, D-501): the top bar's "Sign in" or "Account" leads here. */
  accountPath: string;
  legalPath: (doc: LegalDoc) => string;
}) {
  const navigate = useNavigate();
  const clipStatus = useClipStore((state) => state.status);
  const lastClipStatus = useRef(clipStatus);

  // The move to the editor follows a clip that arrives while this page is
  // shown. A clip that was there already does not send a visitor away who
  // came back to this page.
  useEffect(() => {
    const arrived = lastClipStatus.current === "idle" && clipStatus !== "idle";
    lastClipStatus.current = clipStatus;
    if (arrived) {
      void navigate(appPath);
    }
  }, [clipStatus, appPath, navigate]);

  return (
    <MotionScope>
      <div className={styles.landing} data-landing data-theme="dark">
        <TopBar appPath={appPath} accountPath={accountPath} />
        <Playhead />
        <main>
          <Hero appPath={appPath} settingsPath={settingsPath} />
          <Marquee />
          <BeforeAfter />
          <Features />
          <HowItWorks />
          <TryIt />
          <Privacy settingsPath={settingsPath} />
          <Pricing appPath={appPath} />
          <ComingSoon />
          <Updates />
          <FinalCall appPath={appPath} />
        </main>
        <SiteFooter legalPath={legalPath} settingsPath={settingsPath} />
      </div>
    </MotionScope>
  );
}
