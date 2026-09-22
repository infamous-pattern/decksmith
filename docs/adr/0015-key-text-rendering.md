# ADR-0015: Anti-aliased text on physical keys

The 3x5 diagnostic font was unsuitable for normal key captions. Key text now uses
bundled, unmodified Liberation Sans Bold 2.1.5, parsed with skrifa 0.47.0 and rendered
with tiny-skia 0.12.0. Proportional advances, width fitting, outline rasterization
and alpha blending replace enlarged diagnostic glyphs. Existing positioning,
color, transparency and hidden-label settings remain supported. The touch strip
retains its previous renderer in this checkpoint.

The font was copied from Fedora liberation-sans-fonts-2.1.5-15.fc44.noarch; its SIL
OFL 1.1 license and copyright notices accompany it under assets/fonts. No font
modification or reserved-name change was made. Application source remains Apache-2.0.
Exact dependency license exceptions allow BSD-3-Clause for tiny-skia 0.12.0 and
tiny-skia-path 0.12.0, and BSD-2-Clause for arrayref 0.3.9. Notices are retained under
docs/third-party-licenses. Advisory checks remain enforced; no advisory exceptions
were introduced. The initial ab_glyph route was removed because its ttf-parser
dependency is reported unmaintained by the current advisory database.

The font file is compiled into the binary so it works independently of the user's
installed fonts. Captions are rasterized when page frames change, not every input
poll. Width is fitted within 104px. The caption_preview example renders a 120px
sample using the same production code without accessing hardware.
