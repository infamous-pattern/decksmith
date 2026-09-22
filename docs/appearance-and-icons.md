# Appearance presets and local icons

Implemented September 14, 2026 for the current Stream Deck + development build.

## Themes and defaults

Open **Edit layout → Theme**. Choose **Maker’s Mark**, **Dark**, **Light**, or
**High Contrast**, then adjust shared font, text size, colors, key-label position
and backing. Sans, Serif and Monospace use bundled Liberation Bold fonts; text
still fits the available control area. These presets style device content, not
the GNOME application window or device casing.

Existing custom styles and artwork stay in place. To inherit shared defaults,
select a key or dial and choose **Appearance → Reset to theme**. This clears its
style overrides while keeping its action, target, label and artwork. Opaque artwork
covers the background color; transparent artwork reveals it. Transparent labels
can overlap artwork, so choose placement and size to suit the image.

Key Appearance provides text/color/background/placement controls plus font and
text-size overrides. **Icon size** is a compact percentage field with **− / +**
buttons. It starts at **100%**; each click changes **5 percentage points**, from
**10% to 100%**. Typed values must use the same five-point increments. Blank or
invalid input keeps the last valid preview and disables Save and Apply until
corrected. No slider is used.

100% fits the padded artwork area, preserving proportions and space for the
rendered label. Top labels leave artwork below; bottom and middle labels leave
artwork above. Large or wrapped labels reduce the available artwork area.
Shared icon size is available in Theme defaults, and Reset to theme restores
that inherited value (100% when no shared override exists). Existing unthemed,
non-library images retain their original appearance until scaling or a theme is
chosen. Scaling changes rendering only; original selected image data is retained.
 Dial Appearance provides font, size, text color and background.
Dial appearance follows the existing shared-default/page-override selection.
Changes participate in Undo, Redo and Discard changes. **Done** closes the theme
settings; **Save and Apply** sends the draft to the physical device.

The key and touch-strip previews use the same Rust renderer as the device.
Theme changes do not change audio levels, mute state, routing or action assignments.
An incomplete action keeps the last valid touch-strip preview clearly marked;
it does not enable Save and Apply. Unavailable rendering retains a fallback and
retries; a key tooltip reports when its exact preview is unavailable.

## Icon library

Select a key and open **Appearance → Icon library…**. Search by name or keyword,
filter by category, and select an icon. Use **Cancel** or **Escape** to close the
picker without choosing an icon; your existing artwork and other draft edits are
preserved. Applying an icon changes artwork only;
the caption and action stay as configured. Library artwork has a modest inset
inside the unchanged key face, including icons already selected from the library.
Use Icon size to reduce it further. Built-in outline icons follow the
resolved text color. Imported images retain their own colors and transparency.

The offline starter set contains **134 official Tabler outline icons**, pinned to
[v3.46.0](https://github.com/tabler/tabler-icons/releases/tag/v3.46.0), covering audio
and media, browsers and web, development and apps, files and navigation, network
and devices, and system controls. This is a selected starter set, not the full
Tabler catalog. **Get the full Tabler icon set** opens the
[official releases page](https://github.com/tabler/tabler-icons/releases/latest)
in your browser. Download and import individual icons yourself; Decksmith does
not automatically install the full collection.

**Import local icon…** accepts SVG, PNG, JPEG, WebP and ICO files up to 8 MB.
Raster sources are limited to 16 megapixels. SVG import accepts self-contained
simple vector shapes; embedded files, external references and active content are
rejected. Imported sources are normalized to transparent PNG, at most 1024×1024,
then rendered with padding for the 120×120 key. Higher-resolution source artwork
usually gives better results than enlarging a tiny website favicon.

Imports are stored by content hash under
`$XDG_DATA_HOME/decksmith/icons` (normally `~/.local/share/decksmith/icons`).
Identical normalized images share storage. Back up that directory to preserve
the complete imported library. No account or network access is needed to browse
bundled or previously imported icons.

## Sharing and compatibility

**Theme → Export theme… / Import theme…** shares just the preset and shared
appearance defaults as `decksmith-theme` version 1 JSON. Theme files contain no
actions, targets or captions. Import changes the draft, so it can be undone before
Save and Apply.

Layout export/import preserves the selected theme, control overrides, shared and
page-specific dial styles, and selected icon pixels/provenance. Behavior and
appearance remain separate files within the package. Exports using Tabler artwork
include its MIT notice. A layout package contains selected images, not the whole
local icon library. Older builds may reject new appearance fields; use an updated
Decksmith build to import these packages.

This milestone implements layout-wide defaults and control overrides. The broader
v0.4 profile/workspace/page/state theme hierarchy, panoramic canvases and independent
full appearance packages remain future work. Save and Apply remains explicit.

## Asset licensing

Tabler icons are unmodified source SVGs, copyright 2020–2026 Paweł Kuna, MIT licensed.
See [the bundled license](../assets/icons/tabler/LICENSE) and
[catalog provenance and checksums](../assets/icons/tabler/catalog.json).
Liberation Sans, Serif and Mono Bold are distributed under SIL OFL 1.1; see
[font provenance](../assets/fonts/README.md) and its license. These asset licenses
are separate from Decksmith’s Apache-2.0 original code license.
