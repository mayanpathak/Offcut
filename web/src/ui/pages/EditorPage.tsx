// `/app`: the editor. One clip at a time, and what the page shows is where
// that clip is on its way: the drop zone, the processing card, the preview
// with the export, or what went wrong, always in the same layout
// (v4implementation D-178). From the preview and from a clip that ended badly
// there is a way to start over (J2 to J10).

import { EditorLayout } from "../layout/EditorLayout";
import { useStageView } from "../layout/Stage";

export function EditorPage() {
  const view = useStageView();
  return <EditorLayout view={view} />;
}
