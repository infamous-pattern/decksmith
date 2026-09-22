use decksmith_core::{
    InputEvent, RawEvent,
    trigger::{Bindings, PressResolver, TimingPolicy, TriggerKind},
};
use decksmith_device::{DeckDevice, VirtualDeck};
#[test]
fn keys_and_dial_pushes_share_semantics_without_device_crosstalk() {
    let mut a = VirtualDeck::default();
    let mut b = VirtualDeck::default();
    let mut key = PressResolver::new(TimingPolicy::default(), Bindings::default()).unwrap();
    let mut dial = key.clone();
    a.inject(InputEvent {
        timestamp_ms: 0,
        event: RawEvent::Key {
            index: 0,
            pressed: true,
        },
    })
    .unwrap();
    b.inject(InputEvent {
        timestamp_ms: 0,
        event: RawEvent::DialPush {
            index: 0,
            pressed: true,
        },
    })
    .unwrap();
    for (device, resolver) in [(&mut a, &mut key), (&mut b, &mut dial)] {
        let e = device.receive_event().unwrap();
        let pressed = match e.event {
            RawEvent::Key { pressed, .. } | RawEvent::DialPush { pressed, .. } => pressed,
            _ => panic!(),
        };
        resolver.edge(e.timestamp_ms, pressed).unwrap();
    }
    a.disconnect();
    key.cancel();
    assert!(key.advance(600).unwrap().is_empty());
    assert_eq!(dial.advance(600).unwrap()[0].kind, TriggerKind::LongPress);
    a.reconnect();
    assert!(a.is_connected());
    assert!(a.receive_event().is_none());
}
#[test]
fn disconnect_rejects_writes_and_clears_stale_input() {
    let mut d = VirtualDeck::default();
    let image = vec![21; 120 * 120 * 3];
    d.set_key_image(0, &image).unwrap();
    d.set_touch_image(&vec![42; 800 * 100 * 3]).unwrap();
    let input = InputEvent {
        timestamp_ms: 10,
        event: RawEvent::Key {
            index: 0,
            pressed: true,
        },
    };
    d.inject(input.clone()).unwrap();
    d.disconnect();
    assert!(d.receive_event().is_none());
    assert!(d.inject(input).is_err());
    assert!(d.set_brightness(70).is_err());
    assert!(d.set_key_image(0, &image).is_err());
    assert_eq!(d.key_image(0).unwrap(), image);
    assert_eq!(d.touch_image()[0], 42);
    assert!(d.key_image(8).is_none());
    d.reconnect();
    d.set_brightness(70).unwrap();
    assert!(
        d.inject(InputEvent {
            timestamp_ms: 9,
            event: RawEvent::Key {
                index: 0,
                pressed: false
            }
        })
        .is_err()
    );
}
