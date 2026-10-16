# V1.5 development package review — October 15, 2026

The locally installed package `1.0.0-dev-emoji-4551ce366af5` and companion `37adab0cf9b2886f` underwent additional read-only desktop and isolated AI-server checks after the accepted paired physical resource test. This is development qualification, not a signed V1.5 release or expanded platform support declaration. No runtime source, Decksmith services, desktop audio settings, scheduled jobs or OS packages were changed during this review. The later explicitly authorized speech-service pause/restoration for resource measurement is recorded below.

## Package integrity and dependency/hardening review

- Installed files matched the exact qualified packages: 485 app manifest files and 75 companion files, with no extra files. The companion native binary matched its rebuild receipt and lacked the tested private builder-path strings.
- On Fedora Server 44, all 648 immutable source-snapshot files verified; 482 packaged source files matched that snapshot, and the package manifest verified all 485 runtime files. All twelve Google Fonts provenance records verified, including the new pinned fallback. Liberation assets are covered by the complete package manifest rather than those twelve source records.
- Both packaged executables passed PIE, RELRO, bind-now and non-executable-stack checks; the tested private builder-path strings were absent. Relocation into a path containing spaces passed self-check, VirtualDeck once and CLI help.
- A private paired-device case rendered both models and restored both original profiles from a staged installer backup after 24 later saves/history archiving. This finite test passed without changing runtime or installer source; it does not prove every schema downgrade or every historical backup compatible.
- A current exact-version OSV query covering 277 distinct registry/PyPI versions from the app and rebuilt companion lockfiles plus vendored websockets returned no matches. This is not a complete security audit, does not cover unknown vulnerabilities or Fedora system libraries, and does not replace final signed-artifact admission.

The app archive SHA-256 is `59fb5f56edac2fef461dda9e88b507e7f077864d0a27c05197025806973ffd63`; the immutable source snapshot is `9db91503a6cc3abd725ef5658311b61899e332727b6620ffeca3edca173c99df`. Installed development files remain unchanged.

## Fedora VM package/native qualification

The AI server initially refused another full recovery copy because it could not retain its configured working-space reserve. No original guest started in that failed attempt. Instead, the tests used owned transient copy-on-write clones with the original Fedora 44 and Fedora 45 beta VMs powered off and locked against scheduled jobs. Original disk bytes and domain definitions were verified unchanged. These are clones of existing test baselines, not fresh OS dependency-provisioning evidence.

Both guests passed the latest-package dependency/install checks, saved-file/startup preservation, normal service start/stop/restart in no-device waiting mode, rollback/reinstall, a normal native waiting GUI, and private native two-model editor/save/preview/brightness/previous-layout/restart checks. Fedora 44 Plus and Fedora 45 MK.2 screenshots were visually inspected: the checkmark/music-note caption fallback rendered correctly and the simplified editor retained its layout and scrolling controls. There was no physical USB/native sound test in either VM, and no Debian/Ubuntu/COSMIC preview was rebuilt here.

Fedora 44's baseline restoration and shutdown passed directly. Fedora 45's rapid test sequence hit systemd's start limit while restoring the older baseline build; all new-package checks had already passed. A separate fresh boot of that retained clone verified the original enabled/running build, all 5,158 regular files and its one release symlink exactly, without reset-failed or reinstallation. That follow-up's pinned SSH poweroff was rejected; a subsequent targeted ACPI shutdown of the owned transient clone completed normally. No forced shutdown occurred. The original failure and both follow-up records remain retained rather than rewriting them as a flawless single run.

At final verification, all ten original server guests were off and no transient test domains remained running. Temporary overlay files remain private test artifacts; they are not independent backups and must not be reused after their original backing image changes without requalification.

## Performance review and open desktop check

The accepted [paired resource checkpoint](2026-10-15-paired-resources.md) remains authoritative: the finite loaded test passed user acceptance, while the earlier paused-desktop sample retained two speech playback streams and is not clean idle.

A separate 30.003-second read-only attribution of the current desktop observed approximately 3.10% of one core in the hardware-discovery thread and 2.47% in the main coordinator, compared with 6.87% summed daemon CPU during that short interval. Source review locates discovery on a worker with one-second scan cadence and a five-millisecond coordinator loop. This identifies costs to investigate; it neither isolates their cause in a matched experiment nor explains away a regression. No polling/threading/code change was made.

The user subsequently authorized stopping the leftover speech process and measuring idle. Firefox reactivated the first graceful stop through its speech socket, so that first preflight was rejected with no measurement points. A temporary speech-service/socket pause then allowed 300.001 seconds of clean paired idle (61 points), with no playback streams. Both speech units were restored to their original running/enablement states; Screen Reader stayed disabled. Daemon CPU averaged 5.757% of one core, combined services 8.005%, and daemon RSS stayed at 51.59 MiB. Both Decks and service identities stayed stable, with no restart or saved-layout change. This completes clean-idle data collection; the historical V1 baseline is not a matched workload, so resource regression review remains open. See the paired resource checkpoint for the full method and limits.

## Storage and remaining release steps

The server root filesystem is approximately 96% used, with about 19 GiB free at the final check and about 227.81 GiB in retained test backups. The normal full-recovery-copy test path could not meet its working-space reserve. No existing backups, originals or schedules were deleted/modified. Resolve capacity before the next full-copy VM backup batch; the overlay test is not a change to the scheduled backup policy.

Remaining: resource regression review using the completed clean-idle and loaded observations, version/source finalization and synchronization, a clean signed V1.5 candidate and fresh-download admission, and explicit release approval. The current local development package has not been presented as a public V1.5 artifact. The default/no-hardware build gap and general experimental OS/device limitations remain separately documented.

Private evidence is in `outputs/v15-emoji-20261015/release-review` in the enclosing workspace: package/server checks, advisory records, installed integrity, thread attribution, VM screenshots and original/follow-up cleanup results. Initial artifact-harness syntax and readiness-wait errors are retained; corrected fixture checks passed without app changes. These internal fixtures and guest records are not public release inputs.
