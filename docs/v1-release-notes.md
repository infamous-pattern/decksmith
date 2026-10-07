# Decksmith 1.0.0

Decksmith provides a native GNOME interface and background controls for one
Elgato Stream Deck + on Fedora Workstation 44, x86_64, GNOME/Wayland.

## Included

- An integrated Home, Pages, Keys & Dials and About interface, with settings beside
  a shared device preview and colors that follow GNOME appearance.
- Saved pages, application-based page switching, shared dial defaults and optional
  page overrides. Save and Apply remains explicit.
- Application and website launch, media controls, per-app audio, output and input
  device controls, system sounds and Stream Deck brightness.
- GNOME system actions with confirmation before shutdown or reboot; unavailable
  capabilities are reported rather than silently substituted.
- Custom labels, emoji, image/icon appearance, page themes, Rune Outline artwork
  and 14 bundled font choices, including decorative Viking Runes.
- Live touch-strip audio status, volume/mute feedback and meters.
- Push-to-talk on a key or dial press, with named-microphone targeting and safe
  muting on release or guardian failure. Overlapping holds stay live until the
  last held control is released.
- Background-only login startup, a GNOME indicator, Auto-Lock, device reconnection,
  suspend recovery and complete device blanking when controls quit or stop.
- Versioned layout import/export, retained configuration backups, compatible
  rollback and removal of integration while keeping saved data.
- Build-provenance verification of the accepted downloads before installation.

## Support and testing

Fedora Workstation 44 and one Stream Deck + are the supported V1 combination.
Release testing also exercises Fedora Workstation 45 Beta, Ubuntu 26.04 and Debian
13. Their diagnostic findings do not certify supported installations: Fedora 45
final requires its own validation; Debian's older glibc cannot run this Fedora
binary, and Ubuntu's graphical session coverage remains incomplete.

Keyboard navigation, screen-reader names/values, GNOME appearance and reference
display-scaling review were accepted by the reference user. Clean dependency
refusal/retry, installation, preview upgrade/rollback, saved-data preservation,
login and physical recovery were exercised on the frozen candidate. See the
[release review](v1-release-review.md) for the qualification scope and limits.

## Known limits

- English only; English, German, French, Spanish and Italian localization targets
  V2. Broad GNOME/Wayland distribution compatibility and general plugins target
  V1.5. Other device models and simultaneous independent devices remain later work.
- Homebridge is experimental and separately installed. Its companion binary has
  separate provenance; it is not authenticated by the signed core archive.
- App-volume controls adjust PipeWire/PulseAudio streams, so a website's internal
  volume slider does not move. System sounds control event sounds, not the overall
  speaker/headphone level.
- Below-1% idle daemon CPU remains a future performance target. Finite samples
  measured 1.53–2.01% daemon/2.77–3.72% core-service CPU in two quiet candidate
  comparison phases and 3.52%/6.53% under interactive audio load, where 100%
  means one logical core. An earlier quiet observation was 2.46%/4.42%.
  These finite samples span different sessions and workloads; they are not a
  statistical equivalence test or CPU guarantee. Hardware, layout and workload
  affect these observations.
- Exact input-to-visible feedback targets are not yet numerically qualified on
  this artifact; numerical qualification is deferred beyond V1 by the October 7
  scope decision. Internal audio dispatch p95 was 21.445 ms in the physical sample;
  that boundary excludes USB/firmware, backend completion and visible pixels.
- Updates keep previous releases and saved-data backups. Rollback requires a
  compatible saved format; very large text may require scrolling or a larger
  window. In the VM fixture, 1024-pixel width with enlarged text clipped content;
  normal text at 1024 and enlarged text at the default 1280 width passed. Use the
  wider window with enlarged text. Test results do not cover every display, USB
  controller or firmware.

## Installation and feedback

Use the [verified installation guide](v1-verified-installation.md). Installation uses the regular desktop account, leaves controls
stopped unless explicitly activated, and preserves an existing login preference.
Close other Stream Deck controllers before starting background controls.

Report reproducible problems through the repository's issue tracker with the
Decksmith, Fedora and GNOME versions, device model and expected/actual behavior.
Remove credentials, private addresses and personal layout data from logs/images.
Recovery and retained-data removal are covered in the installation guide.

Decksmith is independent of Elgato. Elgato and Stream Deck are trademarks of their
respective owners; see the project's trademark and third-party license notices.
Original Decksmith code is Apache-2.0.
