# Launcher protocol isolation — October 22, 2026

This is a narrowly authorized reliability correction during the development
pause. General Ubuntu/Debian qualification and feature work remain paused.
After reviewing the source results, the maintainer authorized committing and
pushing the fix/testing notes to both repositories and building/verifying a
signed Fedora V1.5.2 candidate. They subsequently approved desktop installation
and confirmed the targeted physical checks below. Public-release publication
and support promotion remain separate decisions; Decksmith stays paused.

## Report and evidence boundary

On Fedora 44 GNOME/Wayland, the maintainer reported that website, application and
system keys stopped responding while audio dials continued. The installed build
was `1.5.2-46ed033e0491`. Two captured incidents showed
`native_builtin_effect_uncertain`, followed by `native_builtin_effect_blocked`.
Both Decks were reported connected, input/display ready and unlocked at the
second capture, with no USB reset/disconnect at either incident. A service
restart restored website keys after the first incident. The maintainer
identified website launch as the trigger.

A harmless local desktop-entry reproduction established a separate concrete
defect: an application launched by the guarded GIO adapter inherited the
helper's stdout. Its output could share the JSON reply channel with the helper's
own response. The Rust client deliberately treats a malformed/lost reply as an
uncertain effect, and the native backend blocks subsequent ordinary built-ins.
Audio has separate execution state, matching the reported pattern. The exact
failed reply from the real incidents was not retained: output contamination is
a verified defect and plausible explanation, not proof of those reply contents
or of every possible cause of an uncertain effect.

An explicitly approved temporary installed-file patch redirected launched-child
standard input, output and error to `/dev/null` while retaining `setsid()`. After
the controlled service restart, the maintainer reported: “I've tried the website
key several times and it's working. The other keys and audio dials are working
too.” This is immediate physical acceptance of that temporary patch, not a
long-term stability result or acceptance of the broader source change below.
That temporary modification did not match the original signed release manifest.
It was preserved in the rollback backup before the later approved signed-candidate
installation below. Source preparation itself did not change the desktop.

Private diagnostic captures, original/modified installed-file hashes, rollback
copy and full test logs remain outside public source. Do not publish raw logs,
desktop entries, URLs, profile data or device inventories from those captures.

## Source correction and retained safeguards

- The guarded launched child creates its own session and redirects descriptors
  0, 1 and 2 to `/dev/null`. Setup failure exits before executing the application.
  GIO still resolves desktop entries and passes the literal URI; discovery and
  the per-effect authorization fence retain their original order. Legacy launch
  entry points retain their existing behavior.
- The ordinary built-in helper duplicates its reply pipe onto a private,
  non-inheritable descriptor before importing adapters. Its ordinary stdout
  descriptor is redirected to `/dev/null`. Python prints and native library
  writes in the parent cannot become protocol replies, and an executed child
  cannot retain the private reply descriptor. Redirecting Python's stream alone
  would not protect against native writes to descriptor 1.
- Rust logs a fixed uncertainty reason (`reply_timeout`, `transport_failure`,
  `invalid_reply`, `helper_error` or `grant_limit`) and a bounded count of attempts
  to send Apply grants. These diagnostics contain no action payload, URL,
  desktop ID, command, exception message or reply bytes.
- Malformed/closed/timed-out replies remain uncertain. The original action is
  never replayed automatically; the native backend's process-lifetime ordinary
  built-in block remains in place. This patch prevents a known source of
  corruption rather than relaxing acknowledgement or authorization rules.

## Verification

Real GIO tests launch only disposable local fixture applications. Both the
application and website paths deliberately emit child stdout/stderr, including
fake JSON; the simulated parent adapter also emits Python and direct-descriptor
noise at startup and on a later ordinary action. Tests require exact grant/reply
sequences, literal URI preservation, `/dev/null` standard descriptors, no leaked
reply pipe, successful subsequent actions, and child survival after termination
of the helper process group. No browser, user application or actual system
setting is invoked by these fixtures.

The regression failed against the original child setup. Independent negative
controls also fail when either child isolation or parent protocol isolation is
removed. Setup-failure tests retain fail-closed behavior. Rust regressions cover
lost acknowledgements after an Apply grant without replay, and verify that
uncertainty diagnostics omit private action/reply content. Existing stale-scope,
allowlist, bounded-grant and legacy-launch tests remain part of the suite.

The AI-server verification uses an isolated source snapshot, the existing pinned
Rust 1.97.1 toolchain and dependency cache. It starts no guests and changes no
installed Decksmith build, desktop settings or schedule. Local build attempts
were limited by the read-only Cargo cache, so Rust validation is taken from the
server run rather than inferred from the local attempt. Final test receipts are
retained privately in `outputs/launch-protocol-20261022/`.

Final source validation passed: 350 Rust tests (six existing ignored tests remain
ignored), strict all-target/all-feature Clippy, 249 Studio Python tests, 122
installer/script tests, Rust formatting and diff whitespace checks. The Studio
suite passed locally and on the AI server. Both new real-GIO regressions and
the Rust privacy/no-replay checks passed. No ignored test was counted as passed,
and these source checks alone do not establish exact signed-package or physical
acceptance. Those later checks are recorded separately below.

## Signed Fedora candidate and targeted desktop acceptance

The fix was published as canonical/Gitea
`fd484e7f72a70373f588585d4f5b75c7a6155678` and sanitized GitHub
`274ade4e2f42d9cf957a562352df6c253a02b61b`, with identical source tree
`e25590a816f4395d0996cf1beee632890f11534d`. Committed runtime/test bytes match
the passing AI-server source snapshot. [Quality run 39829838476](https://github.com/infamous-pattern/decksmith/actions/runs/39829838476)
and [Fedora build/signing run 39829868860](https://github.com/infamous-pattern/decksmith/actions/runs/39829868860)
both passed. This final testing-notes update is documentation-only and does not
rebuild or change the candidate's signed source identity.

Signed Fedora 44 candidate **`1.5.2-f49eb0631053`** was admitted against that full
public source SHA. All six downloaded subjects passed provenance verification;
checksums, archive metadata and all 502 manifest entries verified. Archive
SHA-256: `65628b1ce534aafb91cc0bfcd2c4e8cc4301005869003cb479fde686deceec1a`.
The exact package also passed its runtime self-check and both real-GIO launcher
fixtures on the AI server. No guest was started for this package verification.

After explicit saved-editor/installation approval, the candidate replaced the
desktop build. The previous locally patched build was retained and copied byte
for byte, with a verified whole-configuration snapshot, icon copy, normal backup
archive and integration snapshots. A pre-install guard first encountered one
pre-existing generated Python cache file outside the old manifest; it stopped
before replacement and restored the old service. The retry preserved that file
as backup data while checking every declared file and the known launcher override.
The new candidate's manifest/signature checks were not relaxed. Raw rollback is
available for the retained modified build; ordinary signed-manifest rollback
validation must not be assumed to accept its local override.

Machine checks confirmed both Decks connected with input/display ready, preserved
saved layouts, pages, brightness and selection, unchanged startup enablement and
the existing Homebridge companion ready. All 502 installed manifest files matched.
The maintainer then confirmed repeated website/application/safe-system keys keep
working, and both Decks' independent pages, displays, audio/media controls and
Homebridge work correctly. A final read-only check found the same two service
processes, zero automatic restarts and no launcher uncertainty/block events since
candidate startup. This is immediate targeted acceptance, not a long-duration
stability result or complete V1.5.2 release/distribution acceptance.

Private receipts in `outputs/launch-protocol-20261022/` include
`build-outcomes.json`, `candidates/fedora44/candidate-admission.json`,
`signed-package-result.json`, `desktop-install-2/result.json`,
`desktop-runtime-verification.json` and `physical-acceptance.json`. The initial
guard-stop receipt and both rollback/configuration copies remain retained.

## Review and release boundary

The approved Fedora desktop now runs the signed candidate above, including both
isolation layers and the diagnostics. Keep the previous patched build and complete
backups; no cleanup or release publication was requested. Observe ordinary use
without resuming development automatically. Further desktop replacements and
public/support promotion require their own approval. Keep uncertain failures
distinguishable from USB transport problems.

After that targeted reliability work, resume the outstanding Ubuntu physical
Firefox-restart retest and independent Debian qualification only when the
maintainer resumes active testing. No support or public-release claim changes
with this checkpoint.
