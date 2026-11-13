// V6.2 (v6.2implementation D-487, §11, P208): the program's audio on the voice
// a clip's Enhance names. `Clean` is today's path: `prog48.f32`, the program
// the pipeline assembled from `out48`, read as the export always read it.
// `Level` is `lvl48.f32`, assembled on the clip's time map by the audio
// worker. The page keeps no second program: a switch, the export and a
// pipeline that starts on `Level` each ask here.

import type { ClipId, TimeMap, VoiceMode } from "../gen/domain";
import * as opfs from "../persistence/opfs";
import { pool, WorkerCallError } from "../workers/pool";

/** A worker's answer in a buffer of its own, which this thread can hand on. */
function owned(pcm: Float32Array): Float32Array<ArrayBuffer> {
  const { buffer } = pcm;
  if (!(buffer instanceof ArrayBuffer)) {
    throw new Error("PCM from a worker is not in a buffer of its own");
  }
  return new Float32Array(buffer, pcm.byteOffset, pcm.length);
}

/** The voice an edit's Enhance names: `Clean` when it names none (`Enhance::v5()`). */
export function voiceOf(enhance: { voice: VoiceMode } | undefined): VoiceMode {
  return enhance?.voice ?? "clean";
}

/**
 * The program on `voice`, in a buffer of this thread's own. A storage failure
 * is thrown as `OpfsError`, a worker's as `WorkerCallError`.
 */
export async function programOn(clipId: ClipId, voice: VoiceMode, timeMap: TimeMap): Promise<Float32Array<ArrayBuffer>> {
  if (voice === "clean") {
    return opfs.readProgram(clipId);
  }
  const lvl48 = await opfs.readLevel(clipId);
  const answer = await pool.audio.assembleVoice({ voice48: lvl48, voice, timeMap, speech: [] }, { transfer: [lvl48.buffer] });
  if ("cancelled" in answer) {
    throw new WorkerCallError({ code: "E_INTERNAL", stage: "audio_chain", retryable: true, detail: "Cancelled" });
  }
  return owned(answer.prog48);
}
