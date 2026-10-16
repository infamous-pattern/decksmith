# Physical Stream Deck support plan

Catalog audit: September 13, 2026. Official Elgato product pages and device
references were checked on this date. “Current” means listed in the official
catalog, not guaranteed stock in every region. This is a support target and
validation plan, not a statement that these devices already work with Decksmith
or that Elgato certifies Linux. Implementation checkpoint: October 16, 2026.
Stream Deck + is the Fedora 44 V1 hardware reference. The standard 15-key MK.2
now has adapter/fixture, permanent-service and paired physical evidence for
unreleased V1.5. Final signed-candidate admission and release certification remain open.

## Current physical device matrix

Each row needs its own identity/protocol/firmware record and physical validation.
Cosmetic colors, faceplates and branded editions share a row only after confirming
that their USB identity and control capabilities match. Do not assume shared SDK
device-type numbers imply interchangeable HID protocols.

| Device / variant | Controls to represent and validate | Planned coverage |
|---|---|---|
| [Stream Deck Mini](https://www.elgato.com/us/en/p/stream-deck-mini) | 6 LCD keys; no dials or touch strip | Key-only device phase; check legacy/revised USB identities |
| [Stream Deck, Classic Keys / MK.2 family](https://www.elgato.com/us/en/p/stream-deck) | 15 LCD keys | V1.5 adapter/fixtures and initial paired physical checks passed for standard MK.2; production/lifecycle/release gates open; original-generation compatibility separate |
| [Stream Deck Scissor Keys](https://www.elgato.com/us/en/p/stream-deck-scissor-keys) | 15 LCD keys, different key mechanism | Separate variant input/identity check; do not infer equivalence from key count |
| [Stream Deck XL](https://www.elgato.com/us/en/p/stream-deck-xl) | 32 LCD keys | Key-only phase; larger canvas, addressing and rendering load |
| [Stream Deck Neo](https://www.elgato.com/us/en/p/stream-deck-neo) | 8 LCD keys, Infobar display, 2 capacitive Touch Points | Distinct display and paging controls; not a four-dial Plus strip |
| [Stream Deck +](https://www.elgato.com/us/en/p/stream-deck-plus) | 8 LCD keys, 4 push/rotate dials, touch strip | Fedora 44 V1 reference; preserve regression coverage and complete V1.5 paired release gates |
| [Stream Deck + XL](https://www.elgato.com/us/en/p/stream-deck-plus-xl) | 36 LCD keys, 6 push/rotate dials, touch strip | Generalize beyond eight keys/four dials; verify model-specific image/gesture geometry |
| [Stream Deck Pedal](https://www.elgato.com/us/en/p/stream-deck-pedal) | 3 pedals; no LCD key images | Input-only editing, holds/releases and accessibility; no fabricated screen controls |
| [Stream Deck Studio](https://www.elgato.com/us/en/p/stream-deck-studio) | 32 LCD keys, 2 push/rotate encoders with LED indicators/rings; USB-C and Ethernet interfaces | Included physical target; separate protocol/transport and LED feasibility gate, then physical certification |
| [Stream Deck Module — 6 keys](https://www.elgato.com/us/en/p/stream-deck-module-6-keys) | Integration-ready 6-key hardware | Module identity/enclosure/connection verification; no assumed equivalence to retail Mini |
| Stream Deck Module — 15 keys | Integration-ready 15-key hardware | Separate variant record and fixture; same official Module page lists all three sizes |
| Stream Deck Module — 32 keys | Integration-ready 32-key hardware | Separate variant record and fixture; same official Module page lists all three sizes |

Elgato's [SDK device guide](https://docs.elgato.com/streamdeck/sdk/guides/devices/)
corroborates the named retail control types. Studio is marketed with Bitfocus
software; its product specifications, rather than generic comparison-table text,
govern its USB/Ethernet and LED investigation. Original first-generation Stream
Deck remains a legacy compatibility candidate, not proof of current retail stock.

## Phased delivery and acceptance

**V1.5 scope update — October 9:** simultaneous multi-device support is a release
requirement, initially qualified with Stream Deck + and the user's incoming
15-key MK.2. Complete model-specific MK.2 integration and physical acceptance,
then verify independent layouts/pages, simultaneous controls and recovery with
both connected. Identical-model simulation is required; physical identical-model
certification and the remaining model rows retain their own evidence gates. See
the [multi-device roadmap](ROADMAP.md#simultaneous-multiple-device-support--clarified-september-13).
This is a planned milestone, not a current support claim.

1. **Foundations:** capability-driven geometry, image formats/resolutions, control
   addressing and per-device session/configuration scopes. Remove prototype
   eight-key/four-dial/800×100 assumptions from generic code without regressing +.
   Use official [HID documentation](https://docs.elgato.com/streamdeck/hid/intro/)
   and model-specific references, including [+ XL](https://docs.elgato.com/streamdeck/hid/stream-deck-plus-xl/),
   where available; audit the selected adapter's actual coverage separately.
2. **Key-only family and Pedal:** validate Mini, current Classic/MK.2, Scissor,
   XL and input-only Pedal. Model-specific VirtualDeck fixtures precede hardware.
3. **Distinct display/control surfaces:** Neo Infobar/Touch Points, + regression
   certification, and + XL six-dial/large-key geometry. Display actual capabilities
   in Studio and reject/remap incompatible layout imports explicitly; never silently
   truncate keys or bind nonexistent dials.
4. **Integration/pro hardware:** Module variants and Studio. Verify actual USB
   IDs, firmware, event semantics and available protocols. Record unavailable
   capabilities/transport limitations explicitly; do not label an untested model
   “supported.” Studio's native Ethernet support needs a separate transport gate.
5. **Per-row release gate:** identify model/serial/firmware, exercise all input
   edges/holds/gestures, image orientation/resolution and preview parity, brightness
   where supported, disconnect/reconnect, suspend/resume, saved assignment recovery,
   and simultaneous operation. Include both two identical units and mixed models
   with different layouts, reversed reconnect order and one-device failure isolation.
   Record hardware/firmware, host environment, evidence date and known limitations.

Track status separately as planned, adapter-implemented, simulated, hardware-tested,
and release-certified. Refresh the official catalog before releases and add newly
listed variants to this matrix. Complete-range support is the product goal; the
existing Plus-first V1 certification boundary is retained, not silently converted
into an immediate all-model release promise. The selected library is not the ceiling
on the planned hardware range.

## October 13 paired hardware evidence

The [guarded hardware trial](checkpoints/2026-10-13-physical-mk2-pair.md) confirmed
all fifteen MK.2 keys, image orientation, brightness and independent mixed-device
pages/brightness. Removing either device left the other connected and responsive;
both recovered their own trial identity. Plus strip restoration, isolated swipes,
four dial presses/turns and touch input were checked. Test profiles are private
and temporary; installed V1 remains Plus-only. Exact touch classification uses
aggregate telemetry plus user-directed actions, not per-gesture timing logs.

Physical reversed connection order, paired lock/suspend, shared action dispatch,
production workspace/enrollment and sustained resource qualification remain gates.
Do not extend this evidence to Scissor/Module variants or certify the full range.

## October 15 permanent-service qualification

The installed development runtime controls Plus and MK.2 together with saved
independent profiles. Physical checks accepted pages/brightness, shared native
audio/media/microphone controls, lock/locked reconnect, suspend, bidirectional USB
isolation/recovery, quit blanking/relaunch, login startup, paired editor save and
previous-layout recovery, keyboard focus and Orca across successive builds.
The latest optimized development package also passed ordinary/quicker controls,
bidirectional USB recovery and finite quiet/loaded resource observations. See the
[V1.5 candidate review](v1.5-candidate.md) for carried-forward coverage and limits.
No other model or physical identical-model pair inherits this evidence. Signed
candidate qualification remains separate from development acceptance.

## Scope decisions kept separate

- [Network Dock](https://www.elgato.com/us/en/p/network-dock-stream-deck) is an
  Ethernet transport accessory, not another key geometry. Its transport support
  needs an explicit scope decision and protocol validation; USB support alone is
  not Network Dock support. Audio docks likewise do not automatically imply audio
  hardware/firmware integration simply because the attached Plus is supported.
- Elgato's [device guide](https://docs.elgato.com/streamdeck/sdk/guides/devices/)
  lists Corsair Galleon 100 SD (12 LCD keys, two dials and a display), Voyager,
  Corsair G-Keys and SCUF integrations. Whether to include these embedded/partner
  products is an open scope question, not silently included in standalone Elgato
  hardware certification.
- Stream Deck Mobile and Virtual Stream Deck are software surfaces. They are not
  included by this physical-hardware request; adding them needs a separate decision.
  Decksmith's own VirtualDeck testing abstraction is already in scope.
