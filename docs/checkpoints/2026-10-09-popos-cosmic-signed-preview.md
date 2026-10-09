# Pop!_OS 24.04 COSMIC signed preview qualification — October 9, 2026

## Candidate identity and scope

This is initial experimental COSMIC compatibility, separate from supported Fedora
V1 and the broader V1.5 GNOME/Wayland goal. Fedora, Debian and Ubuntu release
assets are unchanged. The package also includes the accepted optional dual-arc
touch-strip gauges; publishing this preview does not update the Fedora V1 download.

- Public source: `be238905bb3f6aa793dcda2df063fa5a2f206cbc` (clean source).
- Workflow: [Experimental Pop!_OS COSMIC candidate run 37957900967](https://github.com/infamous-pattern/decksmith/actions/runs/37957900967).
- Build: pinned official Ubuntu 24.04 base, Rust 1.97.1 and locked dependencies.
- Bundle: `1.0.0-5b198a52075d`; 415 manifest files.
- Archive: `decksmith-popos2404-x86_64.tar.gz`.
- SHA-256: `d4c8824980cf417082897805bf64f01e26172fa9fe6df14367816972326961d9`.
- Observed native requirements: daemon GLIBC_2.39, client GLIBC_2.34.
- Test VM: Pop!_OS 24.04 LTS, glibc 2.39, GTK 4.14.5, libadwaita 1.5.0,
  cosmic-comp 0.1~1791227611~24.04~41497b4 and cosmic-session
  1.10.0~1791314577~24.04~8093b59; normal COSMIC Wayland session,
  1920×1200 display. PipeWire, PipeWire-Pulse and WirePlumber were active.

## Verification and automated checks

All six subjects passed independent retained-bundle provenance verification for
the repository, separate Pop!_OS workflow, exact source SHA, main ref and
GitHub-hosted runner identity. Full checksums, archive/source/target/clean-tree
admission and bootstrap byte identity passed before downloaded code executed.
Explicit `--target popos2404` is required; cross-target or signer substitutions
are rejected. The existing signed Ubuntu and Debian previews also passed the
expanded verifier. The package honestly records Ubuntu 24.04 as its build
baseline; the build container is not a COSMIC desktop.

Formatting, strict all-feature lint and 144 Rust tests passed; three existing
hardware/long diagnostics were ignored. CI ran 94 script tests and 170 editor
tests, with one environment-specific skip in each suite. Dependency advisories,
bans, licenses and sources passed cargo-deny policy. This is evidence of these
checks, not a defect-free or byte-reproducible build guarantee; system packages
are obtained from Ubuntu repositories at build time.

The verified archive was transferred over pinned SSH and its digest checked again
before execution in the guest. It passed isolated installation, runtime/integrity,
shared-library and glibc checks. Staged backup/restore and uninstall/reinstall
preserved synthetic emoji labels, settings and custom SVG byte-for-byte. Normal
saved-state hashes were unchanged by isolated testing; this VM had no existing
saved configuration to exercise beyond the synthetic fixtures.

The exact signed package passed relocated native GTK rendering across all editor
sections with shared Rust key/touch previews. Private-bus VirtualDeck startup,
page navigation, save and layout validation passed. Source-matched native fixtures
passed geometry, validation, themes, logical shortcuts, gauge selection, custom
labels and page-style persistence on the real COSMIC Wayland compositor. These
fixture assertions do not replace the human keyboard/picker acceptance below.

## Normal-session and human acceptance

The exact package was installed per user with the previous build and configuration
backup retained. Runtime doctor reported ready. Background controls acquired the
normal session D-Bus name, waited for the absent USB device, stopped and restarted
normally. Login startup remained disabled. Systemd-logind lock reporting was
available with Auto-Lock disabled; no lock/unlock acceptance is inferred from it.

The user accepted Home, Pages, Keys & Dials and About, stable key/dial sizing,
visible controls, Tab/Shift+Tab navigation and focus, and artwork picker
open/cancel on the signed package in the normal COSMIC session. Orca is installed
but its speech was not tested. GNOME Do Not Disturb correctly reports unavailable
without GNOME Shell and rejects execution rather than modifying an ignored GNOME
preference. This guard is covered by GNOME-present and GNOME-absent tests.

## Remaining limits and publication

No physical Stream Deck or host audio was passed into the VM. Actual USB access,
physical feedback, live audio/media/system actions, lock/unlock, suspend/resume,
hotplug, login/reboot recovery and long-term resource use remain unqualified on
COSMIC. Experimental Homebridge and screen-reader support need separate COSMIC
acceptance. GNOME panel/active-window extensions and GNOME-specific notification,
Night Light, Bluetooth and lock-key integration are not COSMIC support.
Auto-Lock should remain off until independently tested on COSMIC.

At 1280×800, COSMIC can still map the editor to a narrow 900×540 window and clip
controls; manually enlarge it. The initial monitor lookup is bounded before map,
but the small-display limitation is not claimed fixed. Native tests at the
reviewed 1920×1200 display mapped the editor at 1280×736 without forced resizing.

The signed candidate is qualified for this limited experimental preview scope.
Public release approval and fresh public-download verification are pending.
[Verified installation instructions](../popos-preview-installation.md) are ready
for the proposed `v1.0.0-popos-preview.1` prerelease; download URLs become usable
only after publication. Signed metadata retains its original staging text.
