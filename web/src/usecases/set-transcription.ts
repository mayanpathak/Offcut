// The drop zone's transcription card was changed (cloudimplementation §9.1,
// D-367): the choice holds for this page at once, and for the next visit
// once it is written. The next clip's run reads it; one already running
// keeps its mode (D-368).

import { setTranscriptionChoice } from "../persistence/prefs-repo";
import { setChoice, type TranscriptionChoice } from "../state/cloud-store";

export function chooseTranscription(choice: TranscriptionChoice): void {
  setChoice(choice);
  void setTranscriptionChoice(choice);
}
