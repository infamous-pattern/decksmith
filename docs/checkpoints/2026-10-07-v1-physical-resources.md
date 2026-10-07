# Signed V1 physical interaction and resources — 2026-10-07

The Fedora Workstation 44 reference desktop completed a ten-minute physical
interaction sample of signed candidate `1.0.0-c5e4fea53afe`, public source
`68c42421ceaa65bb033993b56f0babdde2a14a6e`. The accepted archive SHA-256 remains
`4334a2df091ccdb0fac2431e416fa51cf4dabe0da1dfe4ccc372f60029a3dda2`.
This checkpoint records qualification evidence, not a V1 release decision.

The user confirmed physical dials were used at ordinary and quicker speeds and
pages were switched: “Tested both speeds and pages; everything worked.” No freeze,
missed input, wrong-target change or incorrect label/icon/meter was reported.
All 210 recorded actions succeeded. Core and companion process identities and
restart counts were unchanged, with zero automatic restarts.

## Workload and observation limits

The observer collected 121 five-second points spanning 599.989 seconds. Output,
microphone, Brave and Chrome were the four target assignments; output includes
the browser mix, so these are not four independent playback sources. Both browser
streams remained uncorked at the thirty-second inventory checks.

Chrome temporarily replaced Dial 4 through overrides on Home, System and Media.
The Homebridge page retained its fan override. One observation, approximately
five seconds into the run, captured that page. The expected four-target set was
therefore present in **120 of 121 observations**, not all observations. At the
next five-second point it had returned. All 120 four-target preview checks were
live, without unavailable markers, and every activity bar changed during the run.

The sampler issued no volume, mute, media or accessory commands. The user supplied
physical inputs. Observer reads contribute some work; periodic checks can miss
short faults. Preview frames exercise the shared renderer, not visible physical
pixels. This is sustained mixed-page acceptance with substantial four-target
coverage, not uninterrupted four-target certification.

## Resources

CPU percentages use one logical core as 100%, summed across threads. Core-unit
figures include helpers and exited-child work. The experimental companion is
separate. Cgroup memory includes helpers and cache and is not daemon RSS.

| Measurement | Result |
| --- | --- |
| Main daemon mean CPU | 3.52% |
| Core service/helpers mean CPU | 6.53% |
| Core five-second CPU interval p95 | 13.94% |
| Experimental companion mean CPU | 0.65% |
| Core plus companion mean CPU | 7.18% |
| Main daemon RSS | 27.402–27.773 MiB |
| Main daemon open handles | 16–18 |
| Core cgroup memory | Mean 76.58 MiB; range 69.44–89.05 MiB |
| Companion cgroup memory | Mean 68.53 MiB; range 63.57–70.40 MiB |

Daemon RSS increased about 380 KiB over this finite run. Core anonymous memory
changed from 54.38 to 56.02 MiB; companion anonymous memory changed from 56.45 to
61.79 MiB. These bounded observations do not exclude longer-term growth.

The [previous-build physical sample](2026-10-06-physical-soak.md) measured 2.54%
daemon and 4.66% core CPU. This run measured approximately 38% and 40% more,
respectively, while recording 204 dial adjustments versus 133 previously. The
different activity counts, days and service lifetimes prevent a matched A/B
comparison. They do not explain the separate increase in the
[quiet observation](2026-10-07-v1-quiet-resources.md). CPU regression investigation
remains open; this result is not a regression-free performance acceptance.

## Dispatch and restoration

The 210 successful actions comprised 204 audio dial adjustments, four application
launches and two media play/pause actions. Dial adjustments covered output (30),
microphone (26), Brave (103) and Chrome (45). Dial dispatch median was 0.0585 ms,
p95 **21.445 ms**, and maximum 26.437 ms. Application and media dispatch maxima
were 0.093 and 0.082 ms. Four physical page-render invocations had a maximum of
0.063 ms.

The measured boundary starts at the pinned hardware library's report return and
ends at backend entry or page-render invocation. It excludes USB/firmware delay,
backend completion and visible pixels. Dial p95 met the 25 ms dispatch target in
this sample; the maximum exceeded 25 ms. This does not certify unexercised routes
or close precise visible-response timing.

The original runtime layout and active page were restored without restarting
background controls. All five saved JSON files were verified byte-for-byte against
their pre-test backups. Dial 4 again uses System sounds. User-operated volume
changes were retained. Raw layouts, journal and samples remain private and are
excluded from public exports.

Remaining V1 gates include resource regression review, precise visible-response
timing, final-artifact lifecycle/accessibility, clean dependency provisioning,
optional-companion failure/authentication/recovery and release acceptance.

See the subsequent [CPU investigation](2026-10-07-v1-cpu-investigation.md) for
read-only attribution and the limited paired VirtualDeck comparison. It retains
the physical-comparison limits above and leaves the installed artifact unchanged.
