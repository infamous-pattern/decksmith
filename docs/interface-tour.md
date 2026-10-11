# Interface tour


The current native GTK4/libadwaita interface follows your GNOME light or dark
appearance. These screenshots show the development build with a demonstration
layout; device artwork and control assignments are customizable. Every tab keeps
its controls on the left and the device preview on the right.

The editor includes 14 bundled font choices: Sans, Serif, Monospace, Roboto,
Open Sans, Lato, Montserrat, Oswald, Raleway, Poppins, Nunito, Merriweather,
Source Sans 3, and Viking Runes (decorative). The rune option converts letters
for display while preserving the original label text. Fonts work offline and
use the same renderer for the preview and physical device.
[Font sources and licenses](../assets/fonts/README.md).

The offline icon library includes **Rune Outline** and **Tabler** artwork. New
media and system assignments prefer Rune where available; existing saved artwork
is preserved. Outline icons follow the key's text color and page theme.
[Icon library and asset licenses](appearance-and-icons.md).

When launched with background controls stopped, Decksmith offers to start them
or continue without them. **Quit Decksmith**, **Stop Background controls**, and
normal system shutdown clear all keys and the touch strip. Closing only the
editor leaves background controls running.

**Home — device status, background controls, login startup and device settings.**

![Decksmith Home with device preview and background-control settings](images/decksmith-home.png?v=914df36b4fe4)

**Pages — organize pages, assign automatic switching and edit shared appearance.**

![Decksmith Pages with page selection and theme settings](images/decksmith-pages.png?v=3699dfce7f8b)

**Keys — assign actions, edit labels and customize artwork in the same window.**

![Decksmith key editor showing a Lock Desktop system action](images/decksmith-keys.png?v=398cb45aebf1)

**Dials — choose audio targets, rotation and press behavior, and touch-strip styling (classic bar shown).**

![Decksmith dial editor with audio assignment and touch-strip preview](images/decksmith-dials.png?v=e5c77595d635)

[View the About page](images/decksmith-about.png?v=9f9747314fbe). The **Plugins** tab is a
roadmap page when the experimental integration is not installed; general plugin
support targets **V1.5**. With the managed
[OpenHomeB companion](openhomeb-runtime.md), it provides inline Homebridge
connection settings and follows Decksmith startup and shutdown. The screenshot
uses demonstration connection data. Capability-based accessory assignment is available in Keys & Dials; general
plugin installation and marketplace browsing remain planned.
[View the Plugins page](images/decksmith-plugins.png?v=aa258fd1616f).

The experimental Homebridge integration supports inline server/account setup in
**Plugins → Connection setup**. Password storage requires GNOME Keyring (Secret
Service) and `secret-tool`, supplied by Fedora’s `libsecret` package. Two-factor
codes are not saved; a new code may be needed after restart or session expiry.
See [Homebridge setup and recovery](openhomeb-runtime.md).

## Touch-strip gauge previews

These static examples use demonstration values from the physical device's shared
renderer. Dual-arc gauges are available in the development build and published
COSMIC preview; Single-arc meter is development-only. Original Fedora V1,
Debian and Ubuntu packages retain the classic display. See [live meters](audio-meters.md).

![Dual-arc volume and audio-activity gauges](images/decksmith-dial-gauges.png?v=0b74777f09e5)

![Muted microphone with a red crossed-out icon and red Muted text](images/decksmith-dial-gauges-muted.png?v=d8684d20ec65)

![Mixed dual-arc, single-arc and brightness styles](images/decksmith-single-arc-meter.png?v=50e23c1281bb)
