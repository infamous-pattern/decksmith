//! Pace the live VirtualDeck service without changing deterministic device tests.
use decksmith_core::{Geometry, InputEvent};
use decksmith_device::{DeckDevice, DeviceError, VirtualDeck};
use std::time::Duration;

pub struct ServiceDeck {
    deck: VirtualDeck,
    wait: fn(Duration),
}
impl Default for ServiceDeck {
    fn default() -> Self {
        Self {
            deck: VirtualDeck::default(),
            wait: std::thread::sleep,
        }
    }
}
impl DeckDevice for ServiceDeck {
    fn geometry(&self) -> Geometry {
        self.deck.geometry()
    }
    fn set_key_image(&mut self, index: u8, rgb: &[u8]) -> Result<(), DeviceError> {
        self.deck.set_key_image(index, rgb)
    }
    fn set_touch_image(&mut self, rgb: &[u8]) -> Result<(), DeviceError> {
        self.deck.set_touch_image(rgb)
    }
    fn set_brightness(&mut self, percent: u8) -> Result<(), DeviceError> {
        self.deck.set_brightness(percent)
    }
    fn poll_event(&mut self) -> Result<Option<InputEvent>, DeviceError> {
        let event = self.deck.poll_event()?;
        if event.is_none() {
            (self.wait)(Duration::from_millis(20));
        }
        Ok(event)
    }
    fn poll_event_nonblocking(&mut self) -> Result<Option<InputEvent>, DeviceError> {
        self.deck.poll_event_nonblocking()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use decksmith_core::RawEvent;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[test]
    fn service_waits_only_for_empty_idle_input_never_pending_output_or_ready_input() {
        static WAITS: AtomicUsize = AtomicUsize::new(0);
        fn wait(duration: Duration) {
            assert_eq!(duration, Duration::from_millis(20));
            WAITS.fetch_add(1, Ordering::Relaxed);
        }
        let mut service = ServiceDeck {
            wait,
            ..Default::default()
        };
        assert_eq!(service.poll_event().unwrap(), None);
        assert_eq!(WAITS.load(Ordering::Relaxed), 1);
        assert_eq!(service.poll_event_nonblocking().unwrap(), None);
        let event = InputEvent {
            timestamp_ms: 1,
            event: RawEvent::Key {
                index: 0,
                pressed: true,
            },
        };
        service.deck.inject(event.clone()).unwrap();
        assert_eq!(service.poll_event().unwrap(), Some(event));
        assert_eq!(service.event_received_at(), None);
        service.deck.disconnect();
        assert_eq!(service.poll_event(), Err(DeviceError::Disconnected));
        assert_eq!(WAITS.load(Ordering::Relaxed), 1);
    }
}
