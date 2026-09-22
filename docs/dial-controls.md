# Four-dial controls

In Edit Layout, choose **Edit dials…**. Dial 1 is leftmost. Each dial has an independent label (1–24 ASCII letters, digits or spaces), rotation action (none, output volume, device brightness), step size (1–10 percent per tick), and press/release action (none, mute, next page, previous page). Settings apply across all pages.

**Done** keeps the dial changes in the editor draft and returns to Edit Layout; **Save and Apply** sends it to the device. Existing layouts retain their original dial behavior until explicit settings are saved. The touch strip then displays four labeled panels with current volume/mute, brightness, or page position. Swipes continue to navigate pages.

Rotation events are bounded to ±20 percentage points; brightness is clamped to 0–100%. Brightness changes are persisted off the device loop, with a final save on normal shutdown. Abrupt power loss can lose a pending preference write. Press actions require a matching press and release, and page changes cancel held actions.

Layout JSON stores an optional array of exactly four `dials`. Layout packages separate their labels into optional `dial-appearance.json`; existing packages without dial settings remain supported. The bundled smooth font renders strip labels and values. Each panel uses its full 200-pixel width with 8-pixel side margins; labels automatically shrink from 24 pixels to fit.

Validation: Rust dispatch tests cover all four indices, unmatched/repeated releases, extreme signed ticks, and invalid steps. The GTK regression changes Dial 4, saves into the draft, and verifies package round-trip preservation. Physical interaction remains a user acceptance check after configuration.

## Live level bars

Volume and brightness panels now show their label at the top, percentage in the
middle, and a glossy blue capsule bar below. The bar tracks the reported value
on each update, without independent decorative animation or a new timer. Muted
volume retains its percentage and uses a gray fill. Unavailable values show an
empty track with `--`; controls without a percentage retain their existing text.
Values above 100% still display numerically while the bar is capped at full.

The renderer uses antialiased rounded edges and a gradient highlight inspired
by the supplied reference. Checks cover empty, partial, full and muted fills,
and verify that the bar does not overwrite the label/value area. The production
renderer preview was inspected at the physical 800×100 resolution.
