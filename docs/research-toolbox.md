# Decksmith research toolbox

Added October 11, 2026. These are optional development tools, separate from
Decksmith's application, daemon, installer and release dependencies.

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
