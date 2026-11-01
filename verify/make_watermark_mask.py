"""Draws verify/watermark_mask_720.png, the watermark check 9 looks for.

    py -3.13 -m pip install pillow      (once: this tool's alone, not the verifier's)
    py -3.13 verify/make_watermark_mask.py

Run once, and again only when crates/offcut-scene/src/watermark.rs or
safe_area.rs moves the text: the self-test's watermark case fails until then.
It shares no code with the renderer (TS section 5): the font is read as data,
Pillow draws it, and the numbers below are written here by hand.

The picture is 720 x 1280, 8-bit grey: 128 where there is no watermark, 255 for
the text's fill, 0 for its outline.
"""

from pathlib import Path

from PIL import Image, ImageDraw, ImageFont

ROOT = Path(__file__).resolve().parent.parent
FONT = ROOT / "crates/offcut-scene/assets/fonts/Inter-Variable.ttf"
OUT = ROOT / "verify/watermark_mask_720.png"

WIDTH, HEIGHT = 720, 1280
# A logical pixel of the 1080 x 1920 canvas, at the Free export's size.
SCALE = 720 / 1080

# WATERMARK_TEXT, WATERMARK_SIZE, WATERMARK_STROKE of watermark.rs, in logical pixels.
TEXT = "Made with Offcut"
SIZE_LP = 36.0
STROKE_LP = 3.0
# The body font's weight (theme.rs, Inter 700); the optical size stays at its default.
AXES = [14, 700]
# Where the text starts: INSET_LEFT of safe_area.rs and half the outline.
LEFT_LP = 60.0 + STROKE_LP / 2
# Its baseline: the safe area's bottom, less Inter's descent and half the outline.
BASELINE_LP = 1500.0 - 8.7 - STROKE_LP / 2

# Drawn this many times larger and reduced, so that its edges are as smooth as a frame's.
OVERSAMPLE = 4
BACKGROUND, FILL, OUTLINE = 128, 255, 0


def main():
    factor = SCALE * OVERSAMPLE
    font = ImageFont.truetype(str(FONT), round(SIZE_LP * factor))
    font.set_variation_by_axes(AXES)
    big = Image.new("L", (WIDTH * OVERSAMPLE, HEIGHT * OVERSAMPLE), BACKGROUND)
    # The renderer's outline is centred on the glyph's edge, so half of it lies outside.
    ImageDraw.Draw(big).text(
        (LEFT_LP * factor, BASELINE_LP * factor),
        TEXT,
        font=font,
        fill=FILL,
        anchor="ls",
        stroke_width=round(STROKE_LP / 2 * factor),
        stroke_fill=OUTLINE,
    )
    big.resize((WIDTH, HEIGHT), Image.Resampling.BOX).save(OUT, optimize=True)
    print("wrote %s, %d bytes" % (OUT.name, OUT.stat().st_size))


if __name__ == "__main__":
    main()
