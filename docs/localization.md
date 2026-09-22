# Localization foundation and V2 delivery

## Implemented foundation (English-only release)

The native Python interface has a gettext adapter in
`apps/decksmith-studio/i18n.py`. It resolves `locale/` relative to the installed
application or checkout, loads the session language once, and falls back to the
English source text when catalogs or messages are missing. Invalid catalogs also
fall back to English. It exposes plain, contextual and plural translation helpers.
There is no network lookup, recurring translation work, language selector, or
change to saved layouts. Reopen the process to pick up a different session locale.

Navigation labels/tooltips and desktop power-confirmation dialogs are the initial
marked examples. They are **not complete interface coverage**. Existing labels,
action IDs, device messages, GNOME extension strings and documentation remain
unchanged. Unicode label support is not implemented by this foundation.

`po/decksmith.pot` is the extraction template. `po/POTFILES.in` enumerates marked
Python sources; `po/LINGUAS` explicitly enables shipping catalogs. It is currently
empty: English is the only shipping language. No draft German, French, Spanish or
Italian catalog is advertised as supported.

## Developer workflow

Install Fedora's `gettext` package for extraction and catalog compilation:

```sh
sudo dnf install gettext
python3 scripts/localization.py extract
python3 scripts/localization.py check
python3 scripts/build-bundle.py
```

Add reviewed translations as `po/<language>.po`, then enable their language codes
in `po/LINGUAS`. The bundle builder validates and compiles enabled catalogs into
`locale/<language>/LC_MESSAGES/decksmith.mo`, covered by normal manifest hashes.
It fails on missing or invalid enabled catalogs. With no enabled catalogs, building
requires no gettext tools. Installed users never need those build tools.

In Python, use `from i18n import gettext as tr` and `tr('Complete sentence')`.
Use `pgettext` for ambiguous words and `ngettext` for plural messages. Add source
files to POTFILES when first marking them. Use named placeholders and format after
translation; avoid concatenated sentences and translated f-strings. Add
`Translators:` comments where intent, placeholders or physical space limits matter.

Never translate protocol keys, action IDs, command names, stable error codes,
configuration fields, application/device identifiers or user-authored labels.
Default-label provenance must be designed before localizing generated labels:
current English-string comparisons are not sufficient across language changes.
Translate errors at the presentation boundary, not inside persisted data.

## V2 scope and acceptance

- English, German (`de`), French (`fr`), Spanish (`es`) and Italian (`it`). Agree on
  regional terminology and use English fallback for missing entries.
- Complete UI coverage: settings, actions, tooltips, validation, errors,
  accessibility labels, desktop entries and the GNOME companion extension.
- System-language default with an optional application language preference;
  coordinate the editor and background service without altering the OS language.
- Unicode letters/combining marks and appropriate punctuation in page, key and
  dial names, with consistent normalization and character-count validation in
  Python and Rust. Retain length/resource bounds and reject control characters.
- Verify bundled font glyph coverage, text measurement, wrapping and fitting on
  physical keys/touch strips. Replace remaining ASCII-only rendering paths.
- Translate daemon-rendered status text (Muted, Live, Locked, No Audio, errors)
  while preserving physical/preview parity. Remove logic based on English text
  suffixes and retain semantic state separately from display strings.
- Track automatic versus user-authored labels explicitly; migrate legacy layouts
  conservatively, preserve custom labels, and test language changes, history and
  layout import/export. Never rename user pages automatically.
- Pseudo-localization with expanded/accented text, small windows, high-DPI,
  keyboard/screen-reader checks, plural/context/placeholder checks and missing or
  invalid catalog recovery. Require fluent-speaker review for each new language.
- Measure startup, resident memory and active meter performance; keep translation
  loading/caching out of recurring device rendering and audio polling paths.
- Translate onboarding and essential user documentation after interface terminology
  settles. Community contribution instructions and catalog-update checks accompany
  the V2 release. Do not claim all five languages until coverage and review pass.
