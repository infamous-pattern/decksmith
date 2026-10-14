# Connected native output and media checkpoint — October 14, 2026

The first opt-in native slice now supplies the connected Plus/MK.2 runtime with
real, explicitly selected output volume/mute and pinned MPRIS media commands.
The user accepted both devices controlling the same output and Brave playback
in a bounded physical window. The normal installed V1 service, saved page,
configuration, output volume/mute and playback state were restored afterward.
This is a development checkpoint, not an installed or released multi-device build.

## Implementation boundary

One shared native action adapter serializes commands for a selected named output
or exact MPRIS unique owner. The media selection includes the exact instance name
and owner; capability and owner checks run before a command, and a device/context
fence is checked immediately before the external write. Unsupported commands and
changed sessions fail explicitly without choosing another player. The initial
audio compiler rejects default-output aliases and numeric indexes.

One read-only output observer and one meter serve both devices. Observations
expire after 500 ms; the observer waits 200 ms between reads. Changed values feed
the existing independent render owners. This avoids duplicating an output reader
or meter for each Deck; it does not establish a production resource budget.

MPRIS calls have explicit 200 ms bounds, including the call path using NoAutoStart.
A timed-out external write may already have reached its destination. Such an
uncertain effect blocks further writes to that target for the trial and prevents
a clean acceptance result. Shutdown revokes device scopes, drains input/display
owners and joins action workers before restoration. It cannot undo an external
request that has already been sent.

`--connected-native-lab FRESH_ABSOLUTE_DIRECTORY --settings PRIVATE_SETTINGS
--seconds 20..600` requires both `DECKSMITH_PHYSICAL_TEST=1` and
`DECKSMITH_NATIVE_TEST=1`. The private settings select the named output and exact
player session. The command opens one Plus and one MK.2, creates fresh temporary
profiles, and does not migrate the production layout. The timed external handoff
guard preserves exact per-channel output levels, mute and explicit Play/Pause
state, checks original target identity before restoration, and restores V1 even
when the trial fails. Identity changes or uncertain effects require review.

## Isolated qualification

Source snapshot
`265212071eec4d4b48e4d70d84b844bd253729e939c81193e8f83b34c71f63b4`
passed **287 Rust tests, zero failures, four intentional ignores**, strict
all-feature/all-target Clippy, formatting and build; **192 Studio** and **111
script** tests, with one conditional skip in each suite; native saved-profile,
device-workspace and legacy editor checks; and connected-execution fixtures.
The locally built physical candidate matched the qualified source bytes.

Four separate private-bus MPRIS cases used the actual native adapter and shared
service: success from two device scopes; unsupported capability with no command;
replacement owner with no command; and one uncertain write followed by a blocked
retry. These checks also exposed and corrected the flags-call timeout gap before
the final snapshot. The AI-server qualification opened no physical devices,
started no VM and did not access real audio or change scheduled checkouts.

## Physical acceptance and recovery

One approximately five-minute window used the real connected actors, model
renderers and sole I/O owners. The user confirmed that MK.2 volume/mute keys and
the Plus second dial changed the actual output and that the touch strip followed
both. Both devices paused/resumed Brave without selecting another player.

The final recording contains **7 successful MK.2 replies and 21 successful Plus
replies**, no action-feedback failures and no uncertain native effects. It includes
14 MK.2 key edges, two Plus key edges, 18 Plus dial-turn records and four Plus
dial-push edges. A focused third-dial check matched two new successful commands
and four physical press/release edges to the user's pause/resume confirmation.
These counts are observations, not latency or performance qualification.

Both devices completed joined blanking successfully. The guard verified exact
output channel levels/mute and original player playback state, unchanged saved
configuration/build, and the normal V1 connection, ready display and original
page. Private evidence retains the source/binary hashes, interaction confirmations,
telemetry, backups and restoration outcomes; private hardware/player identifiers
are excluded from this public record.

## Remaining activation gates

This slice covers one explicitly named output and one pinned player instance.
Canonical application/System sounds/default-player selection, input mute and
push-to-talk, other native/plugin actions, live roster extension and the
device-addressed editor/API remain. Next connect the established provider
selection/observation rules to saved device profiles, preserving their unavailable
states and deliberate labels. Then qualify native paired disconnect/reconnect,
lock/suspend/startup, migration/rollback and resource behavior before replacing
the daily V1 runtime. The previous simulated physical disconnect acceptance is
not a substitute for those native lifecycle checks.
