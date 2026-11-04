// GSAP with its scroll plugin, registered: the one module that `gsap.ts`
// loads with `import()` (designimplementation D-318). Both libraries are
// imported here and only here, so the build puts them in one chunk of their
// own, which the editor's route never downloads (case L-8).

import { gsap } from "gsap";
import { ScrollTrigger } from "gsap/ScrollTrigger";

gsap.registerPlugin(ScrollTrigger);

export const runtime = { gsap, trigger: ScrollTrigger };
