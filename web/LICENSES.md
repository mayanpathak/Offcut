# Licences of the assets the web app ships

The page serves three things made by others, besides its npm packages: the interface font, the landing page's accent font and the icons (v4implementation D-180, D-181; designimplementation D-330). The fonts the captions are drawn with are embedded in the scene and listed in `crates/offcut-scene/assets/fonts/LICENSES.md`. The 72 images the scene can draw (V6.2) are embedded in the render module; `crates/offcut-scene/assets/images/manifest.json` lists each with its pinned source and licence, and their notices are below (Fluent Emoji) and under Lucide 1.61.0.

| Asset | What it is | Taken from | Licence |
|---|---|---|---|
| `src/ui/assets/inter-latin.woff2` | Inter, "Version 4.001;git-9221beed3" in its name table: the Latin subset of the scene's `Inter-Variable.ttf` (Inter 4.1), made with `pyftsubset` (fonttools 4.67.0) as WOFF2, with the `wght` (100-900) and `opsz` (14-32) axes and the features `calt kern ss01 tnum`. 65,508 bytes, SHA-256 `8792da2b49183db9439fd7b98655e65f4a8c43711c7cf37b7d32ee8b13c20a8c` | `InterVariable.ttf` of the release `v4.1` of `github.com/rsms/inter`, subset by us (v4changelog Entry 1) | SIL Open Font License 1.1. The font's copyright line: "Copyright 2016 The Inter Project Authors". No Reserved Font Name |
| `src/ui/assets/jetbrains-mono-latin.woff2` | JetBrains Mono, "Version 2.304" in its name table, at weight 700: the scene's `JetBrainsMono-Variable.ttf` pinned to `wght` 700 with `fonttools varLib.instancer`, then subset with `pyftsubset` (fonttools 4.67.0) to U+0020-007E, U+00B7, U+00D7, U+2192 and U+2014, as WOFF2, with no layout features. The landing page's accent face for timecodes, chips and figures (designimplementation D-330). 4,912 bytes, SHA-256 `146e9dd8739aaf28576f88b034e2b6be8ea0819eaec1715b2015cd2dfdb6c2ee` | `fonts/variable/JetBrainsMono[wght].ttf` of the release `v2.304` of `github.com/JetBrains/JetBrainsMono` (`crates/offcut-scene/assets/fonts/LICENSES.md`), subset by us | SIL Open Font License 1.1. The font's copyright line: "Copyright 2020 The JetBrains Mono Project Authors". No Reserved Font Name. The licence text is in `crates/offcut-scene/assets/fonts/LICENSES.md` |
| `src/ui/icons.tsx` | 22 icon paths | `lucide-static` 1.61.0 (`github.com/lucide-icons/lucide`), copied by hand; no package is installed | ISC (Lucide). Of the 22, `alert-triangle`, `arrow-down`, `arrow-right`, `arrow-up`, `check`, `download`, `hash`, `more-horizontal`, `type` and `x` come from Feather, under MIT |
| `crates/offcut-scene/assets/images/emoji/` (in the render module) | 48 emoji, 160 × 160 PNGs resized from the "3D" style's 256 px PNGs by `scripts/gen-image-bundle.mjs` (v6.2implementation D-470) | `github.com/microsoft/fluentui-emoji` at commit `1ffb34c752ecf5d402f04cfb4b392c77f57c54bc` | MIT, "Copyright (c) Microsoft Corporation." The text is below and in `crates/offcut-scene/assets/images/LICENSES/` |
| `crates/offcut-scene/assets/images/icons/` (in the render module) | 24 icons drawn white at 160 × 160 from `lucide-static` 1.61.0's SVGs by the same script | `lucide-static` 1.61.0 | ISC (Lucide); `arrow-right`, `calendar`, `clock`, `lock`, `smartphone`, `target`, `triangle-alert` (Feather's `alert-triangle`) and `circle-x` (Feather's `x-circle`) come from Feather, under MIT. The text is under Lucide 1.61.0 below |

The licences ask that their notice goes with every copy. They follow as their projects ship them.

## Inter 4.1

```text
Copyright (c) 2016 The Inter Project Authors (https://github.com/rsms/inter)

This Font Software is licensed under the SIL Open Font License, Version 1.1.
This license is copied below, and is also available with a FAQ at:
http://scripts.sil.org/OFL

-----------------------------------------------------------
SIL OPEN FONT LICENSE Version 1.1 - 26 February 2007
-----------------------------------------------------------

PREAMBLE
The goals of the Open Font License (OFL) are to stimulate worldwide
development of collaborative font projects, to support the font creation
efforts of academic and linguistic communities, and to provide a free and
open framework in which fonts may be shared and improved in partnership
with others.

The OFL allows the licensed fonts to be used, studied, modified and
redistributed freely as long as they are not sold by themselves. The
fonts, including any derivative works, can be bundled, embedded,
redistributed and/or sold with any software provided that any reserved
names are not used by derivative works. The fonts and derivatives,
however, cannot be released under any other type of license. The
requirement for fonts to remain under this license does not apply
to any document created using the fonts or their derivatives.

DEFINITIONS
"Font Software" refers to the set of files released by the Copyright
Holder(s) under this license and clearly marked as such. This may
include source files, build scripts and documentation.

"Reserved Font Name" refers to any names specified as such after the
copyright statement(s).

"Original Version" refers to the collection of Font Software components as
distributed by the Copyright Holder(s).

"Modified Version" refers to any derivative made by adding to, deleting,
or substituting -- in part or in whole -- any of the components of the
Original Version, by changing formats or by porting the Font Software to a
new environment.

"Author" refers to any designer, engineer, programmer, technical
writer or other person who contributed to the Font Software.

PERMISSION AND CONDITIONS
Permission is hereby granted, free of charge, to any person obtaining
a copy of the Font Software, to use, study, copy, merge, embed, modify,
redistribute, and sell modified and unmodified copies of the Font
Software, subject to the following conditions:

1) Neither the Font Software nor any of its individual components,
in Original or Modified Versions, may be sold by itself.

2) Original or Modified Versions of the Font Software may be bundled,
redistributed and/or sold with any software, provided that each copy
contains the above copyright notice and this license. These can be
included either as stand-alone text files, human-readable headers or
in the appropriate machine-readable metadata fields within text or
binary files as long as those fields can be easily viewed by the user.

3) No Modified Version of the Font Software may use the Reserved Font
Name(s) unless explicit written permission is granted by the corresponding
Copyright Holder. This restriction only applies to the primary font name as
presented to the users.

4) The name(s) of the Copyright Holder(s) or the Author(s) of the Font
Software shall not be used to promote, endorse or advertise any
Modified Version, except to acknowledge the contribution(s) of the
Copyright Holder(s) and the Author(s) or with their explicit written
permission.

5) The Font Software, modified or unmodified, in part or in whole,
must be distributed entirely under this license, and must not be
distributed under any other license. The requirement for fonts to
remain under this license does not apply to any document created
using the Font Software.

TERMINATION
This license becomes null and void if any of the above conditions are
not met.

DISCLAIMER
THE FONT SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND,
EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO ANY WARRANTIES OF
MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT
OF COPYRIGHT, PATENT, TRADEMARK, OR OTHER RIGHT. IN NO EVENT SHALL THE
COPYRIGHT HOLDER BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY,
INCLUDING ANY GENERAL, SPECIAL, INDIRECT, INCIDENTAL, OR CONSEQUENTIAL
DAMAGES, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING
FROM, OUT OF THE USE OR INABILITY TO USE THE FONT SOFTWARE OR FROM
OTHER DEALINGS IN THE FONT SOFTWARE.
```

## JetBrains Mono 2.304

```text
Copyright 2020 The JetBrains Mono Project Authors (https://github.com/JetBrains/JetBrainsMono)

This Font Software is licensed under the SIL Open Font License, Version 1.1.
```

The licence is the SIL Open Font License 1.1, whose full text is quoted under Inter 4.1 above.

## Fluent Emoji (commit `1ffb34c`)

The file `LICENSE` of `github.com/microsoft/fluentui-emoji` at that commit, as shipped (its own indentation kept).

```text
    MIT License

    Copyright (c) Microsoft Corporation.

    Permission is hereby granted, free of charge, to any person obtaining a copy
    of this software and associated documentation files (the "Software"), to deal
    in the Software without restriction, including without limitation the rights
    to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
    copies of the Software, and to permit persons to whom the Software is
    furnished to do so, subject to the following conditions:

    The above copyright notice and this permission notice shall be included in all
    copies or substantial portions of the Software.

    THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
    IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
    FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
    AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
    LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
    OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
    SOFTWARE
```

## GSAP 3.15.1 (an npm package, noted here)

The landing page's animations use `gsap` (designimplementation D-318). It is not open source: it is licensed under GreenSock's "Standard 'no charge' license", https://gsap.com/standard-license, which grants its use "on any website, web application, or digital interface" and forbids its use in tools that compete with Webflow's visual animation building. Offcut uses it on its landing page only (D-319). The production build minifies it and does not keep its copyright banner comments; its notice is kept here and in the package. Whether the banners must be kept in the bundle is for the founder (designchangelog, open items). The licence was read on 2026-11-05 (designchangelog Entry 4).

## Lucide 1.61.0

The file `LICENSE` of `lucide-static` 1.61.0, which also carries the licence of Feather for the icons it lists.

```text
ISC License

Copyright (c) 2026 Lucide Icons and Contributors

Permission to use, copy, modify, and/or distribute this software for any
purpose with or without fee is hereby granted, provided that the above
copyright notice and this permission notice appear in all copies.

THE SOFTWARE IS PROVIDED "AS IS" AND THE AUTHOR DISCLAIMS ALL WARRANTIES
WITH REGARD TO THIS SOFTWARE INCLUDING ALL IMPLIED WARRANTIES OF
MERCHANTABILITY AND FITNESS. IN NO EVENT SHALL THE AUTHOR BE LIABLE FOR
ANY SPECIAL, DIRECT, INDIRECT, OR CONSEQUENTIAL DAMAGES OR ANY DAMAGES
WHATSOEVER RESULTING FROM LOSS OF USE, DATA OR PROFITS, WHETHER IN AN
ACTION OF CONTRACT, NEGLIGENCE OR OTHER TORTIOUS ACTION, ARISING OUT OF
OR IN CONNECTION WITH THE USE OR PERFORMANCE OF THIS SOFTWARE.

---

The following Lucide icons are derived from the Feather project:

airplay, alert-circle, alert-octagon, alert-triangle, aperture, arrow-down-circle, arrow-down-left, arrow-down-right, arrow-down, arrow-left-circle, arrow-left, arrow-right-circle, arrow-right, arrow-up-circle, arrow-up-left, arrow-up-right, arrow-up, at-sign, calendar, cast, check, chevron-down, chevron-left, chevron-right, chevron-up, chevrons-down, chevrons-left, chevrons-right, chevrons-up, circle, clipboard, clock, code, columns, command, compass, corner-down-left, corner-down-right, corner-left-down, corner-left-up, corner-right-down, corner-right-up, corner-up-left, corner-up-right, crosshair, database, divide-circle, divide-square, dollar-sign, download, external-link, feather, frown, hash, headphones, help-circle, info, italic, key, layout, life-buoy, link-2, link, loader, lock, log-in, log-out, maximize, meh, minimize, minimize-2, minus-circle, minus-square, minus, monitor, moon, more-horizontal, more-vertical, move, music, navigation-2, navigation, octagon, pause-circle, percent, plus-circle, plus-square, plus, power, radio, rss, search, server, share, shopping-bag, sidebar, smartphone, smile, square, table-2, tablet, target, terminal, trash-2, trash, triangle, tv, type, upload, x-circle, x-octagon, x-square, x, zoom-in, zoom-out

The MIT License (MIT) (for the icons listed above)

Copyright (c) 2013-present Cole Bemis

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
```
