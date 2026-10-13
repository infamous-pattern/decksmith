# Saved-device workspace checkpoint — October 13, 2026

This slice connects the native model-aware editor to a persistent device-addressed
service and actual read-only inventory. It deliberately does not replace the
installed production worker. See [behavior and developer launch instructions](../device-workspace.md).

The final immutable source snapshot
`62b1cf38d60f4baa40d8756c42c9ce1923ed7e5c86f3249803018cdec759a667`
passed AI-server qualification: **272 Rust cases, zero failures, four intentionally
ignored environment/hardware/stress cases**; strict all-target/all-feature Clippy,
format and build; **192 Studio** and **111 script** cases (one conditional skip
each); native profile lab, saved-device workspace and unchanged legacy workspace
checks at 1920 × 1200. No VM, real audio device or physical HID adapter was opened.

An earlier native workspace command exited successfully without performing its
checks because its GTK application identifier conflicted with the service bus
name. That result is not acceptance evidence. The identifiers were separated;
both native profile checks now require their final test phase and outcome file,
and the corrected run produced completed outcomes and model-specific screenshots.

The saved-workspace native check covers explicit empty-store enrollment, prepared
Plus import, independent MK.2 defaults, adding a device without losing a dirty
Plus draft, one stable editor window, all fifteen MK.2 positions, model-sized
textures, disconnected saves, retained Home drafts and disabled live actions.
Rust fixtures additionally prove persisted identity/selection/page across restart,
stale startup/snapshot/profile rejection, unavailable/duplicate/missing identity
handling, exact retained import bytes and refusal of accidentally live adapters.

A separate workstation coexistence check discovered both attached physical models
without opening HID handles. Explicit test enrollment imported the existing Plus
layout and created MK.2 defaults in a new private test store. A saved MK.2
brightness change stayed offline; restarting the workspace retained opaque IDs
and settings while changing its runtime token. Original production config hashes,
installed build, active page and brightness stayed unchanged, and the Plus stayed
connected/display-ready. No production service restart or desktop install occurred.
The first check correctly refused a relative store path; the absolute-path retry
completed. All temporary workspace processes were stopped afterward.

Remaining activation gates: shared action resolution/execution and microphone
holds, live audio feedback, lock/suspend integration, asynchronous display and
persistent-write scheduling, complete migration/recovery/retention policy, and
paired physical lifecycle/resource tests. Multi-device V1.5 remains unreleased.
