// The drop zone's transcription card (cloudimplementation §9.1, D-344): Cloud
// or On this device, with the trade-off in one line, before any clip is
// dropped. It shows only when the build has the flag and the server has not
// said no (D-366); otherwise it renders nothing and the drop zone is as it
// was. Signed out, the card shows the device choice and a way to sign in
// (D-343); the stored choice is left as it is.

import { useState } from "react";

import { FEATURES } from "../../config/features";
import { messages } from "../../copy/messages";
import { useAuthStore } from "../../state/auth-store";
import { type TranscriptionChoice as Choice, useCloudStore } from "../../state/cloud-store";
import { chooseTranscription } from "../../usecases/set-transcription";
import { Button } from "../primitives/Button";
import { SegmentedControl } from "../primitives/SegmentedControl";
import styles from "../styles/components.module.css";
import { SignInDialog } from "./SignInDialog";

export function TranscriptionChoice() {
  const availability = useCloudStore((state) => state.availability);
  const choice = useCloudStore((state) => state.choice);
  const signedIn = useAuthStore((state) => state.status === "signed_in");
  const [signingIn, setSigningIn] = useState(false);
  if (!FEATURES.cloud_transcribe || availability === "unavailable") {
    return null;
  }
  const copy = messages.transcription;
  const value: Choice = signedIn ? choice : "device";
  let line = copy.signedOutLine;
  if (signedIn) {
    line = value === "cloud" ? copy.cloudLine : copy.deviceLine;
  }
  return (
    <div className={styles.transcriptionChoice} data-testid="transcription-choice">
      <SegmentedControl<Choice>
        label={copy.legend}
        options={[
          { value: "cloud", label: copy.cloud },
          { value: "device", label: copy.device },
        ]}
        value={value}
        onChange={chooseTranscription}
        disabled={!signedIn}
        disabledReason={copy.signedOutLine}
      />
      <p className={styles.transcriptionLine} data-testid="transcription-line">
        {line}
      </p>
      {signedIn ? null : (
        <Button
          variant="secondary"
          size="sm"
          onClick={() => {
            setSigningIn(true);
          }}
        >
          {copy.signIn}
        </Button>
      )}
      {signingIn ? (
        <SignInDialog
          onClose={() => {
            setSigningIn(false);
          }}
        />
      ) : null}
    </div>
  );
}
