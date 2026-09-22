//! Synchronous Plus adapter. Own on a dedicated device worker, never a UI thread.
//! Construction opens a device but performs no reset, brightness, or image write.
use crate::{DeckDevice, DeviceError};
use decksmith_core::{Geometry, InputEvent, RawEvent, TouchGesture};
use elgato_streamdeck::{StreamDeck, StreamDeckInput, images::ImageRect, info::Kind};
use image::{DynamicImage, RgbImage};
use serde::Serialize;
use std::{
    collections::VecDeque,
    time::{Duration, Instant},
};

#[derive(Debug, Serialize)]
pub struct HardwareInfo {
    pub model: &'static str,
    pub firmware: String,
}
// All upstream types remain within this module's private transport boundary.
trait Transport: Send {
    fn read(&mut self) -> Result<StreamDeckInput, DeviceError>;
    fn firmware(&self) -> Result<String, DeviceError>;
    fn key(&mut self, index: u8, image: DynamicImage) -> Result<(), DeviceError>;
    fn strip(&mut self, image: DynamicImage) -> Result<(), DeviceError>;
    fn brightness(&mut self, percent: u8) -> Result<(), DeviceError>;
}
struct UsbTransport(StreamDeck);
impl Transport for UsbTransport {
    fn read(&mut self) -> Result<StreamDeckInput, DeviceError> {
        self.0
            .read_input(Some(Duration::from_millis(20)))
            .map_err(|_| DeviceError::Transport)
    }
    fn firmware(&self) -> Result<String, DeviceError> {
        self.0
            .firmware_version()
            .map_err(|_| DeviceError::Transport)
    }
    fn key(&mut self, index: u8, image: DynamicImage) -> Result<(), DeviceError> {
        self.0
            .set_button_image(index, image)
            .map_err(|_| DeviceError::Transport)?;
        self.0.flush().map_err(|_| DeviceError::Transport)
    }
    fn strip(&mut self, image: DynamicImage) -> Result<(), DeviceError> {
        let rect = ImageRect::from_image(image).map_err(|_| DeviceError::ImageEncoding)?;
        self.0
            .write_lcd(0, 0, &rect)
            .map_err(|_| DeviceError::Transport)
    }
    fn brightness(&mut self, percent: u8) -> Result<(), DeviceError> {
        self.0
            .set_brightness(percent)
            .map_err(|_| DeviceError::Transport)
    }
}
/// One physical Plus session. Closing drops the handle without resetting the deck.
/// Reconnect by opening a new session; state is never reused across USB sessions.
pub struct PhysicalDeck {
    transport: Box<dyn Transport>,
    normalizer: Normalizer,
    queue: VecDeque<InputEvent>,
    epoch: Instant,
    _ownership: Option<std::fs::File>,
}
impl PhysicalDeck {
    /// The caller must arrange sole application ownership before calling this.
    /// Rejects ambiguity rather than selecting an arbitrary identical device.
    pub fn open_only_plus() -> Result<Self, DeviceError> {
        let ownership = crate::ownership::acquire()?;
        let api = elgato_streamdeck::new_hidapi().map_err(|_| DeviceError::Transport)?;
        let mut devices = elgato_streamdeck::list_devices(&api)
            .into_iter()
            .filter(|(kind, _)| matches!(kind, Kind::Plus));
        let (_, serial) = devices.next().ok_or(DeviceError::DeviceSelection)?;
        if devices.next().is_some() {
            return Err(DeviceError::DeviceSelection);
        }
        let deck =
            StreamDeck::connect(&api, Kind::Plus, &serial).map_err(|_| DeviceError::Transport)?;
        let mut physical = Self::with_transport(Box::new(UsbTransport(deck)));
        physical._ownership = Some(ownership);
        Ok(physical)
    }
    fn with_transport(transport: Box<dyn Transport>) -> Self {
        Self {
            transport,
            normalizer: Normalizer::default(),
            queue: VecDeque::new(),
            epoch: Instant::now(),
            _ownership: None,
        }
    }
    pub fn info(&self) -> Result<HardwareInfo, DeviceError> {
        Ok(HardwareInfo {
            model: "Stream Deck +",
            firmware: self.transport.firmware()?,
        })
    }
}
impl DeckDevice for PhysicalDeck {
    fn geometry(&self) -> Geometry {
        Geometry::plus()
    }
    fn set_key_image(&mut self, index: u8, rgb: &[u8]) -> Result<(), DeviceError> {
        if index >= 8 {
            return Err(DeviceError::InvalidKey);
        }
        let image = rgb_image(rgb, 120, 120).ok_or(DeviceError::InvalidKeyImage)?;
        self.transport.key(index, image)
    }
    fn set_touch_image(&mut self, rgb: &[u8]) -> Result<(), DeviceError> {
        let image = rgb_image(rgb, 800, 100).ok_or(DeviceError::InvalidTouchImage)?;
        self.transport.strip(image)
    }
    fn set_brightness(&mut self, percent: u8) -> Result<(), DeviceError> {
        if percent > 100 {
            return Err(DeviceError::InvalidBrightness);
        }
        self.transport.brightness(percent)
    }
    fn poll_event(&mut self) -> Result<Option<InputEvent>, DeviceError> {
        if let Some(event) = self.queue.pop_front() {
            return Ok(Some(event));
        }
        let report = self.transport.read()?;
        let timestamp_ms = u64::try_from(self.epoch.elapsed().as_millis())
            .map_err(|_| DeviceError::TimeReversed)?;
        let events = self.normalizer.normalize(report)?;
        self.queue
            .extend(events.into_iter().map(|event| InputEvent {
                timestamp_ms,
                event,
            }));
        Ok(self.queue.pop_front())
    }
}
fn rgb_image(rgb: &[u8], width: u32, height: u32) -> Option<DynamicImage> {
    if rgb.len() != width as usize * height as usize * 3 {
        return None;
    }
    RgbImage::from_raw(width, height, rgb.to_vec()).map(DynamicImage::ImageRgb8)
}
#[derive(Default)]
struct Normalizer {
    keys: [bool; 8],
    dials: [bool; 4],
}
impl Normalizer {
    fn normalize(&mut self, report: StreamDeckInput) -> Result<Vec<RawEvent>, DeviceError> {
        let mut events = Vec::new();
        match report {
            StreamDeckInput::NoData => {}
            StreamDeckInput::ButtonStateChange(mut states) => {
                // elgato-streamdeck 0.13.1 reads a 14-byte Plus buffer and maps
                // bytes 4..14 as keys, including two zero padding bytes.
                if states.len() == 10 && !states[8] && !states[9] {
                    states.truncate(8);
                }
                if states.len() != 8 {
                    return Err(DeviceError::InvalidReport);
                }
                for (index, pressed) in states.into_iter().enumerate() {
                    if self.keys[index] != pressed {
                        events.push(RawEvent::Key {
                            index: index as u8,
                            pressed,
                        });
                        self.keys[index] = pressed;
                    }
                }
            }
            StreamDeckInput::EncoderStateChange(states) => {
                if states.len() != 4 {
                    return Err(DeviceError::InvalidReport);
                }
                for (index, pressed) in states.into_iter().enumerate() {
                    if self.dials[index] != pressed {
                        events.push(RawEvent::DialPush {
                            index: index as u8,
                            pressed,
                        });
                        self.dials[index] = pressed;
                    }
                }
            }
            StreamDeckInput::EncoderTwist(ticks) => {
                if ticks.len() != 4 {
                    return Err(DeviceError::InvalidReport);
                }
                for (index, ticks) in ticks.into_iter().enumerate() {
                    if ticks != 0 {
                        events.push(RawEvent::DialRotate {
                            index: index as u8,
                            ticks: i16::from(ticks),
                        });
                    }
                }
            }
            StreamDeckInput::TouchScreenPress(x, y) => events.push(touch(x, y, TouchGesture::Tap)?),
            StreamDeckInput::TouchScreenLongPress(x, y) => {
                events.push(touch(x, y, TouchGesture::LongPress)?)
            }
            StreamDeckInput::TouchScreenSwipe((x, y), (end_x, end_y)) => {
                if end_x >= 800 || end_y >= 100 {
                    return Err(DeviceError::InvalidReport);
                }
                if x >= 800 || y >= 100 {
                    return Err(DeviceError::InvalidReport);
                }
                // Only exposed horizontal flicks map into the current domain model.
                if end_x != x {
                    events.push(touch(
                        x,
                        y,
                        if end_x > x {
                            TouchGesture::FlickRight
                        } else {
                            TouchGesture::FlickLeft
                        },
                    )?);
                }
            }
        }
        Ok(events)
    }
}
fn touch(x: u16, y: u16, gesture: TouchGesture) -> Result<RawEvent, DeviceError> {
    if x >= 800 || y >= 100 {
        return Err(DeviceError::InvalidReport);
    }
    Ok(RawEvent::Touch { x, y, gesture })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn state_reports_become_edges_and_signed_ticks() {
        let mut n = Normalizer::default();
        let mut keys = vec![false; 8];
        keys[7] = true;
        assert_eq!(
            n.normalize(StreamDeckInput::ButtonStateChange(keys.clone()))
                .unwrap(),
            vec![RawEvent::Key {
                index: 7,
                pressed: true
            }]
        );
        assert!(
            n.normalize(StreamDeckInput::ButtonStateChange(keys))
                .unwrap()
                .is_empty()
        );
        assert_eq!(
            n.normalize(StreamDeckInput::ButtonStateChange(vec![false; 8]))
                .unwrap(),
            vec![RawEvent::Key {
                index: 7,
                pressed: false
            }]
        );
        assert_eq!(
            n.normalize(StreamDeckInput::EncoderTwist(vec![-128, 0, 1, 127]))
                .unwrap(),
            vec![
                RawEvent::DialRotate {
                    index: 0,
                    ticks: -128
                },
                RawEvent::DialRotate { index: 2, ticks: 1 },
                RawEvent::DialRotate {
                    index: 3,
                    ticks: 127
                }
            ]
        );
    }
    #[test]
    fn invalid_reports_do_not_mutate_state() {
        let mut n = Normalizer::default();
        assert!(
            n.normalize(StreamDeckInput::ButtonStateChange(vec![true; 9]))
                .is_err()
        );
        assert_eq!(n.keys, [false; 8]);
        assert!(
            n.normalize(StreamDeckInput::EncoderStateChange(vec![true; 3]))
                .is_err()
        );
        assert!(
            n.normalize(StreamDeckInput::TouchScreenPress(800, 0))
                .is_err()
        );
        assert_eq!(
            n.normalize(StreamDeckInput::TouchScreenSwipe((100, 50), (20, 50)))
                .unwrap(),
            vec![RawEvent::Touch {
                x: 100,
                y: 50,
                gesture: TouchGesture::FlickLeft
            }]
        );
    }
    #[test]
    fn upstream_plus_padding_is_not_extra_keys() {
        let mut n = Normalizer::default();
        let mut states = vec![false; 10];
        states[0] = true;
        assert_eq!(
            n.normalize(StreamDeckInput::ButtonStateChange(states))
                .unwrap(),
            vec![RawEvent::Key {
                index: 0,
                pressed: true
            }]
        );
        let mut invalid = vec![false; 10];
        invalid[9] = true;
        assert_eq!(
            n.normalize(StreamDeckInput::ButtonStateChange(invalid)),
            Err(DeviceError::InvalidReport)
        );
        assert!(n.keys[0]);
        assert_eq!(
            n.normalize(StreamDeckInput::ButtonStateChange(vec![false; 10]))
                .unwrap(),
            vec![RawEvent::Key {
                index: 0,
                pressed: false
            }]
        );
    }
    struct Fake;
    impl Transport for Fake {
        fn read(&mut self) -> Result<StreamDeckInput, DeviceError> {
            Err(DeviceError::Transport)
        }
        fn firmware(&self) -> Result<String, DeviceError> {
            Ok("fixture".into())
        }
        fn key(&mut self, _: u8, _: DynamicImage) -> Result<(), DeviceError> {
            Err(DeviceError::Transport)
        }
        fn strip(&mut self, _: DynamicImage) -> Result<(), DeviceError> {
            Err(DeviceError::Transport)
        }
        fn brightness(&mut self, _: u8) -> Result<(), DeviceError> {
            Err(DeviceError::Transport)
        }
    }
    #[test]
    fn validates_before_transport_and_preserves_errors() {
        let mut d = PhysicalDeck::with_transport(Box::new(Fake));
        assert_eq!(d.info().unwrap().firmware, "fixture");
        assert_eq!(d.set_brightness(101), Err(DeviceError::InvalidBrightness));
        assert_eq!(d.set_brightness(50), Err(DeviceError::Transport));
        assert_eq!(d.set_key_image(8, &[]), Err(DeviceError::InvalidKey));
        assert_eq!(d.set_key_image(0, &[]), Err(DeviceError::InvalidKeyImage));
        assert_eq!(d.set_touch_image(&[]), Err(DeviceError::InvalidTouchImage));
        assert_eq!(d.poll_event(), Err(DeviceError::Transport));
    }
    #[test]
    fn encoded_images_match_plus_dimensions() {
        let key = rgb_image(&vec![80; 120 * 120 * 3], 120, 120).unwrap();
        let bytes = elgato_streamdeck::images::convert_image(Kind::Plus, key).unwrap();
        let decoded = image::load_from_memory(&bytes).unwrap();
        assert_eq!((decoded.width(), decoded.height()), (120, 120));
        let strip =
            ImageRect::from_image(rgb_image(&vec![90; 800 * 100 * 3], 800, 100).unwrap()).unwrap();
        assert_eq!((strip.w, strip.h), (800, 100));
        let decoded = image::load_from_memory(&strip.data).unwrap();
        assert_eq!((decoded.width(), decoded.height()), (800, 100));
    }
}
