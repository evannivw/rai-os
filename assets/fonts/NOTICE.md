# Font attribution

Copyright 2011 Canonical Ltd. Licensed under the Ubuntu Font Licence 1.0.
See [UFL.txt](UFL.txt). The license applies to the font data, independently of the
Apache-2.0 license of the RAI-OS kernel and renderer.

- `Ubuntu-Light.ttf`: unmodified Ubuntu Light font, copied from the locally
  installed epaint_default_fonts 0.32.3 package (the egui project).
- Modified font name: **Ubuntu Light derivative RAI ASCII**. The ASCII-only,
  four-bit coverage data in `rai-ascii.bin` and glyph metrics in
  `../../src/display/glyphs.rs` are generated derivatives under the same UFL-1.0.
  They retain the original glyph shapes at 14, 20 and 46 pixels. The complete
  source font and its license are included here; the original TTF is not linked
  into the guest.

Regenerate with `scripts/generate_display_font.py`, Python and Pillow 11.3.0
(FreeType 2.13.3 used for the committed output), plus the pinned Rust formatter.
Pillow is only a development-time generator dependency. Cargo builds and QEMU
execution use committed glyph data and do not require Python/Pillow or a font engine.

Upstream distribution:
https://github.com/emilk/egui/tree/0.32.3/crates/epaint_default_fonts/fonts

The original geometric mark and constellation illustration are RAI-OS code,
not assets from the font package. No endorsement by the font authors is implied.
