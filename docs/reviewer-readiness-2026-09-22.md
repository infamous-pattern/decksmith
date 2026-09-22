# Reviewer readiness — September 22, 2026

## Public installer and recovery

Tested the exact `v0.1.0-preview.2` URL installer from the README in a new,
empty user account on the retained Fedora Workstation 44 VM. Its SHA-256 matched
the reviewed source script. This was a fresh **user installation**, not a newly
installed operating system: required Fedora packages were already present.
The missing-dependency DNF prompt was not exercised in this run.

The public bootstrap verified all four release downloads and installed
`0.1.0-739b30218d15`. No source checkout or compiler was used in the guest.
The runtime doctor passed dependency, resource-integrity and VirtualDeck checks.
The application menu entry, three command launchers and user service were created;
background controls stayed inactive and login startup stayed disabled.

Using a saved layout with a custom page name, brightness preference and PNG artwork:

- Uninstall removed all five managed integration files while preserving releases,
  backups, configuration and artwork.
- Installing preview.1 restored integration with the retained data.
- Upgrading preview.1 to preview.2 recorded the previous release correctly.
- Rollback selected preview.1; reinstalling preview.2 restored the current release.
- A final uninstall again removed integration without deleting saved data.

Hashes of all three fixture files were identical after every operation. All
operations completed successfully and controls/startup remained off. The retained
public runtime also passed private-bus VirtualDeck startup, both preview renderers,
page navigation, saving and layout validation. No USB device was attached.

## Documentation and release boundaries

The README's pinned command and installation guide match the tested behavior.
Uninstall means removal of integration, not deletion of user data or release files.
The reviewer guide now points directly to the one-command installation route.

The public preview.2 binary is `0.1.0-739b30218d15`. The subsequent compact-editor
fix and fractional-scaling development checks concern `0.1.0-bdfacd1a1cdb`; publishing
source/documentation does not replace preview.2 release assets. A future preview
must package and validate those changes before they are claimed for a download.

## Scope and evidence

This run does not certify physical USB access, real audio routing, missing-package
installation, every upgrade/schema combination or a fresh Fedora OS installation.
See [accessibility results](accessibility-checkpoint-2026-09-22.md) for the separate
native UI and screen-reader checks and their limits.

Raw results and the isolated test driver are retained locally under
`local/reviewer-readiness-2026-09-22/`. No test account credentials, VM console
captures or private logs are included in the public documentation.

## Accessibility follow-up and cleanup

The paced native test passed and Orca logged individual key/dial names, text-field
labels/values, Save and Apply readiness and navigation. Automatic announcement of
the empty-label validation reason was not demonstrated and remains follow-up work;
a full human listen-through remains outside this automated check.

Temporary Orca settings and its diagnostic service override were restored/removed.
The Fedora 44 VM was shut down after testing, with autostart disabled; Fedora 45
remained shut down. The separate reviewer account is retained for repeat testing
with Decksmith integration uninstalled and its test data/backups preserved.
