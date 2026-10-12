//! Contract fixtures only: no USB, desktop bus, audio or production service access.
use decksmith_core::{
    Geometry, InputEvent, RawEvent, TouchGesture,
    device_model::{ControlAddress, DeviceDescriptor, DeviceModel},
    trigger::{Bindings, PressResolver, TimingPolicy, TriggerKind},
};
use decksmith_device::{DeckDevice, DeviceError, VirtualDeck};

fn key(timestamp_ms: u64, index: u8, pressed: bool) -> InputEvent {
    InputEvent {
        timestamp_ms,
        event: RawEvent::Key { index, pressed },
    }
}

#[test]
fn default_plus_retains_its_geometry_and_input_contract() {
    let mut deck = VirtualDeck::default();
    assert_eq!(deck.geometry(), Geometry::plus());
    assert_eq!(deck.descriptor().model(), DeviceModel::Plus);
    for event in [
        RawEvent::Key {
            index: 7,
            pressed: true,
        },
        RawEvent::DialPush {
            index: 3,
            pressed: true,
        },
        RawEvent::DialRotate {
            index: 3,
            ticks: -127,
        },
        RawEvent::Touch {
            x: 799,
            y: 99,
            gesture: TouchGesture::Tap,
        },
        RawEvent::Touch {
            x: 0,
            y: 0,
            gesture: TouchGesture::LongPress,
        },
        RawEvent::Touch {
            x: 0,
            y: 0,
            gesture: TouchGesture::FlickLeft,
        },
        RawEvent::Touch {
            x: 799,
            y: 99,
            gesture: TouchGesture::FlickRight,
        },
    ] {
        let input = InputEvent {
            timestamp_ms: 10,
            event,
        };
        deck.inject(input.clone()).unwrap();
        assert_eq!(deck.poll_event().unwrap(), Some(input));
    }
    assert_eq!(deck.event_received_at(), None);
}

#[test]
fn mk2_all_fifteen_keys_accept_rgb_frames_and_ordered_edges() {
    let mut deck = VirtualDeck::new(DeviceModel::Mk2);
    let geometry = deck.geometry();
    assert_eq!((geometry.columns, geometry.rows), (5, 3));
    assert_eq!(geometry.key_pixels, (72, 72));
    assert_eq!(geometry.dials, 0);
    assert_eq!(deck.descriptor().capabilities().touch, None);
    let mut expected = Vec::new();
    for index in 0..15 {
        // Unequal corners retain orientation in the shared RGB contract. Physical
        // encoding/orientation is a separate adapter/hardware acceptance gate.
        let mut frame = vec![index; 72 * 72 * 3];
        frame[..3].copy_from_slice(&[255, 0, index]);
        let end = frame.len();
        frame[end - 3..].copy_from_slice(&[0, 255, index]);
        deck.set_key_image(index, &frame).unwrap();
        assert_eq!(deck.key_image(index).unwrap(), frame);
        for pressed in [true, false] {
            let event = key(u64::from(index) * 2 + u64::from(!pressed), index, pressed);
            deck.inject(event.clone()).unwrap();
            expected.push(event);
        }
    }
    for event in expected {
        assert_eq!(deck.poll_event().unwrap(), Some(event));
    }
    assert_eq!(deck.poll_event().unwrap(), None);
    assert!(deck.key_image(15).is_none());
    let before = deck.key_image(14).unwrap().to_vec();
    assert_eq!(
        deck.set_key_image(14, &vec![0; 120 * 120 * 3]),
        Err(DeviceError::InvalidKeyImage)
    );
    assert_eq!(
        deck.set_key_image(15, &before),
        Err(DeviceError::InvalidKeyImage)
    );
    assert_eq!(deck.key_image(14).unwrap(), before);
    deck.set_brightness(73).unwrap();
    assert_eq!(
        deck.set_brightness(101),
        Err(DeviceError::InvalidBrightness)
    );
    deck.blank().unwrap();
    assert_eq!(deck.brightness, 73);
    for index in 0..15 {
        assert!(deck.key_image(index).unwrap().iter().all(|byte| *byte == 0));
    }
    assert!(deck.touch_image().is_empty());
}

#[test]
fn mk2_rejects_nonexistent_surfaces_without_advancing_time_or_mutating_frames() {
    let mut deck = VirtualDeck::new(DeviceModel::Mk2);
    deck.set_key_image(0, &vec![31; 72 * 72 * 3]).unwrap();
    for (event, error) in [
        (
            RawEvent::Key {
                index: 15,
                pressed: true,
            },
            DeviceError::InvalidKey,
        ),
        (
            RawEvent::DialPush {
                index: 0,
                pressed: true,
            },
            DeviceError::InvalidDial,
        ),
        (
            RawEvent::DialRotate { index: 0, ticks: 1 },
            DeviceError::InvalidDial,
        ),
        (
            RawEvent::Touch {
                x: 0,
                y: 0,
                gesture: TouchGesture::Tap,
            },
            DeviceError::InvalidTouch,
        ),
    ] {
        assert_eq!(
            deck.inject(InputEvent {
                timestamp_ms: u64::MAX,
                event
            }),
            Err(error)
        );
    }
    for image in [&[][..], &vec![1; 800 * 100 * 3][..]] {
        assert_eq!(
            deck.set_touch_image(image),
            Err(DeviceError::InvalidTouchImage)
        );
    }
    assert!(deck.touch_image().is_empty());
    assert_eq!(deck.key_image(0).unwrap()[0], 31);
    let accepted = key(1, 0, true);
    deck.inject(accepted.clone()).unwrap();
    assert_eq!(deck.poll_event().unwrap(), Some(accepted));
}

#[test]
fn key_only_blank_attempts_every_key_after_failure_and_never_calls_touch() {
    struct Probe {
        written: Vec<u8>,
    }
    impl DeckDevice for Probe {
        fn descriptor(&self) -> DeviceDescriptor {
            DeviceDescriptor::for_model(DeviceModel::Mk2)
        }
        fn set_key_image(&mut self, index: u8, rgb: &[u8]) -> Result<(), DeviceError> {
            assert_eq!(rgb.len(), 72 * 72 * 3);
            assert!(rgb.iter().all(|byte| *byte == 0));
            self.written.push(index);
            if index == 7 {
                Err(DeviceError::Transport)
            } else {
                Ok(())
            }
        }
        fn set_touch_image(&mut self, _: &[u8]) -> Result<(), DeviceError> {
            panic!("key-only devices have no touch surface");
        }
        fn set_brightness(&mut self, _: u8) -> Result<(), DeviceError> {
            panic!("blanking must preserve brightness");
        }
        fn poll_event(&mut self) -> Result<Option<InputEvent>, DeviceError> {
            Ok(None)
        }
    }
    let mut device = Probe {
        written: Vec::new(),
    };
    assert_eq!(device.blank(), Err(DeviceError::Transport));
    assert_eq!(device.written, (0..15).collect::<Vec<_>>());
}

#[test]
fn queue_overflow_is_atomic_for_each_model_and_retains_fifo() {
    for model in [DeviceModel::Plus, DeviceModel::Mk2] {
        let mut deck = VirtualDeck::new(model);
        let accepted = key(1, 0, true);
        for _ in 0..4096 {
            deck.inject(accepted.clone()).unwrap();
        }
        assert_eq!(deck.inject(key(100, 0, false)), Err(DeviceError::QueueFull));
        assert_eq!(deck.poll_event().unwrap(), Some(accepted.clone()));
        // A rejected future event must not advance the monotonic clock.
        let final_event = key(2, 0, false);
        deck.inject(final_event.clone()).unwrap();
        for _ in 0..4095 {
            assert_eq!(deck.poll_event().unwrap(), Some(accepted.clone()));
        }
        assert_eq!(deck.poll_event().unwrap(), Some(final_event));
        assert_eq!(deck.poll_event().unwrap(), None);
    }
}

#[test]
fn mixed_and_identical_devices_keep_frames_input_and_hold_state_independent() {
    for (first, second) in [
        (DeviceModel::Plus, DeviceModel::Mk2),
        (DeviceModel::Mk2, DeviceModel::Mk2),
    ] {
        let mut a = VirtualDeck::new(first);
        let mut b = VirtualDeck::new(second);
        a.set_key_image(0, &vec![11; a.descriptor().key_rgb_bytes()])
            .unwrap();
        b.set_key_image(0, &vec![22; b.descriptor().key_rgb_bytes()])
            .unwrap();
        a.set_brightness(30).unwrap();
        b.set_brightness(80).unwrap();
        let mut hold_a = PressResolver::new(TimingPolicy::default(), Bindings::default()).unwrap();
        let mut hold_b = hold_a.clone();
        for (deck, resolver) in [(&mut a, &mut hold_a), (&mut b, &mut hold_b)] {
            deck.inject(key(0, 0, true)).unwrap();
            let received = deck.poll_event().unwrap().unwrap();
            resolver.edge(received.timestamp_ms, true).unwrap();
            // Queue a different control; key 0 remains physically held while
            // the caller advances its resolver's monotonic deadline.
            deck.inject(key(10, 1, true)).unwrap();
        }
        a.disconnect();
        hold_a.cancel(); // The future manager must cancel only the disconnected unit.
        assert_eq!(a.poll_event(), Err(DeviceError::Disconnected));
        assert_eq!(a.blank(), Err(DeviceError::Disconnected));
        assert!(hold_a.advance(600).unwrap().is_empty());
        assert_eq!(hold_b.advance(600).unwrap()[0].kind, TriggerKind::LongPress);
        assert_eq!(b.poll_event().unwrap(), Some(key(10, 1, true)));
        assert_eq!(b.brightness, 80);
        assert_eq!(b.key_image(0).unwrap()[0], 22);
        a.reconnect();
        assert_eq!(a.poll_event().unwrap(), None); // No replay after reconnect.
        assert_eq!(a.key_image(0).unwrap()[0], 11);
        assert_eq!(a.brightness, 30);
        b.blank().unwrap();
        assert_eq!(a.key_image(0).unwrap()[0], 11);
        a.inject(key(9, 0, false)).unwrap_err();
        a.inject(key(11, 0, false)).unwrap();
        assert_eq!(a.poll_event().unwrap(), Some(key(11, 0, false)));
    }
}

#[test]
fn typed_control_addresses_keep_key_and_dial_push_distinct() {
    let descriptor = DeviceDescriptor::for_model(DeviceModel::Plus);
    let key = RawEvent::Key {
        index: 0,
        pressed: true,
    }
    .control();
    let dial = RawEvent::DialPush {
        index: 0,
        pressed: true,
    }
    .control();
    assert_ne!(key, dial);
    assert!(descriptor.supports(key));
    assert!(descriptor.supports(dial));
    assert!(!descriptor.supports(ControlAddress::Key(u8::MAX)));
    assert!(!descriptor.supports(ControlAddress::DialRotation(u8::MAX)));
}
