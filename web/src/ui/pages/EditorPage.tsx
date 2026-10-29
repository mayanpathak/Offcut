// `/app`: the editor. One clip at a time, and what the page shows is where
// that clip is on its way: the drop zone, the processing card, the preview
// with the export, or what went wrong, always in the same layout
// (v4implementation D-178). From the preview and from a clip that ended badly
// there is a way to start over (J2 to J10).

import { DEV_HOOKS } from "../../config/env";
import { installDevHooks } from "../dev-hooks";
import { EditorLayout } from "../layout/EditorLayout";
import { useStageView } from "../layout/Stage";

// The development hooks, outside a production build only (D-155): there
// `DEV_HOOKS` is false and the build drops the branch and the module.
if (DEV_HOOKS) {
  installDevHooks();
}

export function EditorPage() {
  const view = useStageView();
  return <EditorLayout view={view} />;
}
