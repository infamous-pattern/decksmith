# Rune Outline assets

This offline subset contains 12 selected Rune Outline SVGs and two derivatives:
Play Pause combines Play and Pause; Previous mirrors Next. It is not the full
Rune catalog. Source: [Rune Icons](https://github.com/Runeicons/runeicons), commit
`210c95e68213ff6cd521ea7364b96c8bdff26d40`.

Icon assets are Apache-2.0 licensed under Part 1 of [LICENSE](LICENSE). Rune's
website/editor software has separate terms and is not included in Decksmith.
[NOTICE](NOTICE) records attribution and modifications. Unmodified upstream SVGs
are retained in `originals/`; `catalog.json` records original/prepared checksums
and source paths. Fixed black fills/strokes were changed to `currentColor` without
altering the original geometry, so icons follow the key's resolved text color.

`touch/` contains five 32×32 transparent PNG derivatives rasterized from the
prepared SVGs with Decksmith's safe SVG renderer. The daemon decodes these once
into five alpha masks (5 KiB total), then blends theme colors directly into its
existing touch-strip frame. Custom application/device artwork takes precedence.
No SVG parsing or icon-mask allocation occurs during live touch updates.

The library does not rewrite saved artwork when an existing layout is loaded.
New action assignments prefer a bundled Rune equivalent and fall back to Tabler.
Layout exports containing Rune key artwork include LICENSE and NOTICE.
