# Bundled fonts

Liberation Sans Bold, Liberation Serif Bold and Liberation Mono Bold are bundled
unmodified for consistent offline rendering. Sans was already used by Decksmith;
Serif and Mono were copied from Fedora's installed liberation-serif-fonts and
liberation-mono-fonts 2.1.5-15.fc44 packages for the September 14 appearance milestone.

All three are covered by [SIL Open Font License 1.1](LICENSE-Liberation.txt),
including the retained Google and Red Hat copyright and reserved-font-name notices.
The same font bytes are used by physical keys and renderer-backed editor previews.

## Expanded editor choices

Roboto, Open Sans, Lato, Montserrat, Oswald, Raleway, Poppins, Nunito,
Merriweather, Source Sans 3 and Noto Sans Runic are bundled unmodified from the
Google Fonts repository. Each family's `LICENSE-*.txt` retains its OFL 1.1 license
and copyright notices. `font-sources.json` records exact download URLs and SHA-256
hashes. Variable fonts render at weight 700; static fonts use their supplied weight.
Font bytes are embedded read-only in the renderer, without system-font discovery
or network requests while editing.

**Viking Runes (decorative)** uses Noto Sans Runic and a Latin-to-Younger-Futhark
approximation during rendering. It is decorative lettering, not an Old Norse
translation or historically exact transliteration. Saved labels remain unchanged;
numbers and spaces remain readable. The same conversion is used for hardware and
editor previews.
