//! Static diagnostic pattern, not a production brand asset.
use decksmith_device::{DeckDevice, DeviceError};
const DIGITS: [[u8; 5]; 8] = [
    [2, 6, 2, 2, 7],
    [7, 1, 7, 4, 7],
    [7, 1, 7, 1, 7],
    [5, 5, 7, 1, 1],
    [7, 4, 7, 1, 7],
    [7, 4, 7, 5, 7],
    [7, 1, 1, 1, 1],
    [7, 5, 7, 5, 7],
];
fn pattern(width: usize, height: usize, digit: usize) -> Vec<u8> {
    let mut rgb = vec![0; width * height * 3];
    for y in 0..height {
        for x in 0..width {
            let color = if x < 12 && y < 12 {
                [230, 50, 50]
            } else if x >= width - 12 && y < 12 {
                [40, 210, 80]
            } else if x < 12 && y >= height - 12 {
                [50, 100, 240]
            } else if x >= width - 12 && y >= height - 12 {
                [240, 210, 40]
            } else {
                [30, 34, 39]
            };
            let offset = (y * width + x) * 3;
            rgb[offset..offset + 3].copy_from_slice(&color);
        }
    }
    let scale = 10;
    let left = (width - 3 * scale) / 2;
    let top = (height - 5 * scale) / 2;
    for (row, bits) in DIGITS[digit].iter().enumerate() {
        for col in 0..3 {
            if bits & (1 << (2 - col)) != 0 {
                for y in top + row * scale..top + (row + 1) * scale {
                    for x in left + col * scale..left + (col + 1) * scale {
                        let offset = (y * width + x) * 3;
                        rgb[offset..offset + 3].copy_from_slice(&[240, 240, 240]);
                    }
                }
            }
        }
    }
    rgb
}
pub fn write(deck: &mut impl DeckDevice) -> Result<(), DeviceError> {
    for index in 0..8 {
        deck.set_key_image(index, &pattern(120, 120, index as usize))?;
    }
    let mut strip = vec![0; 800 * 100 * 3];
    for panel in 0..4 {
        let tile = pattern(200, 100, panel);
        for y in 0..100 {
            let start = (y * 800 + panel * 200) * 3;
            strip[start..start + 600].copy_from_slice(&tile[y * 600..(y + 1) * 600]);
        }
    }
    deck.set_touch_image(&strip)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn diagnostic_pattern_fits_virtual_device() {
        let mut deck = decksmith_device::VirtualDeck::default();
        write(&mut deck).unwrap();
        // Asymmetric corner colors make rotation and mirroring distinguishable.
        let tile = pattern(120, 120, 0);
        assert_eq!(&tile[..3], &[230, 50, 50]);
        assert_eq!(&tile[119 * 3..120 * 3], &[40, 210, 80]);
        assert_eq!(&tile[119 * 120 * 3..119 * 120 * 3 + 3], &[50, 100, 240]);
    }
}
