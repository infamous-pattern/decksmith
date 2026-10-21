# Decksmith research toolbox

Added October 11, 2026. These are optional development tools, separate from
Decksmith's application, daemon, installer and release dependencies.

## Archify — reference adopted; post-acceptance trial planned October 21

[Archify](https://github.com/tt-a1i/archify) is an approved optional development
and documentation reference. The maintainer approved a bounded trial after
physical acceptance while keeping Decksmith paused. This entry authorizes no
trial execution during the pause, global skill installation, agent hooks or
application/daemon/installer dependency.

Reviewed main at `25cf37080fc8b1dbe758a4efc3935b55f3327083`; the latest observed
release was [v3.2.0](https://github.com/tt-a1i/archify/releases/tag/v3.2.0), and the
reviewed package declares version 3.2.0. This is a pinned main-source assessment,
not execution or release-artifact verification. The repository is MIT-licensed;
retain applicable notices if source/assets are later reused and review bundled
third-party notices separately. No Archify code was installed or run. Private
source hashes and review material are in `outputs/archify-review-20261021`.

Archify turns agent-authored typed diagram data into explorable HTML, including
architecture, sequence, workflow and lifecycle views. It does not independently
discover or certify Decksmith's architecture. Its
[repository evidence implementation](https://github.com/tt-a1i/archify/blob/25cf37080fc8b1dbe758a4efc3935b55f3327083/archify/renderers/shared/repository-evidence.mjs)
verifies source references against committed Git bytes. The
[authoring reference](https://github.com/tt-a1i/archify/blob/25cf37080fc8b1dbe758a4efc3935b55f3327083/archify/references/repository-authoring.md)
requires tracing actual ownership and preserving uncertainty. These checks can
establish the cited revision/files/ranges, not that our explanation is complete,
our inferred relationships are correct, or hardware behavior is qualified.

Useful Decksmith subjects are the paired runtime (editor, coordinator, independent
profiles and shared audio providers), selected-device draft/Save and Apply state,
and later recovery/qualification workflows. Retain the native Rust/Python/GTK
architecture and accepted interface. This is communication tooling, separate from
XERJ retrieval, the proposed RepoWise assessment and the TCA-inspired state trial.
Use a simpler existing diagram when interaction/source cards add little value.

### Bounded trial after physical acceptance and active-work resumption

Prepare two artifacts on a clean, pinned source copy: one multi-device runtime
architecture diagram and one lifecycle diagram for the selected-device editor
assignment/save flow. Keep editable diagram data, generated HTML, exact source
revision and validation receipts together. Use only selected source and sanitized
documentation; exclude private settings, credentials, recordings, transcripts and
raw host/guest evidence. No hosted publishing or automatic workspace indexing.

Compare both artifacts with a simple Mermaid/static baseline for the same scope.
Check whether readers can identify control ownership, independent device state,
the normal save path, failure/draft retention, stale replies and disconnect/recovery
without misleading edges or excess detail. Recheck every asserted relationship
in current source; the lifecycle must match observed/qualified behavior rather
than imply the later state refactor already exists. Make one representative source
change and measure update effort, stale references, useful interaction, readability,
output size and tooling/resource overhead. Inspect the actual rendered artifacts
and distinguish automated validation from human visual/semantic review.

Use the exact sanitized public GitHub commit in a matching public checkout for
web source links. Reviewed web providers include GitHub, Gitee and GitLab, not
Gitea; private Gitea/SSH references use `local-only`. Do not relabel a canonical
Gitea revision as its different public-history GitHub commit or rewrite origins
merely to pass the validator. Reconfirm this behavior against the chosen trial pin.

The reviewed [package](https://github.com/tt-a1i/archify/blob/25cf37080fc8b1dbe758a4efc3935b55f3327083/archify/package.json)
requires Node.js 18 or later; inspect browser-check requirements before the trial.
Use a disposable local tool copy and output directory with optional update checks
disabled through `ARCHIFY_UPDATE_CHECK_DISABLED=1`; no global installation,
automatic updates or persistent preview watcher. A later explicit local preview
may use its loopback mode, then must be stopped. Validate before replacing an
existing artifact and retain the last usable output; the reviewed publication
helper describes recoverable replacement, not guaranteed crash-atomic replacement.

Retain Archify only if these two examples are clearer and reasonably maintainable
compared with the simpler baseline. Otherwise keep the diagrams and existing tools
without adopting another standard dependency. Broader release/recovery diagrams,
CI integration and public visual documentation are separate follow-ups. Trial
approval does not replace physical acceptance or authorize release promotion.

## Boatswain and metric-aware design — references adopted October 20

The maintainer approved adding [Boatswain](https://gitlab.gnome.org/World/boatswain)
and the recommendations from the October 20 review to the project plan. No earlier
Boatswain assessment was found in the recorded toolbox/reference refresh. This is
a source and interface-definition review, not runtime or physical qualification.
No Boatswain code was built, installed or executed.

Reviewed main at `077e6c0291e068b2ffc0c7db972af0df6ccfb5cd` (October 14).
The latest observed release was 6.0, tagged at
`7569a5d6f951003f6468fbc0f0442575f05de06d`; reviewed main identifies itself as
development version 6.1. Main-source findings do not establish the exact behavior
of the stable package. The [official GNOME listing](https://apps.gnome.org/Boatswain/)
describes pages/profiles, media and soundboard actions, OBS control and network
requests. Boatswain's C/GTK4/libadwaita implementation is a native GNOME reference;
keep Decksmith's existing Rust/Python/GTK architecture and project-owned device
contract. Its GPL-3.0-or-later code/assets remain reference material for independent
implementation, outside Decksmith's Apache-2.0 source and release dependencies.

Study these bounded mechanisms:

- **Selected-control inspector and empty states:** the
  [device editor](https://gitlab.gnome.org/World/boatswain/-/blob/077e6c0291e068b2ffc0c7db972af0df6ccfb5cd/src/bs-device-editor.ui)
  uses a clamped device area, a selection-dependent inspector and a "Select a
  button" empty state. Its window also distinguishes no-device and device views.
  Refine our existing chooser/inspector and disconnected feedback; do not add a
  second dense canvas or replace accepted navigation and scroll markers.
- **Explicit cross-device profile action:** the
  [switch-profile action](https://gitlab.gnome.org/World/boatswain/-/blob/077e6c0291e068b2ffc0c7db972af0df6ccfb5cd/src/plugins/default/default-switch-profile-action.c)
  resolves a device serial and profile ID. This is a useful later target-selection
  reference, not evidence of linked navigation groups or atomic paired page
  changes. Keep missing targets explicit; do not adopt silent profile substitution.
- **Ordered multi-actions:** the
  [sequence implementation](https://gitlab.gnome.org/World/boatswain/-/blob/077e6c0291e068b2ffc0c7db972af0df6ccfb5cd/src/plugins/default/default-multi-action.c)
  includes delays and cancellable execution. Study the editor and execution model
  for the existing deferred V2 sequence idea; independently define lock,
  disconnect, page/context change and held-input cleanup. Never automatically
  retry a step with uncertain external side effects.
- **OBS and soundboard candidates:** OBS scene/source controls are a possible V2
  integration, subject to a concrete use case. Soundboard
  [preferences](https://gitlab.gnome.org/World/boatswain/-/blob/077e6c0291e068b2ffc0c7db972af0df6ccfb5cd/src/plugins/soundboard/soundboard-play-action-prefs.ui)
  expose Play/Stop, Overlap, Restart, Loop/Stop and Press-and-Hold plus volume.
  Keep these optional later features, with explicit connection/output targets,
  missing-resource states and safe cancellation rather than expanding this release.
- **Interactive emulated devices:** the
  [mock provider](https://gitlab.gnome.org/World/boatswain/-/blob/077e6c0291e068b2ffc0c7db972af0df6ccfb5cd/lib/devices/mock/mock-device-provider.c)
  is a reference for extending our existing VirtualDeck into interactive editor
  testing with multiple geometries and connection states. Preserve the planned
  local virtual Deck scope and independent physical acceptance.

The accompanying UX Collective article,
["UX designers are not designing interfaces, they design personalities"](https://uxdesign.cc/ux-designers-are-not-designing-interfaces-they-design-personalities-d94d874bb43d),
argues that metrics shape users' behavior and should be treated as partial
representations, not complete measures of success. Our application of that idea
is calmer, task-oriented feedback: clear selected Deck/page/target, distinct
draft/saved/applied/connection states, actionable recovery and predictable
automation with manual control. Keep configured volume separate from live signal;
neither is an overall audio-quality score. Keep diagnostics optional and avoid
adding productivity scores, streaks or a metrics dashboard. These are Decksmith
design recommendations, not claims that the article discusses this application.

**Ordering:** corrected Ubuntu physical acceptance remains first, then the
approved editor-state trial and scoped target/state-feedback work. Interactive
virtual-device testing remains a V2 development capability; linked navigation,
multi-actions, OBS and soundboard stay later candidates with separate design and
qualification. Existing independent pages, media controls, artwork and copy tools
are not proposed as new features. Private pinned source hashes/review evidence
are retained in `outputs/boatswain-review-20261020`; no third-party source is
included in this repository.

## XERJ — development-search assessment, October 20

[XERJ](https://github.com/xerj-org/xerj) is an optional development-search trial
candidate. The maintainer requested assessing its use in our development process
or project. This entry records the assessment and subsequently approved CLI trial. XERJ
remains optional, with no global/agent integration. It adds no application,
daemon, installer or release dependency.

Reviewed main at `764e7a9b69f393228b1af638cbf184538b590e56`; latest release observed
was [v1.0.0-rc.97](https://github.com/xerj-org/xerj/releases/tag/v1.0.0-rc.97),
from `e6216b6597ca11b816caa4e3e0ca2ffdf73d9a2b`. These are different revisions.
The repository is Apache-2.0. The initial assessment executed no upstream code;
the later trial below used the pinned release. Main documentation/source does
not establish a released binary's exact behavior.

Useful Decksmith questions are locating Rust dispatch and Python editor paths
for one behavior, connecting qualification notes to selected failure evidence,
and finding previously reviewed mechanisms in pinned reference repositories.
XERJ offers ranked text/code retrieval and optional vector/hybrid search. Start
with its local keyword/code retrieval; its default lexical embeddings are not a
neural model. Upstream token/cost benchmarks are not Decksmith measurements.
Simple exact-symbol/file lookups continue to use `rg` when sufficient.

Trial privately on the AI server against a curated source snapshot and selected
project documents, followed by explicitly selected sanitized evidence summaries.
Do not index the workspace root, home directory, private profiles, credentials,
voice recordings, VM disks, binaries, transcripts or all raw test outputs. Copy
only allowlisted regular files; retain original path, source commit and content
hash. Keep third-party reference corpora separate, pinned and license-labelled;
retrieval is evidence for independent implementation, not permission to copy.

Start through the CLI, with a disposable index, explicit resource/time budgets,
local storage and authenticated loopback access. Use a separate scoped read-only
search credential if later connected to an agent; verify denied writes against
the chosen release. Keep administrative ingest credentials separate. Do not run
`xerj init`: the reviewed implementation can edit agent configuration and
`AGENTS.md`. Do not enable automatic memory writes, watchers, hosted corpus
fallback, feedback submission, external reranking/proxy embeddings, WAL export,
cloud storage or clustering for this trial. Inspect effective settings and
network behavior rather than assuming that "local-first" means no egress.

The reviewed MCP source includes mutation tools, optional automatic admin-key
loading and external field/corpus feedback tools. Its feedback text is upstream
content, not authorization to publish a report. Verify opt-out and access controls
against the pinned trial binary; main/source features may differ from rc.97.
References: [security model](https://github.com/xerj-org/xerj/blob/764e7a9b69f393228b1af638cbf184538b590e56/docs/SECURITY_MODEL.md),
[MCP source](https://github.com/xerj-org/xerj/blob/764e7a9b69f393228b1af638cbf184538b590e56/engine/crates/xerj-mcp/src/lib.rs),
[init source](https://github.com/xerj-org/xerj/blob/764e7a9b69f393228b1af638cbf184538b590e56/engine/crates/xerj-autoindex/src/init.rs)
and [outbound-data controls](https://github.com/xerj-org/xerj/blob/764e7a9b69f393228b1af638cbf184538b590e56/docs/RERANK.md).

Compare ten preselected real Decksmith questions with targeted `rg` plus file
reads: relevant results among the first five hits, correct file/line evidence,
misses, end-to-end lookup time, actual tool/context volume where measurable,
initial/incremental indexing time, CPU, peak memory and disk use. Include changed
and deleted files to detect stale results. Recheck returned passages in their
original current files. Retain XERJ only if that comparison shows enough practical
benefit to justify the extra service and index; do not promise a token reduction.
This trial complements physical acceptance and the later editor-state trial;
it does not replace tests or delay Ubuntu qualification. No installed desktop,
VM, scheduled job or global agent settings was changed by the initial
assessment. Private review evidence is under `outputs/xerj-review-20261020`.

### Bounded CLI trial result — October 20

After explicit maintainer approval, the assessment/plan was committed and pushed
to both repositories, then rc.97 was trialled privately on the AI server. Its
Linux musl release archive matched the published checksum and GitHub asset digest;
this is integrity verification, not a signed-provenance claim. The corpus was
385 files / 3,560,010 bytes: allowlisted tracked text from canonical source
`35e235d76325dde06c200e59972a4681b4a02ecb` (public
`fd49ffd1b749cc90cec70b0be1d59d8ec3d10205`) plus one sanitized candidate summary.
No third-party reference corpus or private settings/transcripts was indexed.

- Initial indexing completed in 25.73 seconds. Across ten preselected questions,
  an expected implementation file appeared among the first five XERJ results
  for eight, versus seven for targeted `rg` plus bounded file reads. XERJ missed
  the expected stale-meter and permanent-device-enrollment targets. Both tools
  can require query refinement; file retrieval is not a solved-task score.
- Median process/query latency was 7.01 ms for XERJ versus 5.85 ms for `rg` and
  bounded reads. Natural-language queries and expert regexes require different
  prior knowledge, so this small comparison does not establish overall coding
  speed or token/cost savings. No actual model token usage was measured.
- Forty-nine of fifty passages matched source text exactly; one Python property
  signature normalized indentation and was checked against its file/definition
  line. Returned evidence still requires current-source verification.
- A separate disposable fixture passed changed/deleted-file reconciliation:
  the old symbol disappeared, the new symbol appeared, then deletion removed it.
  This does not qualify every file format, watcher or corpus-update pattern.
- Anonymous reads returned 401. A scoped reader could search, while document
  writes, index deletion and memory writes returned 403. The processes had only
  loopback in a separate network namespace and could not see the host home.
  No MCP client, automatic init, hosted processing, reporting or memory store
  was connected; the copied corpus and binaries were mounted read-only.
- The indexing/comparison cgroup reached its 2 GiB memory ceiling and completed;
  that peak includes CLI/indexing helpers and cache, not just server RSS. An
  immediate post-index sample was busy and is not an idle benchmark. A separate
  restart with no queries, ten seconds settling and three twenty-second samples
  averaged 0.15% of one CPU core with about 45 MiB server RSS. This is a short
  controlled check, not long-running production qualification. The retained
  private directory occupied about 177 MiB including binaries, data and evidence.

The first harness attempt used an unsuitable readiness endpoint; XERJ started
and shut down normally. The corrected probe used its authenticated ES root.
Both attempts are retained. Both trial services exited and were collected; no
background XERJ process remains. Existing server schedules, guests, desktop
controls and agent settings were unchanged. Private results and reproducible
harnesses are in `outputs/xerj-trial-20261020`.

**Decision:** retain as an optional on-demand CLI lookup aid for unfamiliar code
and cross-document questions. Keep `rg` for exact/local lookups. The modest
retrieval improvement does not justify making XERJ mandatory or always running.
A later read-only agent integration or reference-corpus expansion remains a
separate bounded decision; no runtime/release dependency or physical acceptance
claim follows from this trial.

## October 19 reference refresh

Rechecked 21 documented/history/discussion repositories, including our public
repository and infrastructure references. The
[refresh report](checkpoints/2026-10-19-reference-refresh.md) records exact heads,
release/main distinctions, current Decksmith overlap and proposed additions.
Deckhand is 53 commits beyond the original review, with v0.8.1 released;
Deck-Switch is unchanged. Impeccable is 62 commits beyond the original pin.
The original assessments below remain dated review records, not current-version
claims. Cross-device Copy/Paste is now implemented and physically accepted on
Fedora. On October 19 the maintainer approved adding the report's recommendations
to the roadmap, with post-physical-acceptance ordering and conditional/V2 trial
boundaries retained. This does not mark the proposed tools or features qualified.
No upstream tools, hooks or application code were installed or run.

## The Composable Architecture (TCA) — reference adopted October 19

[TCA](https://github.com/pointfreeco/swift-composable-architecture) is an approved
architecture and testing reference. The maintainer requested its addition and a
bounded editor-state trial after physical acceptance. Retain Decksmith's Rust
daemon and Python/GTK editor; do not add the Swift library, Apple UI frameworks
or a new runtime dependency.

Reviewed at `bc2db5ba8ad3a47deba5db32fa340637ba6c9a76`; latest release observed
was 1.26.2. This is an architectural reference, not a package selected for use.

Study explicit state/actions and state transitions, feature composition,
dependency-controlled effect tests and cancellation of obsolete asynchronous
work. Adapt the principles independently to selected-device ownership, per-device
drafts, validation, Save and Apply, discovery responses and disconnect/reconnect.
Decksmith already has device-addressed clients and stale-context rejection;
preserve those safeguards rather than duplicating or weakening them.

Start with one editor assignment/save flow after physical acceptance. Compare the
existing behavior with a small explicit state model, fake transport and controlled
responses. Exercise device switching during a request, save failure with draft
retention, disconnect/reconnect, and rejection of obsolete results. Preserve the
accepted interface, saved-data format, undo/redo and explicit cross-device copy.
Record added complexity and retain the approach only if it clarifies behavior
and improves meaningful test coverage. No trial or application refactor has been
performed. This is not a replacement for physical or resource acceptance.

## Impeccable — design guidance adopted October 14

[Impeccable](https://github.com/pbakaus/impeccable), reviewed at commit
`2f98774f86bc05563876643db37e6c37c322850f`, is an approved part of Decksmith's
development toolbox for design-review guidance. Its Apache-2.0 repository includes
audit, polish, clarity and hardening
procedures. Source and third-party notices were inspected; no installer, binary,
hook or bundled skill was installed or executed during this review.

Use its principles to review task clarity, consistent spacing, long labels and
localization expansion, empty/unavailable states, focus, contrast and image
handling. Preserve Decksmith's accepted navigation/layout and GNOME theme. For
native editor reviews, adapt these checks to GTK/libadwaita, Orca, keyboard
navigation and real window sizes; validate actual behavior rather than treating
a source scan as acceptance.

Its deterministic/browser tools are aimed at web interfaces. The October 14 native
audit is for iOS/Android/adaptive mobile apps, with no GTK/GNOME reference, so
neither path qualifies our desktop application directly. Web examples, custom
font/color advice, motion recommendations and mobile target sizes are not
automatic requirements for Decksmith. Browser tooling may be useful separately
for the optional plugin web panel and HTML mock-ups after a bounded trial.

The October 19 refresh found an explicit native-review path that skips the web
detector, plus newer contrast and text-clipping checks. Adapt the review questions
to actual GTK behavior; the DevTools/CDP desktop scanner is for web-based apps,
not this editor. See the refresh report for the new pin and limits.

The user approved using these ideas throughout the project on October 14. For
interface changes, identify the user's task and required states, review hierarchy
and wording, then check the real interface at representative window sizes with
long labels, keyboard navigation, Orca and light/dark GNOME themes. Prioritize
task-blocking or confusing behavior over decoration; verify performance changes
with measurements. Apply this process to the V1.5 device chooser and independent
device status, and to later editor and plugin-panel work.

Keep the reference outside runtime/release dependencies. A full installation or
hook trial is a separate decision; adopting its guidance does not add automatic
code-edit hooks or a new UI framework. No performance or design-quality
improvement is claimed from adoption alone.

## Deck-Switch and Deckhand — reference repositories adopted October 15

Both repositories are approved references for specific feature and architecture
questions. Source, documentation and representative editor views were inspected;
no foreign code, installer or dependency was executed or added to Decksmith.
Retain Rust/GTK, the simplified editor and independent device profiles.

- [Deck-Switch](https://github.com/LabRaTox/Deck-Switch), reviewed at
  `bce843ae6d7dcad55707aa6076611556d7d43970`: useful capability/dependency reasons,
  preserved unavailable assignments, declarative localized plugin settings,
  named Dial Stack entries and shared system-metric snapshots. Its MIT license
  requires notices if code is ever reused; dependencies need separate review.
  Its Python/Tauri stack, in-process plugin loader, KDE/Arch assumptions and
  periodic animation loop are not Decksmith implementation templates. Its
  long-dial-push stack cycling conflicts with our approved tap-to-cycle and
  microphone hold rules; retain our semantics.
- [Deckhand](https://github.com/juicetheforce/deckhand), reviewed at
  `0ca5bd9b7e08a8988d6c329f65ad83323d430296`: useful event-driven cached audio/media
  inventory, device-scoped failure badges, duplicate-replacing actionable
  notifications, and geometry-aware bulk copy with explicit skipped controls and
  navigation warnings. Its GPL-3.0-or-later source remains reference material;
  implement ideas independently rather than copying code into Decksmith's
  Apache-2.0 distribution. It has no plugin host and does not implement Plus
  dial/touch-strip controls. Its global profile switching differs from our
  independent layouts. Periodic safety scans still exist, and its published CPU
  figure is not a verified benchmark for Decksmith's live gauges.

Use these ideas selectively: capability/failure clarity and cached observations
can support multi-device qualification. Cross-device copying was follow-up work
at the original review and is now implemented; clearer Dial Stack lists remain
follow-up work. Plugin field schemas and shared built-in
system monitors now belong to V2. Keep device defaults independent, copying
explicit, errors actionable and the accepted native layout stable. Repository
review is not physical, lifecycle or performance acceptance.

## Choose the simplest source of evidence

Start with official Elgato HID/SDK documentation, saved test profiles and observed
behavior in the Windows review VM. Validate device behavior on actual hardware.
Use reverse-engineering tools only for a specific question these sources leave
unanswered; static findings do not establish complete runtime behavior.

## REA

Use REA to locate readable plugin handlers, configuration fields and native
references. The October 9 trial identified Dial Stack configuration and Wave Link
display/rotation patterns, but its JavaScript graph was partial and its native
analysis did not decompile the relevant functions. Review the private trial report
and evidence before reusing findings. Existing REA setup remains on the workstation.

## Ghidra

Use Ghidra for focused inspection of compiled native executables or libraries:
function references, disassembly and decompilation where needed. It complements
REA, which already used Ghidra during the earlier native trial.

The AI server has a separate user-owned Ghidra 12.1.4 installation and isolated
Temurin JDK 21.0.12.1+1, selected for that release's Java requirement. Official
package checksums are recorded and verified before extraction. Development-branch
requirements may differ; verify a release's requirements before updating tools.

The private `decksmith-ghidra` launcher creates a distinct run/project, copies and
hashes its input, and keeps logs and outcomes. It imports the binary for analysis
without executing it. It does not overwrite existing analysis projects. An
analysis timeout and an outer process timeout prevent forgotten analysis jobs;
only that run's process group is terminated on timeout or interruption. Analysis
completion alone is not proof of a hypothesis: inspect the evidence and confirm
behavior through reference-app or device tests. Run only reviewed analysis scripts.

Research runs are manual and do not join scheduled compatibility tests. Keep
reference binaries, recovered implementation/assets, profiles and generated
decompilation in private research storage, outside Decksmith's Git/public history.
Implement Decksmith behavior and artwork independently; record the evidence and
uncertainties needed to reproduce a finding.

Useful initial questions are ambiguous Dial Stack touch/press/hold rules and
undocumented native behavior. For MK.2 and multi-device work, documented HID
commands and model-specific hardware tests remain the first choice.

## RepoWise — evaluation planned

RepoWise is a candidate for understanding Decksmith's own source and keeping its
documentation aligned with implementation. Its repository advertises code and
dependency graphs, Git change analysis, documentation drift detection and a local
CLI/dashboard/MCP interface. Rust and Python have full language-analysis support
in its published matrix. These capabilities are not yet verified against
Decksmith's GTK callbacks, dynamic Python/GI behavior or D-Bus interactions.

The reviewed source reports version 0.55.0, Python 3.11 or newer and alpha
development status. It uses AGPL-3.0-or-later licensing and has a substantial
analysis dependency set. Keep it in an isolated development environment on the
AI server; do not embed it in Decksmith or add it to application, installer or
release dependencies. Recheck the selected release and requirements before use.

For the first trial, use a clean tracked-source copy, excluding private settings,
research inputs, build caches and generated outputs. Use deterministic local
analysis without model calls or prose generation. Do not enable transcript
capture, hooks or automatic agent/MCP configuration changes. Review initialization
behavior before running it; repository setup instructions do not themselves
authorize configuration changes.

Compare its file citations and answers with direct source inspection for three
questions: which installer/runtime-doctor paths define dependencies; how a key
action travels from editor configuration to daemon dispatch; and which page/dial
state dependencies matter for independent devices. Record false positives,
missed dynamic connections, indexing time, peak memory and refresh behavior.
Retain it only if it provides accurate, useful results. Static graph findings
complement tests and physical acceptance; they do not replace them. No trial,
installation or integration has been completed yet.

## Awesome LLM Apps — development skills to evaluate

Use this collection as a source of optional development-review skills. Initial
candidates are Dependency Doctor for direct Python/JavaScript manifest issues,
Scope Creep Detector for comparing a change with its stated intent, and Commit
Archaeologist for tracing the history of a source region. Start with an isolated,
read-only trial against a clean Decksmith source copy; compare findings with
direct inspection before adopting a skill. No skills have been installed or
qualified for Decksmith yet.

Dependency Doctor does not cover Rust manifests, Fedora/Debian OS packages,
GTK/PipeWire runtime readiness or full vulnerability audits. Scope classification
is a heuristic, not proof that a necessary cross-cutting change is unrelated.
These tools complement the existing checks and fresh-VM installer acceptance;
they are not application dependencies or replacements for those tests.

## Awesome MCP Servers — discovery reference

Use this catalog to find candidates for development, monitoring, desktop and
home-automation research when a specific task warrants another tool. A catalog
entry does not establish security, maintenance quality, Linux compatibility or
Decksmith plugin compatibility. Review each candidate's own implementation,
license, permissions and resource use, then test it in isolation. No server,
automatic MCP configuration or Decksmith integration is enabled by listing it
here. Keep the catalog outside runtime and installer requirements.

## Agents Towards Production — educational reference

Consult the tracing, evaluation, secure tool-calling and failure-recovery tutorials
for ideas that may improve AI-server development jobs. The material targets GenAI
agents, so applicability to Decksmith's native UI, device transport and daemon
must be demonstrated for a specific question. Do not add its frameworks or cloud
services to Decksmith merely to follow an example.

The repository currently declares a custom non-commercial license. Treat it as
educational reference material; do not copy code into Decksmith's Apache-2.0
distribution without a separate license review. No tutorial installation or
integration has been completed. Recheck upstream licensing before any reuse.

## Verification and location

An original tiny C fixture verifies import, function discovery and decompilation
without executing the fixture or inspecting proprietary binaries. The private
server setup report records the installed paths, package digests and smoke result.
No background Ghidra service, network listener or system-wide Java change is needed.

## Primary references

- [Deck-Switch reviewed source](https://github.com/LabRaTox/Deck-Switch/tree/bce843ae6d7dcad55707aa6076611556d7d43970)
- [Deckhand reviewed source](https://github.com/juicetheforce/deckhand/tree/0ca5bd9b7e08a8988d6c329f65ad83323d430296)

- [Ghidra repository](https://github.com/NationalSecurityAgency/ghidra)
- [Ghidra 12.1.4 release](https://github.com/NationalSecurityAgency/ghidra/releases/tag/Ghidra_12.1.4_build)
- [REA](https://github.com/morluto/rea)
- [RepoWise repository](https://github.com/repowise-dev/repowise)
- [RepoWise language support](https://github.com/repowise-dev/repowise/blob/main/docs/layers/LANGUAGE_SUPPORT.md)
- [RepoWise package requirements](https://github.com/repowise-dev/repowise/blob/main/pyproject.toml)
- [Awesome LLM Apps](https://github.com/Shubhamsaboo/awesome-llm-apps)
- [Dependency Doctor scope](https://github.com/Shubhamsaboo/awesome-llm-apps/blob/main/agent_skills/dependency-doctor/SKILL.md)
- [Scope Creep Detector](https://github.com/Shubhamsaboo/awesome-llm-apps/blob/main/agent_skills/scope-creep-detector/SKILL.md)
- [Awesome MCP Servers](https://github.com/punkpeye/awesome-mcp-servers)
- [Agents Towards Production](https://github.com/NirDiamant/agents-towards-production)
- [Agents Towards Production license](https://github.com/NirDiamant/agents-towards-production/blob/main/LICENSE)
- [Elgato HID documentation](https://docs.elgato.com/streamdeck/hid/intro/)
- [Elgato Stream Deck SDK](https://docs.elgato.com/streamdeck/sdk/)
