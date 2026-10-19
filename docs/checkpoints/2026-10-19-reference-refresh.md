# Reference repository refresh — October 19, 2026

The maintainer requested TCA in the toolbox, an editor-state trial after physical
acceptance, and a fresh review of GitHub repositories in the project history,
especially Deckhand. TCA and that bounded trial are now recorded in the
[toolbox](../research-toolbox.md) and [roadmap](../ROADMAP.md#current-work--monday-october-19).
After this review, the maintainer approved adding the other recommendations to
the project plan on October 19. The roadmap records their order, evidence and
conditional/V2 scope. They are planned work, not implemented or qualified features.
Ubuntu physical acceptance remains first.

## Scope and method

Reviewed the 21 repositories identified in tracked documentation, baseline
references, the Git history of those documents, and the earlier yasdec feedback
link, including Decksmith's own public repository and infrastructure references.
Checked live repository heads, latest releases, recent commits and selected
primary source files. Compared Deckhand, Deck-Switch and Impeccable with their
previously recorded review revisions. This is a bounded reference review, not an
audit of every dependency, every catalogue entry or every upstream implementation.

Private evidence is retained outside product source at
`outputs/research-reference-refresh-20261019/`: `reference-inventory.json`,
`upstream-status.json`, `recent-commits.json`, three comparison responses and
`source-evidence-index.json`. Its source snapshots were read as data. No upstream
installer, hook, binary or application code was executed; no desktop, guest,
bridge, service or dependency was changed. Static review does not establish
hardware compatibility, performance or security acceptance.

## Recommended order

| Priority / timing | Proposed change | Why it helps Decksmith |
| --- | --- | --- |
| First after physical acceptance; already approved trial | Explicit editor state and controlled asynchronous tests, using TCA principles and Deckhand's stale-reply example | Preserve each device's draft and make failed saves, switches and obsolete replies predictable without changing the simplified interface. |
| Next product improvement; extends an existing approved plan | Consistent action-failure feedback on the physical control and in its editor inspector | Explain what failed and how to recover; keep the reason until success or reassignment, and combine repeated optional notifications. |
| Small follow-up proposal | Manual update check in About, with optional checks when the editor opens | Show installed build, supported OS/channel and an appropriate signed release link, without a daemon polling loop or automatic installation. |
| Regression coverage alongside editor work | Protect saved captions/artwork from late provider events; validate artwork inputs at the boundary | Prevent a status refresh from replacing user choices and give unsupported inputs a clear, safe failure. |
| Ongoing design review | Refresh native Impeccable checks for visible text, contrast, focus and recoverable states | Verify Save and Apply, scroll markers, long labels and device status at actual GTK window sizes instead of adding panels. |
| Conditional integration follow-up | Homebridge empty-discovery recovery and brightness-range scenarios | Prove that temporary inventory absence recovers and controls retain the right target; review Govee updates separately if that plugin is installed. |
| Optional development trial | RepoWise on a clean source copy before expanding the editor refactor | Test whether current Python/Rust analysis gives useful, verifiable change-impact guidance; installation and evaluation remain separate work. |
| V2 exploration | An interactive local virtual Deck | Let people try assignments or use a spare touchscreen without hardware; build on the existing test backend only after a separate permissions, lock and focus design. |
| V2, with plugin work | A catalogue admission checklist and declarative settings | Check bundle identity, permissions, dependencies, versions and assets before installation; a catalogue listing is not a trust or compatibility guarantee. |
| Optional later visual work | Selected system-monitor/media icons and optional stereo/spectrum meters | Add only when the corresponding feature needs them; retain current gauges and saved artwork by default and measure the cost. |

## Deckhand: the most useful changes

The previous review used `0ca5bd9b7e08a8988d6c329f65ad83323d430296`.
The refreshed head is `e0b8fba16e9c62e4798b5eb6018ec50f492d511e`, **53 commits ahead**;
the latest release is [v0.8.1](https://github.com/juicetheforce/deckhand/releases/tag/v0.8.1),
published October 18. The findings below distinguish that release from other
changes across the comparison range.

**Complete our existing failure-feedback plan.** Its
[failure registry](https://github.com/juicetheforce/deckhand/blob/e0b8fba16e9c62e4798b5eb6018ec50f492d511e/src/key-failures.ts)
addresses a failure by device/profile/page/control. The same last failure appears
on the physical key and in the editor; page changes and reconnection do not hide
it. Success or editing/removing the assignment clears it. The state is in memory,
not a new persistent history. Its
[notification service](https://github.com/juicetheforce/deckhand/blob/e0b8fba16e9c62e4798b5eb6018ec50f492d511e/src/services/notifications.ts)
combines rapid repeats into one counted notification per key.

Decksmith already has failure text/badges and an approved capability/failure
clarity plan. Extend and verify that plan across the permanent multi-device runtime
and inspector rather than creating a separate diagnostics dashboard. Give a
specific remedy for missing permissions, disconnected devices or unavailable
targets. Treat a timeout as uncertain execution, not permission to retry a
potentially non-idempotent action automatically. Test failure isolation, success
clearing, reassignment and reconnect behavior with no healthy-state notifications.

**Make update discovery simple.** Deckhand's
[update checker](https://github.com/juicetheforce/deckhand/blob/e0b8fba16e9c62e4798b5eb6018ec50f492d511e/editor/src/main/updates.ts)
checks once when the editor opens, if enabled. Its daemon does not poll and it
links to the release rather than downloading or installing it. Decksmith should
start with a manual About check and consider an opt-in opening check. Bound the
request, explain the GitHub connection and show an honest offline/failure state.
Match OS, architecture and stable/candidate channel: GitHub's generic latest tag
alone must not direct a Debian preview to a Fedora package. Preserve our signed
artifact verification and complete-backup recovery safeguards.

**Use the stale-reply fix as an editor test case.** v0.8.1 fixes a Twitch linking
response that could obscure a newer code/Cancel state. Its
[current settings implementation](https://github.com/juicetheforce/deckhand/blob/e0b8fba16e9c62e4798b5eb6018ec50f492d511e/editor/src/renderer/TwitchSettings.tsx)
keeps the event authoritative instead of applying the older reply. This is a
general lesson, not a proposal to add Twitch. Combine it with the approved TCA
trial: control response order during discovery, preview, save, device switching
and disconnect/reconnect. Decksmith already rejects obsolete contexts and
revision mismatches; extend sequence coverage without replacing those protections.

**Consider a user-facing virtual Deck in V2.** Its
[virtual driver](https://github.com/juicetheforce/deckhand/blob/e0b8fba16e9c62e4798b5eb6018ec50f492d511e/src/drivers/virtual.ts)
and [security boundaries](https://github.com/juicetheforce/deckhand/blob/e0b8fba16e9c62e4798b5eb6018ec50f492d511e/SECURITY.md)
support an interactive local display/touchscreen, explicit one-device pairing,
session/lock restrictions and reconnection. v0.8.1 adds a usable default geometry
for every profile and reports unavailable touch access honestly. Decksmith's
existing VirtualDeck is a testing backend, not this product feature. A trial
would need GNOME focus behavior, coordinate/geometry checks, scoped permissions,
display removal and lock/push-to-talk acceptance. It does not imply a browser or
phone remote-control service.

**Review input boundaries as hardening work.** Deckhand's icon loading rejects
non-regular/oversized files and protects its state directory; credential changes
are serialized with owned temporary files. Map those cases to Decksmith's current
import/artwork and companion configuration tests before proposing any code change.
These are useful checks, not demonstrated vulnerabilities in Decksmith.

Keep its limitations explicit. Its README reports physical testing on XL and
Original V2; its
[HID adapter](https://github.com/juicetheforce/deckhand/blob/e0b8fba16e9c62e4798b5eb6018ec50f492d511e/src/drivers/hid.ts)
deliberately ignores non-button controls such as Plus dials. It is not a reference
proving our Plus dial/touch behavior. Its global profile switching is not a reason
to change Decksmith's independent default layouts. Linked navigation remains a
separate opt-in follow-up. GPL source remains reference material; independently
implement ideas rather than importing it into the Apache-2.0 distribution.

## Other repositories with actionable lessons

**OpenDeck:** [current head](https://github.com/nekename/OpenDeck/commit/772189ca74c89a10bad219b67de620ecb07bf1f6)
fixes custom action images/text being overwritten by plugin events. That is
unreleased main, separate from
[v2.14.1](https://github.com/nekename/OpenDeck/releases/tag/v2.14.1), whose resume,
exit cleanup, image-quality and loopback-server changes are already released.
Decksmith already protects custom media labels and packaged artwork and has
physically accepted Fedora wake/quit behavior. Add focused asynchronous-provider
regressions, not a claim that those features are missing. The new MK.2 Scissor
Keys module upstream is a future model-qualification reference, not certification
of another hardware variant for Decksmith. Plugins remain V2; no GPL code import.

**Impeccable:** 62 commits beyond our October 14 pin, now reviewed at
`0665bd812ae0908fcab17b96fd4f840defb882d9`; latest CLI release is
[cli-v4.6.0](https://github.com/pbakaus/impeccable/releases/tag/cli-v4.6.0).
Its current native review skips the web detector; recent work improves visible
label/clipping analysis, contrast and OS dark-mode handling. Use those questions
on GTK screenshots, keyboard navigation and Orca. Its DevTools/CDP desktop path
applies to web-based applications, not Decksmith's GTK UI. No CLI, hooks or web
runtime are needed. Preserve our simplified hierarchy and obvious scroll markers.

**Homebridge Govee:**
[v11.39.1](https://github.com/homebridge-plugins/homebridge-govee/releases/tag/v11.39.1)
corrects brightness scaling for models using 0–100 versus 0–254 and serializes
scene requests with bounded rate-limit handling. Current main also keeps LAN
discovery running when the first scan finds nothing; that fix is not in the
release above. Apply the general empty-inventory/recovery scenario to our
integration tests. Confirm the actual installed plugin/model/version before any
maintenance recommendation; do not put vendor-specific behavior in Decksmith
or change a live bridge from this research request.

**RepoWise:** stable
[v0.57.0](https://github.com/repowise-dev/repowise/releases/tag/v0.57.0)
adds relevant Python lazy-export handling and Rust refactoring test-plan support;
later main includes additional exclusions and analysis fixes. This makes the
previously planned bounded development trial more interesting, not automatically
qualified. Use a clean local source copy, check findings manually against
GTK/GI/D-Bus behavior, and assess its substantial dependencies and AGPL license.
Do not enable automatic hooks, transcript ingestion or hosted analysis. It is not
an application dependency or a replacement for actual integration tests.

**Tabler / yasdec / OpenAction:** Tabler
[v3.49.0](https://github.com/tabler/tabler-icons/releases/tag/v3.49.0) is newer than
our v3.46.0 packaged subset; select useful memory/media/status icons only when
needed and preserve saved asset identity/notices. The earlier feedback-linked
[yasdec](https://github.com/neoyagami/yasdec) offers stereo/spectrum visualization
ideas, but excludes dial-based Decks and has GPL source. Treat it as an optional
visual experiment, not proof of low CPU or calibrated VU/true-peak metering.
[OpenAction's catalogue](https://github.com/OpenActionAPI/plugins) offers manifest,
artifact and asset-validation ideas for V2 admission checks; each plugin needs
its own compatibility, permission and license review. None is auto-approved.

## What not to add now

- Cross-device Copy/Paste, independent profiles, event-based discovery and
  reduced redundant rendering already exist. Recent reference changes can guide
  regressions; they are not new Decksmith feature proposals.
- Do not replace explicit device copying with linked files/global profiles,
  expand the current release into plugin work, or crowd the editor with a new
  diagnostics dashboard.
- No reason from this review to switch our icon system to Runeicons, adopt a
  web UI framework, install agent collections, or update reverse-engineering
  tools just because main has moved.
- Keep signed downloads and dependency/security gates. A checksum-only upstream
  installer is not an improvement over our verified candidate/release process.
- Nothing here promotes Ubuntu/Debian support or substitutes for pending physical
  acceptance. No live code tests were run during this documentation review.

## Repository inventory

The following table records the reviewed snapshot and disposition. Release tags
and main-branch commits are deliberately separate; a moving head is not a release.

| Repository | Reviewed head | Latest release observed | Disposition |
| --- | --- | --- | --- |
| [LabRaTox/Deck-Switch](https://github.com/LabRaTox/Deck-Switch) | [`bce843ae6d7d`](https://github.com/LabRaTox/Deck-Switch/commit/bce843ae6d7dcad55707aa6076611556d7d43970) | [v1.3.5](https://github.com/LabRaTox/Deck-Switch/releases/tag/v1.3.5) | Identical to prior review; retain capability/settings/Dial Stack references, no new addition. |
| [NationalSecurityAgency/ghidra](https://github.com/NationalSecurityAgency/ghidra) | [`f70f07d07ca6`](https://github.com/NationalSecurityAgency/ghidra/commit/f70f07d07ca6284c1f26557ad19071885ed8590b) | [Ghidra_12.1.4_build](https://github.com/NationalSecurityAgency/ghidra/releases/tag/Ghidra_12.1.4_build) | Latest release remains 12.1.4; main changes are not a reason to alter the existing targeted research setup. |
| [NirDiamant/agents-towards-production](https://github.com/NirDiamant/agents-towards-production) | [`49a03386d2ee`](https://github.com/NirDiamant/agents-towards-production/commit/49a03386d2eee50229b3f5083b0e43768041dc97) | No GitHub release observed | Recent marketing/docs changes; custom noncommercial terms remain. Educational reference only. |
| [OpenActionAPI/plugins](https://github.com/OpenActionAPI/plugins) | [`e57f4edea4cf`](https://github.com/OpenActionAPI/plugins/commit/e57f4edea4cf2601edde8c91a48fef4b690d1887) | No GitHub release observed | Recent catalogue additions/metadata validation; use admission-checklist ideas in V2, not automatic compatibility or installation. |
| [Runeicons/runeicons](https://github.com/Runeicons/runeicons) | [`2f2a86a3a96c`](https://github.com/Runeicons/runeicons/commit/2f2a86a3a96cbf5026e4276d349c50a139bd8a88) | No GitHub release observed | Recent site/sprite changes do not justify replacing Tabler. Icon artwork has Apache-2.0 terms; brand/site terms differ. |
| [RustSec/advisory-db](https://github.com/rustsec/advisory-db) | [`999ec89d5d3f`](https://github.com/rustsec/advisory-db/commit/999ec89d5d3f10f20be14750ec8c2d8b44e0ac9b) | No GitHub release observed | New maintenance/advisory entries; preserve live CI security policy. This refresh is not a dependency vulnerability audit. |
| [Shubhamsaboo/awesome-llm-apps](https://github.com/Shubhamsaboo/awesome-llm-apps) | [`874bc13a57aa`](https://github.com/Shubhamsaboo/awesome-llm-apps/commit/874bc13a57aa2ba1266a84f2444808ce3093efb6) | No GitHub release observed | CommitArchaeologist quoted-path fix is relevant to the planned optional read-only trial; no application feature or automatic skill installation. |
| [StreamController/StreamController](https://github.com/StreamController/StreamController) | [`0f439967a16a`](https://github.com/StreamController/StreamController/commit/0f439967a16a8bfdc859439b0ae2b1370185aa06) | [1.5.0-beta.15](https://github.com/StreamController/StreamController/releases/tag/1.5.0-beta.15) | Recent focus/backend handling remains a reference; event/cached rendering is already present in Decksmith. Foreign performance claims are not our benchmark. |
| [cli/cli](https://github.com/cli/cli) | [`fa92838bef95`](https://github.com/cli/cli/commit/fa92838bef95fe4a29c1587693885d5e9bdb6cf1) | [v2.102.0](https://github.com/cli/cli/releases/tag/v2.102.0) | Released 2.102.0 already matches our pinned portable verifier. Keep verification gate; no upgrade proposed. |
| [homebridge-plugins/homebridge-govee](https://github.com/homebridge-plugins/homebridge-govee) | [`c7b3037bddaf`](https://github.com/homebridge-plugins/homebridge-govee/commit/c7b3037bddaf465e440276a8fe74c8ec4e42e13a) | [v11.39.1](https://github.com/homebridge-plugins/homebridge-govee/releases/tag/v11.39.1) | Released brightness/scene fixes; first-empty-LAN-scan recovery is later main. Test the scenario; bridge updates require installed-version inspection. |
| [infamous-pattern/decksmith](https://github.com/infamous-pattern/decksmith) | [`76a75c8e0507`](https://github.com/infamous-pattern/decksmith/commit/76a75c8e0507786f8a22cd94361cf2e929900493) | [v1.5.1](https://github.com/infamous-pattern/decksmith/releases/tag/v1.5.1) | Public source matches Saturday testing notes; public release remains v1.5.1. Physical OS qualification remains open. |
| [juicetheforce/deckhand](https://github.com/juicetheforce/deckhand) | [`e0b8fba16e9c`](https://github.com/juicetheforce/deckhand/commit/e0b8fba16e9c62e4798b5eb6018ec50f492d511e) | [v0.8.1](https://github.com/juicetheforce/deckhand/releases/tag/v0.8.1) | 53 commits beyond prior pin; failure reporting, update notice, stale-response handling and local virtual Decks are useful references. Plus dials remain excluded. |
| [morluto/rea](https://github.com/morluto/rea) | [`b999e7247675`](https://github.com/morluto/rea/commit/b999e724767587bc5a2f0589579e0f836774b0e3) | [rea-agents-6.7.0](https://github.com/morluto/rea/releases/tag/rea-agents-6.7.0) | Latest API release is specifically rea-agents 6.7.0, not proof of a core-package upgrade. Duplicate-config safety/cancellation changes; retain targeted-use limits. |
| [nekename/OpenDeck](https://github.com/nekename/OpenDeck) | [`772189ca74c8`](https://github.com/nekename/OpenDeck/commit/772189ca74c89a10bad219b67de620ecb07bf1f6) | [v2.14.1](https://github.com/nekename/OpenDeck/releases/tag/v2.14.1) | Released resume/exit/image fixes; custom text/image precedence and another MK.2 module are later main. Extend regression/model qualification coverage selectively. |
| [neoyagami/yasdec](https://github.com/neoyagami/yasdec) | [`7ee6048f46e2`](https://github.com/neoyagami/yasdec/commit/7ee6048f46e2391563e4fb232768abe85df54359) | [v1.0.5](https://github.com/neoyagami/yasdec/releases/tag/v1.0.5) | Earlier community feedback reference, newly assessed here; stereo/spectrum ideas only. GPL source, dial-based models excluded. |
| [pbakaus/impeccable](https://github.com/pbakaus/impeccable) | [`0665bd812ae0`](https://github.com/pbakaus/impeccable/commit/0665bd812ae0908fcab17b96fd4f840defb882d9) | [cli-v4.6.0](https://github.com/pbakaus/impeccable/releases/tag/cli-v4.6.0) | 62 commits beyond prior pin; native audit skips web detector. Apply contrast, clipping, theme and recovery questions to actual GTK. |
| [pointfreeco/swift-composable-architecture](https://github.com/pointfreeco/swift-composable-architecture) | [`bc2db5ba8ad3`](https://github.com/pointfreeco/swift-composable-architecture/commit/bc2db5ba8ad3a47deba5db32fa340637ba6c9a76) | [1.26.2](https://github.com/pointfreeco/swift-composable-architecture/releases/tag/1.26.2) | Approved reference and bounded post-physical-acceptance editor-state trial. No Swift dependency or rewrite. |
| [pulseaudio/pulseaudio](https://github.com/pulseaudio/pulseaudio) | [`77d25a1e6130`](https://github.com/pulseaudio/pulseaudio/commit/77d25a1e613095bdf87a1d13b65e7c330565077a) | No GitHub release observed | Recent mirror commits concern FreeBSD/Solaris/translation/client utilities; no new Linux metering requirement found. Keep official API guidance. |
| [punkpeye/awesome-mcp-servers](https://github.com/punkpeye/awesome-mcp-servers) | [`f00a852f6e44`](https://github.com/punkpeye/awesome-mcp-servers/commit/f00a852f6e44db766312c7ad0de5e98803aefc9d) | No GitHub release observed | Catalogue additions do not establish a concrete Decksmith need, trust or permissions. No new server installation proposed. |
| [repowise-dev/repowise](https://github.com/repowise-dev/repowise) | [`272848e1d7dc`](https://github.com/repowise-dev/repowise/commit/272848e1d7dc2dc3ca87e09e365fa9d6def68d2a) | [v0.57.0](https://github.com/repowise-dev/repowise/releases/tag/v0.57.0) | Stable Python/Rust analysis improvements strengthen the case for the existing bounded source-analysis trial; no installation/qualification performed. |
| [tabler/tabler-icons](https://github.com/tabler/tabler-icons) | [`a4ce1404bc6d`](https://github.com/tabler/tabler-icons/commit/a4ce1404bc6d24d3c365afe7b258d6bf6f48d62d) | [v3.49.0](https://github.com/tabler/tabler-icons/releases/tag/v3.49.0) | v3.49.0 is newer than bundled v3.46.0 subset. Select needed monitor/media icons later, with notices and preserved saved artwork. |
