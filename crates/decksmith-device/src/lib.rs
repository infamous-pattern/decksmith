//! Project-owned device contract, VirtualDeck, and Linux discovery.
use decksmith_core::{Geometry, InputEvent, RawEvent};
use std::collections::VecDeque;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceError {
    Disconnected,
    TimeReversed,
    InvalidKey,
    InvalidDial,
    ZeroRotation,
    InvalidTouch,
    QueueFull,
    InvalidKeyImage,
    InvalidTouchImage,
    InvalidBrightness,
    Transport,
    InvalidReport,
    ImageEncoding,
    DeviceSelection,
    DeviceBusy,
}
impl std::fmt::Display for DeviceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::DeviceBusy => "device ownership unavailable or competing application running",
            Self::Disconnected => "device disconnected",
            Self::Transport => "hardware transport failed",
            Self::InvalidReport => "invalid device report",
            Self::ImageEncoding => "image encoding failed",
            Self::DeviceSelection => "expected exactly one connected Stream Deck Plus",
            Self::TimeReversed => "timestamps must be monotonic",
            Self::InvalidKey => "key index must be 0..7",
            Self::InvalidDial => "dial index must be 0..3",
            Self::ZeroRotation => "rotation requires nonzero ticks",
            Self::InvalidTouch => "touch coordinates out of bounds",
            Self::QueueFull => "virtual input queue is full",
            Self::InvalidKeyImage => "expected key 0..7 and 120x120 RGB888 image",
            Self::InvalidTouchImage => "expected 800x100 RGB888 image",
            Self::InvalidBrightness => "brightness must be 0..100",
        })
    }
}
impl std::error::Error for DeviceError {}

pub trait DeckDevice {
    fn geometry(&self) -> Geometry;
    fn set_key_image(&mut self, index: u8, rgb: &[u8]) -> Result<(), DeviceError>;
    fn set_touch_image(&mut self, rgb: &[u8]) -> Result<(), DeviceError>;
    fn set_brightness(&mut self, percent: u8) -> Result<(), DeviceError>;
    fn poll_event(&mut self) -> Result<Option<InputEvent>, DeviceError>;
    /// Clear all display surfaces without changing the saved brightness.
    /// Attempt every surface even when a previous write fails.
    fn blank(&mut self) -> Result<(), DeviceError> {
        let geometry = self.geometry();
        let key =
            vec![0; usize::from(geometry.key_pixels.0) * usize::from(geometry.key_pixels.1) * 3];
        let touch =
            vec![
                0;
                usize::from(geometry.touch_pixels.0) * usize::from(geometry.touch_pixels.1) * 3
            ];
        let mut result = Ok(());
        for index in 0..geometry.columns.saturating_mul(geometry.rows) {
            if let Err(error) = self.set_key_image(index, &key) {
                result = Err(error);
            }
        }
        if let Err(error) = self.set_touch_image(&touch) {
            result = Err(error);
        }
        result
    }
}
/// Deterministic in-memory reference device; image buffers use RGB888.
#[derive(Debug)]
pub struct VirtualDeck {
    pub brightness: u8,
    connected: bool,
    keys: [Vec<u8>; 8],
    touch: Vec<u8>,
    events: VecDeque<InputEvent>,
    last_timestamp: Option<u64>,
}
impl Default for VirtualDeck {
    fn default() -> Self {
        Self {
            brightness: 50,
            connected: true,
            keys: std::array::from_fn(|_| vec![]),
            touch: vec![],
            events: VecDeque::new(),
            last_timestamp: None,
        }
    }
}
impl VirtualDeck {
    pub fn receive_event(&mut self) -> Option<InputEvent> {
        self.events.pop_front()
    }
    pub fn is_connected(&self) -> bool {
        self.connected
    }
    /// Drop queued input; retain frame snapshots for diagnostics. Callers must
    /// also cancel their control resolvers when disconnecting a device.
    pub fn disconnect(&mut self) {
        self.connected = false;
        self.events.clear();
    }
    /// Reconnect keeps the monotonic clock and last frame snapshots.
    pub fn reconnect(&mut self) {
        self.connected = true;
    }
    pub fn key_image(&self, index: u8) -> Option<&[u8]> {
        self.keys.get(index as usize).map(Vec::as_slice)
    }
    pub fn touch_image(&self) -> &[u8] {
        &self.touch
    }
    fn require_connected(&self) -> Result<(), DeviceError> {
        if self.connected {
            Ok(())
        } else {
            Err(DeviceError::Disconnected)
        }
    }

    pub fn inject(&mut self, input: InputEvent) -> Result<(), DeviceError> {
        self.require_connected()?;
        if self.last_timestamp.is_some_and(|t| input.timestamp_ms < t) {
            return Err(DeviceError::TimeReversed);
        }
        match input.event {
            RawEvent::Key { index, .. } if index >= 8 => {
                return Err(DeviceError::InvalidKey);
            }
            RawEvent::DialPush { index, .. } | RawEvent::DialRotate { index, .. } if index >= 4 => {
                return Err(DeviceError::InvalidDial);
            }
            RawEvent::DialRotate { ticks: 0, .. } => {
                return Err(DeviceError::ZeroRotation);
            }
            RawEvent::Touch { x, y, .. } if x >= 800 || y >= 100 => {
                return Err(DeviceError::InvalidTouch);
            }
            _ => {}
        }
        if self.events.len() >= 4096 {
            return Err(DeviceError::QueueFull);
        }
        self.last_timestamp = Some(input.timestamp_ms);
        self.events.push_back(input);
        Ok(())
    }
}
impl DeckDevice for VirtualDeck {
    fn geometry(&self) -> Geometry {
        Geometry::plus()
    }
    fn set_key_image(&mut self, index: u8, rgb: &[u8]) -> Result<(), DeviceError> {
        self.require_connected()?;
        if index >= 8 || rgb.len() != 120 * 120 * 3 {
            return Err(DeviceError::InvalidKeyImage);
        }
        self.keys[index as usize] = rgb.to_vec();
        Ok(())
    }
    fn set_touch_image(&mut self, rgb: &[u8]) -> Result<(), DeviceError> {
        self.require_connected()?;
        if rgb.len() != 800 * 100 * 3 {
            return Err(DeviceError::InvalidTouchImage);
        }
        self.touch = rgb.to_vec();
        Ok(())
    }
    fn set_brightness(&mut self, percent: u8) -> Result<(), DeviceError> {
        self.require_connected()?;
        if percent > 100 {
            return Err(DeviceError::InvalidBrightness);
        }
        self.brightness = percent;
        Ok(())
    }
    fn poll_event(&mut self) -> Result<Option<InputEvent>, DeviceError> {
        self.require_connected()?;
        Ok(self.events.pop_front())
    }
}
#[cfg(test)]
mod tests {
    #[test]
    fn blank_clears_keys_and_touch_without_changing_brightness() {
        use super::*;
        let mut device = VirtualDeck::default();
        device.set_brightness(72).unwrap();
        for i in 0..8 {
            device.set_key_image(i, &vec![255; 120 * 120 * 3]).unwrap();
        }
        device.set_touch_image(&vec![255; 800 * 100 * 3]).unwrap();
        device.blank().unwrap();
        for i in 0..8 {
            assert!(device.key_image(i).unwrap().iter().all(|v| *v == 0));
        }
        assert!(device.touch_image().iter().all(|v| *v == 0));
        assert_eq!(device.brightness, 72);
    }
    #[test]
    fn blank_attempts_every_surface_even_after_a_write_error() {
        use super::*;
        struct Faulty {
            keys: Vec<u8>,
            touch: bool,
        }
        impl DeckDevice for Faulty {
            fn geometry(&self) -> Geometry {
                Geometry::plus()
            }
            fn set_key_image(&mut self, i: u8, _: &[u8]) -> Result<(), DeviceError> {
                self.keys.push(i);
                Err(DeviceError::Transport)
            }
            fn set_touch_image(&mut self, _: &[u8]) -> Result<(), DeviceError> {
                self.touch = true;
                Ok(())
            }
            fn set_brightness(&mut self, _: u8) -> Result<(), DeviceError> {
                panic!("must preserve brightness")
            }
            fn poll_event(&mut self) -> Result<Option<InputEvent>, DeviceError> {
                Ok(None)
            }
        }
        let mut device = Faulty {
            keys: vec![],
            touch: false,
        };
        assert!(device.blank().is_err());
        assert_eq!(device.keys, (0..8).collect::<Vec<_>>());
        assert!(device.touch);
    }

    use super::*;
    use decksmith_core::TouchGesture;
    #[test]
    fn validates_input_and_retains_fifo() {
        let mut d = VirtualDeck::default();
        let first = InputEvent {
            timestamp_ms: 10,
            event: RawEvent::Key {
                index: 0,
                pressed: true,
            },
        };
        d.inject(first.clone()).unwrap();
        assert!(
            d.inject(InputEvent {
                timestamp_ms: 9,
                event: first.event.clone()
            })
            .is_err()
        );
        assert!(
            d.inject(InputEvent {
                timestamp_ms: 11,
                event: RawEvent::Key {
                    index: 8,
                    pressed: true
                }
            })
            .is_err()
        );
        d.inject(InputEvent {
            timestamp_ms: 12,
            event: RawEvent::DialRotate {
                index: 3,
                ticks: -2,
            },
        })
        .unwrap();
        assert_eq!(d.receive_event(), Some(first));
        assert_eq!(d.receive_event().unwrap().timestamp_ms, 12);
        assert_eq!(d.receive_event(), None);
    }
    #[test]
    fn rejects_bad_images_without_mutation() {
        let mut d = VirtualDeck::default();
        d.set_key_image(7, &vec![255; 120 * 120 * 3]).unwrap();
        assert!(d.set_key_image(7, &[0]).is_err());
        assert_eq!(d.keys[7].len(), 120 * 120 * 3);
        assert!(d.set_touch_image(&[]).is_err());
        assert!(d.set_brightness(101).is_err());
        assert_eq!(d.brightness, 50);
    }
    #[test]
    fn checks_touch_and_dial_boundaries() {
        let mut d = VirtualDeck::default();
        for event in [
            RawEvent::Touch {
                x: 800,
                y: 0,
                gesture: TouchGesture::Tap,
            },
            RawEvent::DialPush {
                index: 4,
                pressed: true,
            },
            RawEvent::DialRotate { index: 0, ticks: 0 },
        ] {
            assert!(
                d.inject(InputEvent {
                    timestamp_ms: 0,
                    event
                })
                .is_err()
            );
        }
    }
}

// Discovery sends no HID commands, resets, or display writes.
use serde::Serialize;
use std::{fs, io, path::Path};
#[derive(Debug, Serialize)]
pub struct Probe {
    pub model: String,
    pub path: String,
    pub read_access: bool,
    pub error: Option<String>,
}
pub fn devices() -> io::Result<Vec<Probe>> {
    discover(Path::new("/sys/class/hidraw"), Path::new("/dev"))
}
fn discover(sys: &Path, dev: &Path) -> io::Result<Vec<Probe>> {
    let mut found = Vec::new();
    for entry in fs::read_dir(sys)? {
        let entry = entry?;
        let data = fs::read_to_string(entry.path().join("device/uevent"))?;
        if !data.lines().any(|l| l == "HID_ID=0003:00000FD9:00000084") {
            continue;
        }
        let path = dev.join(entry.file_name());
        let opened = fs::File::open(&path);
        found.push(Probe {
            model: "Stream Deck +".into(),
            path: path.display().to_string(),
            read_access: opened.is_ok(),
            error: opened.err().map(|e| e.to_string()),
        });
    }
    found.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(found)
}

#[cfg(feature = "hardware")]
pub mod hardware;

#[cfg(feature = "hardware")]
pub mod ownership;
