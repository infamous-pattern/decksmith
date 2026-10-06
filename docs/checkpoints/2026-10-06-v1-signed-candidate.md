# Signed V1 candidate qualification — 2026-10-06

This checkpoint identifies an immutable **candidate**, not a published V1 release.
The public preview remains `v0.1.0-preview.3`. Later evidence-only commits do not
replace or rebuild the candidate below.

## Accepted test artifact

| Field | Value |
| --- | --- |
| Application version / bundle | `1.0.0` / `1.0.0-c5e4fea53afe` |
| Public source | `68c42421ceaa65bb033993b56f0babdde2a14a6e` |
| Corresponding canonical source | `aa76487ec1dcfb8561dd747ecf2153fc2b1ddec3` |
| Shared source tree | `49d0ac5f41384a52c7f385e7f36dd3f8ee476df1` |
| Archive SHA-256 | `4334a2df091ccdb0fac2431e416fa51cf4dabe0da1dfe4ccc372f60029a3dda2` |
| Manifest files | 389; clean Fedora 44 x86_64 source |
| Hosted build/signing | [Successful candidate run 37510261430](https://github.com/infamous-pattern/decksmith/actions/runs/37510261430) |

The hosted signing job verified all six signed subjects. Independent local
verification used the retained signing bundle, accepted source commit, repository,
workflow, source ref and hosted-runner policy before downloaded code executed.
It then checked complete checksum coverage, archive contents and matching
standalone installer files. Trusted roots were fetched normally; this is not
disconnected-network verification. The verifier inside the archive matches the
reviewed source byte for byte.

Source checks passed locally and in the hosted build: formatting, strict Clippy,
140 Rust tests (three existing diagnostic tests ignored), 33 installer/security/
candidate/version tests and 167 Studio tests. Dependency advisory, ban, license
and source checks passed. Only the four project package versions changed in the
lockfile. Application version metadata is now shared across binaries, archives
and translation extraction; saved-data/schema versions did not change.

The 389-file artifact passed integrity checks, scans for known private project
details and credential patterns, and bundled-SVG active/external-reference checks.
These scans complement the test/advisory checks, not a guarantee of defect-free
code. The daemon requires glibc 2.43; the CLI requires glibc 2.38.

## Four-guest matrix

All guests tested the exact signed archive, one VM at a time. Installations used
an isolated stage; the normal guest's saved configuration, icons, install record,
integration files and current-release link were checked before and after.

| Guest | Observed environment | Exact-candidate result |
| --- | --- | --- |
| Fedora Workstation 44 | GNOME 50.5; glibc 2.43 | Signature/archive admission, staged install, data recovery, actual preview upgrade/rollback, VirtualDeck and native-editor smoke passed. |
| Fedora Workstation 45 Beta | GNOME 51.0; glibc 2.44 | Same feasible staged checks passed. This does not certify final Fedora 45. |
| Ubuntu 26.04 | GNOME 50.1; glibc 2.43 | Signature/archive admission, staged install, recovery, preview upgrade/rollback and private-bus VirtualDeck passed. Native editor was not run because no graphical session was logged in. Diagnostic only. |
| Debian 13 | GNOME 48.7; glibc 2.41 | Signature/archive admission passed. Runtime installation safely refused the incompatible Fedora binary requiring glibc 2.43. Normal saved data/setup remained intact. Native source build/runtime qualification remains open for the later distribution milestone. |

Recovery tests changed custom layout/settings and imported artwork before
restoring them, included ordinary and emoji labels, and verified uninstall/
reinstall preservation. Fedora and Ubuntu tests installed the actual published
preview.3 archive, upgraded to this candidate, rolled back to the compatible
preview and returned to V1. A deliberately unsupported saved-layout format was
rejected on rollback without changing the current release or saved data.
Historical preview.3 downloads passed their advertised digests/checksums; that
older release is not retroactively signed.

Native screenshots were inspected for visible editor controls. They show the
editor mounted by the smoke harness, not the complete application, human-paced
accessibility, display-scaling or GPU certification. Private-bus tests do not
establish real user-service/login/reboot behavior. Ubuntu/Debian remain tested
diagnostics, not supported V1 platforms.

## Helper failures and sustained VM workload

On Fedora 44, the same installed candidate passed meter stall/expiry/resume,
meter crash/replacement, reader crash/timeout and writer crash/timeout recovery.
Uncertain writer actions were not replayed; subsequent independent actions
recovered. Writer reuse respected external volume changes and reversals without
changing other targets. Idle writer release preserved the reader and reconnected
for the next action. The daemon survived without restarting.

The subsequent four-target VirtualDeck workload ran for **600.02 seconds**:

| Observation | Result |
| --- | --- |
| Four live meters | Present at all 61 sample points |
| Page changes / successful audio actions | 60 / 480 |
| Daemon CPU, percentage of one core | 1.46% |
| Observed owned-process CPU, percentage of one core | 3.11% |
| Daemon RSS range | 11.40–12.23 MiB |
| Daemon descriptors / observed process count | 16 throughout / five throughout |
| Daemon restart | None |

Sixteen pre-dispatch waits respected the display guard; uncertain backend actions
were never retried. Across 108 page requests in the fault/workload run, observed
display-ready response was median 25.10 ms, p95 54.78 ms, maximum 55.87 ms. This
includes D-Bus, rendering and observer polling. It is **not physical HID dispatch
or input-to-visible-pixel latency**.

These measurements use synthetic null-sink audio targets, editor-test actions
and a private D-Bus. CPU omits exited children between samples and external
backend work; RSS covers the daemon, not its entire service. Private-bus GNOME
screen-state activation adds fixture noise. These are reproducible VM stability
observations, not a replacement for final physical idle/loaded measurements.
The original guest layout/default audio routing remained unchanged. Owned fixture
processes and audio modules were removed. The first attempt stopped before test
execution because shutdown had cleared the guest's temporary staging directory;
restaging the same signed archive fixed that fixture setup failure.

All four VMs are now shut off with autostart disabled. The physical desktop still
runs the previous bundle `0.1.0-257113d71d41`; this work did not replace its build,
restart its controls, change its layout or issue host audio/power actions. Raw
logs, signing evidence and machine-readable results remain private.

## Remaining release gates

The candidate's identity/signing, scoped staged recovery matrix and VM helper/
workload checks above are complete. The overall release gates are not yet closed:

- Final-artifact physical actions, lifecycle/recovery, quiet and interactive
  resource observations, and precise visible-response qualification.
- Real per-user service/login/reboot and clean dependency/first-run installation
  checks, rather than only existing-system staged installs.
- Human-paced final-artifact usability/accessibility, theme/text/scaling checks
  and any untested advertised action/fault case.
- Experimental Homebridge failure/expiry/recovery and agreed live qualification
  if this component ships in the accepted artifact.
- Public V1 installation/provenance guidance and the explicit release decision.
  The existing preview URL bootstrap remains checksum-only and defaults to
  preview.3; do not describe it as the signed V1 installation path.

Prior-build physical evidence may support comparison, but must not silently
qualify this new artifact. The below-1% idle daemon CPU target remains post-V1.
Publish only the accepted bytes after completing or explicitly reconciling the
[required release gates](../v1-release-scope.md).
