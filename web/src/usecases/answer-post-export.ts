// The one question after an export (PS §20.5; v5implementation D-234): the answer
// is sent once per export, and a cache hit shares its export's id. In memory only,
// as one tab's session: a reload asks again.

import { track } from "../analytics/client";
import type { PostExportAnswer } from "../gen/api";
import { answered, hasAnswered, useExportStore } from "../state/export-store";

export function answerPostExport(answer: PostExportAnswer): void {
  const { exportId, status } = useExportStore.getState();
  if (status !== "done" || exportId === undefined || hasAnswered(exportId)) {
    return;
  }
  answered();
  track({ name: "post_export_answer", props: { answer } });
}
