//! Synchronous Plus adapter. Own on a dedicated device worker, never a UI thread.
//! Construction opens a device but performs no reset, brightness, or image write.
use crate::{DeckDevice, DeviceError, InputCounts};
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
    fn read(&mut self, timeout: Duration) -> Result<StreamDeckInput, DeviceError>;
    fn firmware(&self) -> Result<String, DeviceError>;
    fn key(&mut self, index: u8, image: DynamicImage) -> Result<(), DeviceError>;
    fn strip(&mut self, x: u16, y: u16, image: DynamicImage) -> Result<(), DeviceError>;
    fn brightness(&mut self, percent: u8) -> Result<(), DeviceError>;
}
struct UsbTransport(StreamDeck);
impl Transport for UsbTransport {
    fn read(&mut self, timeout: Duration) -> Result<StreamDeckInput, DeviceError> {
        self.0
            .read_input(Some(timeout))
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
    fn strip(&mut self, x: u16, y: u16, image: DynamicImage) -> Result<(), DeviceError> {
        let rect = ImageRect::from_image(image).map_err(|_| DeviceError::ImageEncoding)?;
        self.0
            .write_lcd(x, y, &rect)
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
    queue: VecDeque<(InputEvent, Instant)>,
    event_received_at: Option<Instant>,
    input_counts: Option<Box<InputCounts>>,
    epoch: Instant,
    _ownership: Option<std::fs::File>,
    touch_frame: Option<Vec<u8>>,
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
            event_received_at: None,
            input_counts: None,
            epoch: Instant::now(),
            _ownership: None,
            touch_frame: None,
        }
    }
    fn poll_with_timeout(&mut self, timeout: Duration) -> Result<Option<InputEvent>, DeviceError> {
        self.event_received_at = None;
        if let Some((event, received)) = self.queue.pop_front() {
            self.event_received_at = Some(received);
            return Ok(Some(event));
        }
        if let Some(counts) = self.input_counts.as_mut() {
            counts.reads = counts.reads.saturating_add(1);
        }
        let report = match self.transport.read(timeout) {
            Ok(report) => report,
            Err(error) => {
                if let Some(counts) = self.input_counts.as_mut() {
                    counts.transport_errors = counts.transport_errors.saturating_add(1);
                }
                return Err(error);
            }
        };
        let received = Instant::now();
        let timestamp_ms = u64::try_from(received.duration_since(self.epoch).as_millis())
            .map_err(|_| DeviceError::TimeReversed)?;
        if let Some(counts) = self.input_counts.as_mut() {
            let count = match &report {
                StreamDeckInput::NoData => None,
                StreamDeckInput::ButtonStateChange(_) => Some(&mut counts.key_reports),
                StreamDeckInput::EncoderStateChange(_) => Some(&mut counts.dial_push_reports),
                StreamDeckInput::EncoderTwist(_) => Some(&mut counts.dial_turn_reports),
                StreamDeckInput::TouchScreenPress(..)
                | StreamDeckInput::TouchScreenLongPress(..)
                | StreamDeckInput::TouchScreenSwipe(..) => Some(&mut counts.touch_reports),
            };
            if let Some(count) = count {
                *count = count.saturating_add(1);
            }
        }
        let events = match self.normalizer.normalize(report) {
            Ok(events) => events,
            Err(error) => {
                if let Some(counts) = self.input_counts.as_mut() {
                    counts.invalid_reports = counts.invalid_reports.saturating_add(1);
                }
                return Err(error);
            }
        };
        if let Some(counts) = self.input_counts.as_mut() {
            for event in &events {
                let count = match event {
                    RawEvent::Key { .. } => &mut counts.key_events,
                    RawEvent::DialPush { .. } => &mut counts.dial_push_events,
                    RawEvent::DialRotate { .. } => &mut counts.dial_turn_events,
                    RawEvent::Touch { .. } => &mut counts.touch_events,
                };
                *count = count.saturating_add(1);
            }
        }
        self.queue.extend(events.into_iter().map(|event| {
            (
                InputEvent {
                    timestamp_ms,
                    event,
                },
                received,
            )
        }));
        Ok(self.queue.pop_front().map(|(event, received)| {
            self.event_received_at = Some(received);
            event
        }))
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
        if rgb.len() != 800 * 100 * 3 {
            return Err(DeviceError::InvalidTouchImage);
        }
        let region = match &self.touch_frame {
            Some(previous) => match changed_touch(previous, rgb) {
                Some(region) => region,
                None => return Ok(()),
            },
            None => TouchRegion {
                x: 0,
                y: 0,
                width: 800,
                height: 100,
            },
        };
        if let Err(error) = self
            .transport
            .strip(region.x, region.y, region_image(rgb, region))
        {
            // A failed multi-report write may have changed part of the display.
            // Repaint the full frame on retry; never advance an uncertain baseline.
            self.touch_frame = None;
            return Err(error);
        }
        match &mut self.touch_frame {
            Some(previous) => previous.copy_from_slice(rgb),
            None => self.touch_frame = Some(rgb.to_vec()),
        }
        Ok(())
    }
    fn blank(&mut self) -> Result<(), DeviceError> {
        // Shutdown always clears the complete strip, even when cached pixels match.
        self.touch_frame = None;
        crate::blank_display(self)
    }
    fn set_brightness(&mut self, percent: u8) -> Result<(), DeviceError> {
        if percent > 100 {
            return Err(DeviceError::InvalidBrightness);
        }
        self.transport.brightness(percent)
    }
    fn poll_event(&mut self) -> Result<Option<InputEvent>, DeviceError> {
        self.poll_with_timeout(Duration::from_millis(20))
    }
    fn poll_event_nonblocking(&mut self) -> Result<Option<InputEvent>, DeviceError> {
        self.poll_with_timeout(Duration::ZERO)
    }
    fn event_received_at(&self) -> Option<Instant> {
        self.event_received_at
    }
    fn set_input_diagnostics(&mut self, enabled: bool) {
        self.input_counts = enabled.then(|| Box::new(InputCounts::default()));
    }
    fn input_counts(&self) -> Option<InputCounts> {
        self.input_counts.as_deref().copied()
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct TouchRegion {
    x: u16,
    y: u16,
    width: u16,
    height: u16,
}
fn changed_touch(previous: &[u8], rgb: &[u8]) -> Option<TouchRegion> {
    const ROW: usize = 800 * 3;
    let mut rows = previous.chunks_exact(ROW).zip(rgb.chunks_exact(ROW));
    let top = rows.position(|(a, b)| a != b)?;
    let bottom = top + 1 + rows.rposition(|(a, b)| a != b).map_or(0, |i| i + 1);
    let mut left = 800;
    let mut right = 0;
    for y in top..bottom {
        let a = &previous[y * ROW..(y + 1) * ROW];
        let b = &rgb[y * ROW..(y + 1) * ROW];
        for (x, (a, b)) in a.chunks_exact(3).zip(b.chunks_exact(3)).enumerate() {
            if a != b {
                left = left.min(x);
                right = right.max(x + 1);
            }
        }
    }
    // Preserve the full-frame JPEG block grid, including unchanged edge pixels.
    let x = left / 8 * 8;
    let y = top / 8 * 8;
    Some(TouchRegion {
        x: x as u16,
        y: y as u16,
        width: (right.div_ceil(8) * 8 - x) as u16,
        height: ((bottom.div_ceil(8) * 8).min(100) - y) as u16,
    })
}
fn region_image(rgb: &[u8], region: TouchRegion) -> DynamicImage {
    let width = usize::from(region.width);
    let mut pixels = Vec::with_capacity(width * usize::from(region.height) * 3);
    for y in usize::from(region.y)..usize::from(region.y + region.height) {
        let start = (y * 800 + usize::from(region.x)) * 3;
        pixels.extend_from_slice(&rgb[start..start + width * 3]);
    }
    DynamicImage::ImageRgb8(
        RgbImage::from_raw(u32::from(region.width), u32::from(region.height), pixels).unwrap(),
    )
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
        fn read(&mut self, _: Duration) -> Result<StreamDeckInput, DeviceError> {
            Err(DeviceError::Transport)
        }
        fn firmware(&self) -> Result<String, DeviceError> {
            Ok("fixture".into())
        }
        fn key(&mut self, _: u8, _: DynamicImage) -> Result<(), DeviceError> {
            Err(DeviceError::Transport)
        }
        fn strip(&mut self, _: u16, _: u16, _: DynamicImage) -> Result<(), DeviceError> {
            Err(DeviceError::Transport)
        }
        fn brightness(&mut self, _: u8) -> Result<(), DeviceError> {
            Err(DeviceError::Transport)
        }
    }
    struct Reports(VecDeque<Result<StreamDeckInput, DeviceError>>);
    impl Transport for Reports {
        fn read(&mut self, _: Duration) -> Result<StreamDeckInput, DeviceError> {
            self.0.pop_front().unwrap_or(Err(DeviceError::Transport))
        }
        fn firmware(&self) -> Result<String, DeviceError> {
            Fake.firmware()
        }
        fn key(&mut self, index: u8, image: DynamicImage) -> Result<(), DeviceError> {
            Fake.key(index, image)
        }
        fn strip(&mut self, x: u16, y: u16, image: DynamicImage) -> Result<(), DeviceError> {
            Fake.strip(x, y, image)
        }
        fn brightness(&mut self, percent: u8) -> Result<(), DeviceError> {
            Fake.brightness(percent)
        }
    }
    #[test]
    fn optional_counts_separate_reports_edges_queue_reads_and_errors() {
        let mut keys = vec![false; 10];
        keys[0] = true;
        let reports = [
            Ok(StreamDeckInput::ButtonStateChange(keys.clone())),
            Ok(StreamDeckInput::ButtonStateChange(keys)),
            Ok(StreamDeckInput::EncoderTwist(vec![1, -1, 0, 0])),
            Ok(StreamDeckInput::EncoderStateChange(vec![
                false, true, false, false,
            ])),
            Ok(StreamDeckInput::TouchScreenPress(20, 30)),
            Ok(StreamDeckInput::NoData),
            Ok(StreamDeckInput::ButtonStateChange(vec![true; 9])),
        ];
        let mut device = PhysicalDeck::with_transport(Box::new(Reports(reports.into())));
        assert!(device.input_counts().is_none());
        device.set_input_diagnostics(true);
        assert!(matches!(
            device.poll_event().unwrap().unwrap().event,
            RawEvent::Key { pressed: true, .. }
        ));
        assert_eq!(device.poll_event().unwrap(), None); // Repeated state has no edge.
        assert!(matches!(
            device.poll_event().unwrap().unwrap().event,
            RawEvent::DialRotate { ticks: 1, .. }
        ));
        assert_eq!(device.input_counts().unwrap().reads, 3);
        assert!(matches!(
            device.poll_event().unwrap().unwrap().event,
            RawEvent::DialRotate { ticks: -1, .. }
        ));
        assert_eq!(device.input_counts().unwrap().reads, 3); // Same report's queued edge.
        assert!(matches!(
            device.poll_event().unwrap().unwrap().event,
            RawEvent::DialPush { pressed: true, .. }
        ));
        assert!(matches!(
            device.poll_event().unwrap().unwrap().event,
            RawEvent::Touch { .. }
        ));
        assert_eq!(device.poll_event().unwrap(), None);
        assert_eq!(device.poll_event(), Err(DeviceError::InvalidReport));
        assert_eq!(device.poll_event(), Err(DeviceError::Transport));
        assert_eq!(
            device.input_counts(),
            Some(InputCounts {
                reads: 8,
                transport_errors: 1,
                invalid_reports: 1,
                key_reports: 3,
                dial_push_reports: 1,
                dial_turn_reports: 1,
                touch_reports: 1,
                key_events: 1,
                dial_push_events: 1,
                dial_turn_events: 2,
                touch_events: 1,
            })
        );
        device.set_input_diagnostics(false);
        assert!(device.input_counts().is_none());
        device.set_input_diagnostics(true);
        assert_eq!(device.input_counts(), Some(InputCounts::default()));
        let fresh = PhysicalDeck::with_transport(Box::new(Fake));
        assert!(fresh.input_counts().is_none());
        let mut virtual_deck = crate::VirtualDeck::default();
        virtual_deck.set_input_diagnostics(true);
        assert!(virtual_deck.input_counts().is_none());
    }
    #[test]
    fn queued_edges_retain_report_receipt_and_empty_or_failed_polls_clear_it() {
        let reports = [
            Ok(StreamDeckInput::EncoderTwist(vec![1, 1, 0, 0])),
            Ok(StreamDeckInput::NoData),
            Ok(StreamDeckInput::EncoderTwist(vec![1])),
        ];
        let mut device = PhysicalDeck::with_transport(Box::new(Reports(reports.into())));
        assert!(device.event_received_at().is_none());
        let first = device.poll_event().unwrap().unwrap();
        let received = device.event_received_at().unwrap();
        std::thread::sleep(Duration::from_millis(5));
        let second = device.poll_event_nonblocking().unwrap().unwrap();
        assert_eq!(second.timestamp_ms, first.timestamp_ms);
        assert_eq!(device.event_received_at(), Some(received));
        assert!(received.elapsed() >= Duration::from_millis(5));
        assert_eq!(device.poll_event().unwrap(), None);
        assert!(device.event_received_at().is_none());
        assert_eq!(device.poll_event(), Err(DeviceError::InvalidReport));
        assert!(device.event_received_at().is_none());
        assert_eq!(device.poll_event(), Err(DeviceError::Transport));
        assert!(device.event_received_at().is_none());
        let fresh = PhysicalDeck::with_transport(Box::new(Fake));
        assert!(fresh.event_received_at().is_none());
        assert!(crate::VirtualDeck::default().event_received_at().is_none());
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
    #[derive(Default)]
    struct Recording {
        pixels: Vec<u8>,
        writes: Vec<TouchRegion>,
        read_timeouts: Vec<Duration>,
        fail: bool,
        keys: usize,
    }
    struct Recorder(std::sync::Arc<std::sync::Mutex<Recording>>);
    impl Transport for Recorder {
        fn read(&mut self, timeout: Duration) -> Result<StreamDeckInput, DeviceError> {
            self.0.lock().unwrap().read_timeouts.push(timeout);
            Ok(StreamDeckInput::NoData)
        }
        fn firmware(&self) -> Result<String, DeviceError> {
            Ok("fixture".into())
        }
        fn key(&mut self, _: u8, _: DynamicImage) -> Result<(), DeviceError> {
            self.0.lock().unwrap().keys += 1;
            Ok(())
        }
        fn brightness(&mut self, _: u8) -> Result<(), DeviceError> {
            Ok(())
        }
        fn strip(&mut self, x: u16, y: u16, image: DynamicImage) -> Result<(), DeviceError> {
            let mut state = self.0.lock().unwrap();
            let image = image.into_rgb8();
            state.writes.push(TouchRegion {
                x,
                y,
                width: image.width() as u16,
                height: image.height() as u16,
            });
            state.pixels.resize(800 * 100 * 3, 0);
            for row in 0..image.height() as usize {
                let start = ((usize::from(y) + row) * 800 + usize::from(x)) * 3;
                let width = image.width() as usize * 3;
                state.pixels[start..start + width]
                    .copy_from_slice(&image.as_raw()[row * width..(row + 1) * width]);
            }
            if std::mem::take(&mut state.fail) {
                Err(DeviceError::Transport)
            } else {
                Ok(())
            }
        }
    }
    #[test]
    fn ready_input_poll_removes_only_the_requested_wait_and_restores_idle_timeout() {
        let recording = std::sync::Arc::new(std::sync::Mutex::new(Recording::default()));
        let mut device = PhysicalDeck::with_transport(Box::new(Recorder(recording.clone())));
        assert_eq!(device.poll_event().unwrap(), None);
        assert_eq!(device.poll_event_nonblocking().unwrap(), None);
        assert_eq!(device.poll_event().unwrap(), None);
        assert!(device.event_received_at().is_none());
        assert_eq!(
            recording.lock().unwrap().read_timeouts,
            [
                Duration::from_millis(20),
                Duration::ZERO,
                Duration::from_millis(20)
            ]
        );
    }
    #[test]
    fn touch_regions_reconstruct_frames_skip_duplicates_and_handle_edges() {
        let state = std::sync::Arc::new(std::sync::Mutex::new(Recording::default()));
        let mut deck = PhysicalDeck::with_transport(Box::new(Recorder(state.clone())));
        let mut frame = vec![25; 800 * 100 * 3];
        deck.set_touch_image(&frame).unwrap();
        assert_eq!(
            state.lock().unwrap().writes,
            vec![TouchRegion {
                x: 0,
                y: 0,
                width: 800,
                height: 100
            }]
        );
        for (i, (x, y)) in [(0, 0), (799, 99), (193, 71), (398, 94), (400, 72)]
            .into_iter()
            .enumerate()
        {
            let at = (y * 800 + x) * 3;
            frame[at..at + 3].copy_from_slice(&[200, i as u8, 0]);
            deck.set_touch_image(&frame).unwrap();
            let recorded = state.lock().unwrap();
            assert_eq!(recorded.pixels, frame);
            let region = recorded.writes.last().unwrap();
            assert_eq!(region.x % 8, 0);
            assert_eq!(region.y % 8, 0);
            assert!(region.x + region.width <= 800 && region.y + region.height <= 100);
            assert!(region.width <= 8 && region.height <= 8);
        }
        let count = state.lock().unwrap().writes.len();
        deck.set_touch_image(&frame).unwrap();
        assert_eq!(state.lock().unwrap().writes.len(), count);
        assert_eq!(
            deck.set_touch_image(&frame[..frame.len() - 1]),
            Err(DeviceError::InvalidTouchImage)
        );
        assert_eq!(state.lock().unwrap().writes.len(), count);
    }
    #[test]
    fn failed_touch_write_retries_full_frame_and_blank_always_clears_all_surfaces() {
        let state = std::sync::Arc::new(std::sync::Mutex::new(Recording::default()));
        let mut deck = PhysicalDeck::with_transport(Box::new(Recorder(state.clone())));
        let mut frame = vec![50; 800 * 100 * 3];
        deck.set_touch_image(&frame).unwrap();
        frame[(72 * 800 + 205) * 3] = 200;
        state.lock().unwrap().fail = true;
        assert_eq!(deck.set_touch_image(&frame), Err(DeviceError::Transport));
        assert!(deck.touch_frame.is_none());
        deck.set_touch_image(&frame).unwrap();
        assert_eq!(state.lock().unwrap().writes.last().unwrap().height, 100);
        assert_eq!(state.lock().unwrap().pixels, frame);
        deck.blank().unwrap();
        deck.blank().unwrap();
        let recorded = state.lock().unwrap();
        assert_eq!(recorded.keys, 16);
        assert_eq!(
            recorded.writes.last().unwrap(),
            &TouchRegion {
                x: 0,
                y: 0,
                width: 800,
                height: 100
            }
        );
        assert!(recorded.pixels.iter().all(|n| *n == 0));
        let fresh = PhysicalDeck::with_transport(Box::new(Recorder(state.clone())));
        assert!(fresh.touch_frame.is_none());
    }
    #[test]
    fn aligned_touch_regions_preserve_full_frame_jpeg_pixels() {
        let mut frame = Vec::with_capacity(800 * 100 * 3);
        for y in 0..100 {
            for x in 0..800 {
                frame.extend_from_slice(&[(x % 256) as u8, (y * 2) as u8, ((x + y) % 256) as u8]);
            }
        }
        fn encoded(image: DynamicImage) -> RgbImage {
            let region = ImageRect::from_image(image).unwrap();
            image::load_from_memory(&region.data).unwrap().into_rgb8()
        }
        let mut device = encoded(rgb_image(&frame, 800, 100).unwrap());
        for (x, y) in [(0, 0), (799, 99), (193, 71), (398, 94), (400, 72)] {
            let previous = frame.clone();
            let at = (y * 800 + x) * 3;
            frame[at..at + 3].copy_from_slice(&[240, 10, 80]);
            let region = changed_touch(&previous, &frame).unwrap();
            let patch = encoded(region_image(&frame, region));
            image::imageops::replace(
                &mut device,
                &patch,
                i64::from(region.x),
                i64::from(region.y),
            );
            assert_eq!(device, encoded(rgb_image(&frame, 800, 100).unwrap()));
        }
    }
    #[test]
    #[ignore = "native JPEG diagnostic; run explicitly in an otherwise idle test VM"]
    fn touch_region_encoding_diagnostic() {
        let baseline = vec![30; 800 * 100 * 3];
        let frames: Vec<Vec<u8>> = (0..200)
            .map(|i| {
                let mut frame = baseline.clone();
                for panel in 0..4 {
                    for y in 74..94 {
                        for x in 16..184 {
                            let at = (y * 800 + panel * 200 + x) * 3;
                            frame[at..at + 3].copy_from_slice(if x < 16 + i % 168 {
                                &[40, 130, 230]
                            } else {
                                &[50, 50, 50]
                            });
                        }
                    }
                }
                frame
            })
            .collect();
        for partial in [false, true, true, false] {
            let mut previous = baseline.clone();
            let mut pixels = 0;
            let mut bytes = 0;
            let start = Instant::now();
            for frame in &frames {
                let region = if partial {
                    changed_touch(&previous, frame).unwrap()
                } else {
                    TouchRegion {
                        x: 0,
                        y: 0,
                        width: 800,
                        height: 100,
                    }
                };
                let encoded = ImageRect::from_image(region_image(frame, region)).unwrap();
                pixels += usize::from(encoded.w) * usize::from(encoded.h);
                bytes += encoded.data.len();
                previous.copy_from_slice(frame);
            }
            println!(
                "{} frames=200 wall_ms={:.3} encoded_pixels={} jpeg_bytes={}",
                if partial { "regions" } else { "full" },
                start.elapsed().as_secs_f64() * 1000.,
                pixels,
                bytes
            );
        }
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
