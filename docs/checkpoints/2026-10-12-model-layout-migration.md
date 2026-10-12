# Model-aware layout and migration foundation — October 12, 2026

Second implementation slice of [ADR-0017](../adr/0017-device-scoped-actions-and-settings.md).
Base source revision: `55caa4bd061d7fae14f98cc8f8a03b5d3665cb91`; this builds on
the [model/VirtualDeck checkpoint](2026-10-12-device-model-foundation.md).

## Implemented

- Shared bounded Rust document validation accepts legacy Plus/V1 and explicitly
  modeled Plus/MK.2 V2, with exact key/control requirements and strict unknown-field
  handling. The running Plus Control1/renderer still accepts V1 only.
- Immutable document migration retains nested raw JSON values and exact original
  backup bytes, including Unicode labels, canonical PNG assets, themes, navigation,
  dial overrides, unavailable providers and large unknown-settings integer tokens.
- MK.2 has independent Home/Media/System defaults and 15 keys per page, without
  copied Plus assignments or fabricated dials/touch controls.
- An explicit saved-action identity projection separates provider/action/schema
  from display labels without replacing execution handlers or plugin IDs.
- Offline generation tools prepare and verify both documents plus a bounded
  private configuration backup, atomically publish a generation pointer, reject
  stale writers and damaged/unsafe inputs, and restore original files into a new
  directory. Normal startup does not invoke them or activate the bundle.
- V2 package metadata/model matching and 256-entry capacity cover the MK.2 grid;
  V1 imports and explicit compatible Plus legacy export remain available.

See [format, tool and limit documentation](../layout-models.md).

## Validation and evidence

Focused tests cover format/model mismatch, absent controls, bounded documents,
custom labels/artwork/preferences, missing-provider persistence, action identity,
large-number token preservation, exact backup/restore, interrupted initial and
subsequent generations, writer exclusion, stale generations, checksums,
symlinks/traversal, destination protection and generation/snapshot limits.

The source-built CLI was exercised end to end with a separate fixture directory:
prepare a store, inspect both model documents, verify V2 is rejected by the
current control/renderer path, restore every original file byte-for-byte, and
validate the restored V1 layout with the currently installed build's read-only
layout validator. No source fixture or installed configuration was changed.

The full all-feature Rust workspace suite passed **171 tests, zero failures,
three explicitly ignored environment/hardware tests**. Formatting, all-target/
all-feature Clippy with warnings denied and the hardware-enabled build passed.
The focused Python package/editor/theme/plugin suite passed **33 tests**,
including nine package tests. Cargo-deny passed advisories, bans, licenses and
sources under the existing policy. Physical or environment-gated checks are not
counted as new acceptance.
SHA-256 uses pinned `sha2` 0.10.9; dependency/license/advisory policy remains enabled.
The raw JSON preservation feature adds no separate runtime service or network use.

## Limits and next work

This is a document/migration foundation and cold migration bundle, not an active
multi-device store/registry or physical MK.2 implementation. Preferences are
retained in the backup; per-device preference interpretation and assignment are
still required before live activation. The checks prove content restoration and
reader compatibility, not full deployed cross-build service/hardware rollback.
Snapshots do not include installed binaries, systemd/login integrations or external
keyring contents. Numeric visible-response and resource acceptance remain separate.

No production service was restarted, installed build replaced, saved layout
migrated, device opened or VM started. Next: stable device registry, device-scoped
runtime/API and shared audio/PTT lifecycle, followed by editor/MK.2 integration
and physical Plus/MK.2 qualification. General native plugin permissions and
OpenAction mapping remain unfinished gates.
