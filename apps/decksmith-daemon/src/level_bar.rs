//! Value-driven glossy level indicator for a 200 by 100 touch-strip panel.
pub fn draw(rgb: &mut [u8], percent: Option<u8>, muted: bool) {
    draw_tinted(rgb, percent, muted, None);
}
#[allow(dead_code)] // Standalone caption examples only use the ordinary bar.
pub fn signal_tint(level: u8) -> Option<[f32; 3]> {
    match level {
        95..=255 => Some([230., 45., 40.]),
        90..=94 => Some([235., 115., 15.]),
        80..=89 => Some([225., 195., 15.]),
        _ => None,
    }
}
#[allow(dead_code)] // Standalone caption examples only use the ordinary bar.
pub fn draw_signal(rgb: &mut [u8], level: u8) {
    draw_tinted(rgb, Some(level), false, signal_tint(level));
}
fn draw_tinted(rgb: &mut [u8], percent: Option<u8>, muted: bool, tint: Option<[f32; 3]>) {
    let level = f32::from(percent.unwrap_or(0).min(100)) / 100.0;
    for y in 72..96 {
        for x in 9..191 {
            let mut sum = [0.0; 3];
            for sy in 0..4 {
                for sx in 0..4 {
                    let px = x as f32 + (sx as f32 + 0.5) / 4.0;
                    let py = y as f32 + (sy as f32 + 0.5) / 4.0;
                    let mut color = [30.0, 34.0, 39.0];
                    if capsule(px, py, 10.0, 73.0, 180.0, 22.0) {
                        let shine = 1.0 - ((py - 77.0) / 18.0).clamp(0.0, 1.0);
                        color = [65.0 + 80.0 * shine; 3];
                    }
                    if capsule(px, py, 12.0, 75.0, 176.0, 18.0) {
                        let shade = 1.0 - ((py - 84.0).abs() / 9.0).clamp(0.0, 1.0);
                        color = [
                            20.0 + 22.0 * shade,
                            23.0 + 22.0 * shade,
                            27.0 + 22.0 * shade,
                        ];
                    }
                    if level > 0.0 && capsule(px, py, 14.0, 77.0, 172.0 * level, 14.0) {
                        let shine = 1.0 - ((py - 79.0) / 12.0).clamp(0.0, 1.0);
                        color = if muted {
                            [65.0 + 55.0 * shine; 3]
                        } else if let Some(tint) = tint {
                            tint.map(|channel| channel + (255. - channel) * shine * 0.45)
                        } else {
                            [
                                8.0 + 70.0 * shine,
                                115.0 + 85.0 * shine,
                                195.0 + 55.0 * shine,
                            ]
                        };
                    }
                    for c in 0..3 {
                        sum[c] += color[c];
                    }
                }
            }
            for c in 0..3 {
                rgb[(y * 200 + x) * 3 + c] = (sum[c] / 16.0).round() as u8;
            }
        }
    }
}
fn capsule(x: f32, y: f32, left: f32, top: f32, width: f32, height: f32) -> bool {
    let radius = (height / 2.0).min(width / 2.0);
    // Centered distances avoid independently rounded clamp endpoints becoming
    // inverted for narrow fills (for example a 4% live signal).
    let dx = ((x - (left + width / 2.0)).abs() - (width / 2.0 - radius).max(0.0)).max(0.0);
    let dy = ((y - (top + height / 2.0)).abs() - (height / 2.0 - radius).max(0.0)).max(0.0);
    dx * dx + dy * dy <= radius * radius
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn every_small_fill_is_safe_for_signal_and_brightness() {
        for level in 0..=100 {
            let mut rgb = [30, 34, 39].repeat(200 * 100);
            draw_signal(&mut rgb, level);
            draw(&mut rgb, Some(level), false);
            draw(&mut rgb, Some(level), true);
        }
    }
    #[test]
    fn fill_is_bounded_and_muted_is_neutral() {
        let base = [30, 34, 39].repeat(200 * 100);
        let mut empty = base.clone();
        draw(&mut empty, Some(0), false);
        let mut half = base.clone();
        draw(&mut half, Some(50), false);
        let mut full = base.clone();
        draw(&mut full, Some(100), false);
        let blue = |image: &[u8]| {
            image
                .chunks_exact(3)
                .filter(|p| p[2] > 150 && p[2] > p[0])
                .count()
        };
        assert_eq!(blue(&empty), 0);
        assert!(blue(&half) > 0);
        assert!(blue(&full) > blue(&half));
        assert_eq!(&full[..72 * 200 * 3], &base[..72 * 200 * 3]);
        let mut muted = base;
        draw(&mut muted, Some(100), true);
        assert_eq!(blue(&muted), 0);
    }
}
