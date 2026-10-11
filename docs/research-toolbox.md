# Decksmith research toolbox

Added October 11, 2026. These are optional development tools, separate from
Decksmith's application, daemon, installer and release dependencies.

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

## Verification and location

An original tiny C fixture verifies import, function discovery and decompilation
without executing the fixture or inspecting proprietary binaries. The private
server setup report records the installed paths, package digests and smoke result.
No background Ghidra service, network listener or system-wide Java change is needed.

## Primary references

- [Ghidra repository](https://github.com/NationalSecurityAgency/ghidra)
- [Ghidra 12.1.4 release](https://github.com/NationalSecurityAgency/ghidra/releases/tag/Ghidra_12.1.4_build)
- [REA](https://github.com/morluto/rea)
- [RepoWise repository](https://github.com/repowise-dev/repowise)
- [RepoWise language support](https://github.com/repowise-dev/repowise/blob/main/docs/layers/LANGUAGE_SUPPORT.md)
- [RepoWise package requirements](https://github.com/repowise-dev/repowise/blob/main/pyproject.toml)
- [Elgato HID documentation](https://docs.elgato.com/streamdeck/hid/intro/)
- [Elgato Stream Deck SDK](https://docs.elgato.com/streamdeck/sdk/)
