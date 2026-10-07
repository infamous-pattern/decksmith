# Signed V1: experimental Homebridge fault qualification — October 7, 2026

The retained Fedora Workstation 44 VM ran isolated tests against companion
`01e396045147cdd7` and the frozen signed core candidate `1.0.0-c5e4fea53afe`,
public source `68c42421ceaa65bb033993b56f0babdde2a14a6e`. No production code,
polling interval, desktop assignment or accepted archive byte changed.

## Artifact admission and isolation

All 75 files in the retained companion package matched its manifest; the manifest
identity was recomputed. All 13 bundled Homebridge runtime members in the signed
core archive match that companion byte-for-byte. The reviewed plugin binary's
SHA-256 is `418e8a4d70650b94588b368968cdef3b6e1ea6f7d13b24ee75bb68e9b46add4e`.
The core archive retained its accepted SHA-256:
`4334a2df091ccdb0fac2431e416fa51cf4dabe0da1dfe4ccc372f60029a3dda2`.
This binds tested companion source to the candidate; it does not claim the
separately packaged plugin binary/vendor files have the core archive's signature.

The tests ran in a unique guest `/tmp` tree with temporary XDG paths, loopback
fake Homebridge servers and invented credentials. Python non-loopback connection
and lookup attempts were blocked by the harness; real credential-store methods
were forbidden. The pinned plugin received only fixture loopback destinations.
No normal guest service, saved profile, real Homebridge server or physical device
was operated. The normal guest service was inactive before and after.

## Results: 70 checks passed

- **66 regression/authentication/recovery tests:** accessory capability admission,
  read-only sensors, off-light protection, busy/rejected inputs, lost replies,
  bounded and redacted diagnostics, connection removal/preservation, offline and
  child-crash recovery, password rejection, revoked sessions, persistent denial,
  OTP expiry/replacement, single-session sharing, keyring failure and setup expiry.
  Zero failures, errors or skips; 66.47 seconds.
- **One real loopback HTTP boundary test:** panel/state/catalog require local
  authorization; unauthorized commands, foreign Origins and invalid Host headers
  are rejected; the privileged landing response applies no-referrer protection.
  Panel state/commands are mocked for this boundary test; it is not a real server
  connection or credential-store test.
- **Three core/companion isolation tests:** during simulated whole-server outage,
  plugin crash/reconnect and applied-write/lost-reply scenarios, the exact signed
  daemon ran VirtualDeck on a private bus. Ten status checks, ten page changes and
  ten shared-key-preview checks passed. It remained display-ready and exited
  cleanly only after explicit termination. This finite fixture test complements
  physical controls; it is not physical USB/audio/visible-pixel evidence.

Uncertain writes were not replayed. A terminal HTTP authentication rejection is
separate: its test permits one bounded reauthentication retry after a definite
401 response and verifies exactly one accepted write. An ambiguous applied write
with no reply retains the manual-recovery gate.

Initial supplemental attempts exposed test-harness setup errors: an HTTP script
name shadowed Python's library, a retained test symlink was recreated, extracted
binaries lacked installer-applied executable modes, and fixture isolation removed
the newly created private bus address. These were corrected in ignored harness
files. No application defect/fix is inferred from those startup attempts. The
final supplementary cases passed. Private-bus ScreenSaver activation messages do
not qualify GNOME lock behavior; the separate physical lock test covers that.

## Preservation, limits and follow-up

Normal guest configuration/artwork, integration files, release links and startup
preferences compared identically before/after. Temporary test files were removed,
and the guest shut down normally. All four retained VMs are off. The desktop
still runs the same candidate/companion and has zero automatic daemon restarts.

This completes the scoped simulated Homebridge failure/authentication/recovery,
local HTTP boundary and core-isolation checks. The separate companion remains
experimental; general plugin compatibility/sandboxing and marketplace support are
not V1 claims. The user then completed the agreed live check: existing accessory toggle keys and
ordinary-speed brightness dial, followed by a normal page and audio control. They
reported: “Everything responds correctly and stays available.” This records
physical accessory/built-in coexistence acceptance on the signed core and same
companion, without an automated accessory command.

Precise visible response, resource reconciliation, any remaining advertised
control/draft/error cases and final verified delivery/release acceptance remain
separate. Private evidence is retained under
`local/v1-final-20261007/homebridge-vm/`.
