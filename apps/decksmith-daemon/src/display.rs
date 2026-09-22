//! Nine latest-wins slots; one USB image write per scheduler turn.
use decksmith_core::{Geometry, InputEvent};
use decksmith_device::{DeckDevice, DeviceError};
#[derive(Default)]
pub struct Display {
    desired: [Option<Vec<u8>>; 9],
    sent: [Option<Vec<u8>>; 9],
    cursor: usize,
}
impl Display {
    pub fn sent_touch(&self) -> Option<&Vec<u8>> {
        self.sent[8].as_ref()
    }
    pub fn pending(&self) -> bool {
        self.desired
            .iter()
            .zip(&self.sent)
            .any(|(a, b)| a.is_some() && a != b)
    }
    /// Returns Some(true) when the last pending image has been written.
    pub fn flush_one(&mut self, device: &mut impl DeckDevice) -> Result<Option<bool>, DeviceError> {
        for offset in 0..9 {
            let slot = (self.cursor + offset) % 9;
            if let Some(frame) = &self.desired[slot]
                && self.sent[slot].as_ref() != Some(frame)
            {
                if slot == 8 {
                    device.set_touch_image(frame)?;
                } else {
                    device.set_key_image(slot as u8, frame)?;
                }
                self.sent[slot] = Some(frame.clone());
                self.cursor = (slot + 1) % 9;
                return Ok(Some(!self.pending()));
            }
        }
        Ok(None)
    }
}
impl DeckDevice for Display {
    fn geometry(&self) -> Geometry {
        Geometry::plus()
    }
    fn set_key_image(&mut self, index: u8, rgb: &[u8]) -> Result<(), DeviceError> {
        if index >= 8 {
            return Err(DeviceError::InvalidKey);
        }
        if rgb.len() != 120 * 120 * 3 {
            return Err(DeviceError::InvalidKeyImage);
        }
        self.desired[index as usize] = Some(rgb.to_vec());
        Ok(())
    }
    fn set_touch_image(&mut self, rgb: &[u8]) -> Result<(), DeviceError> {
        if rgb.len() != 800 * 100 * 3 {
            return Err(DeviceError::InvalidTouchImage);
        }
        self.desired[8] = Some(rgb.to_vec());
        Ok(())
    }
    fn set_brightness(&mut self, _: u8) -> Result<(), DeviceError> {
        Err(DeviceError::InvalidBrightness)
    }
    fn poll_event(&mut self) -> Result<Option<InputEvent>, DeviceError> {
        Err(DeviceError::InvalidReport)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[derive(Default)]
    struct Device {
        writes: Vec<(usize, u8)>,
        fail: bool,
    }
    impl DeckDevice for Device {
        fn geometry(&self) -> Geometry {
            Geometry::plus()
        }
        fn set_key_image(&mut self, i: u8, rgb: &[u8]) -> Result<(), DeviceError> {
            if self.fail {
                return Err(DeviceError::Transport);
            }
            self.writes.push((i as usize, rgb[0]));
            Ok(())
        }
        fn set_touch_image(&mut self, rgb: &[u8]) -> Result<(), DeviceError> {
            self.writes.push((8, rgb[0]));
            Ok(())
        }
        fn set_brightness(&mut self, _: u8) -> Result<(), DeviceError> {
            Ok(())
        }
        fn poll_event(&mut self) -> Result<Option<InputEvent>, DeviceError> {
            Ok(None)
        }
    }
    #[test]
    fn coalesces_frames_skips_duplicates_and_writes_only_one_slot() {
        let mut queue = Display::default();
        let mut device = Device::default();
        queue.set_key_image(0, &vec![1; 120 * 120 * 3]).unwrap();
        queue.set_key_image(0, &vec![2; 120 * 120 * 3]).unwrap();
        queue.set_touch_image(&vec![3; 800 * 100 * 3]).unwrap();
        assert_eq!(queue.flush_one(&mut device), Ok(Some(false)));
        assert_eq!(device.writes, vec![(0, 2)]);
        assert_eq!(queue.flush_one(&mut device), Ok(Some(true)));
        queue.set_key_image(0, &vec![2; 120 * 120 * 3]).unwrap();
        assert_eq!(queue.flush_one(&mut device), Ok(None));
        assert_eq!(device.writes, vec![(0, 2), (8, 3)]);
    }
    #[test]
    fn failed_write_is_not_marked_sent_and_reset_forces_repaint() {
        let mut queue = Display::default();
        let mut device = Device {
            fail: true,
            ..Device::default()
        };
        queue.set_key_image(0, &vec![1; 120 * 120 * 3]).unwrap();
        assert!(queue.flush_one(&mut device).is_err());
        assert!(queue.pending());
        device.fail = false;
        assert_eq!(queue.flush_one(&mut device), Ok(Some(true)));
        queue = Display::default();
        queue.set_key_image(0, &vec![1; 120 * 120 * 3]).unwrap();
        assert_eq!(queue.flush_one(&mut device), Ok(Some(true)));
        assert_eq!(device.writes.len(), 2);
    }
}
