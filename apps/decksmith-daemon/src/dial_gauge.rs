//! Compact dual-arc and meter-only displays. Geometry is sampled once, never on live-meter ticks.
use std::sync::OnceLock;
use tiny_skia::{Paint, PathBuilder, Pixmap, Stroke, Transform};

struct ArcPixel {
    offset: usize,
    // Each valid supersample stores the first percentage that covers it.
    thresholds: [u8; 16],
}
struct Pixel {
    offset: usize,
    alpha: u8,
}
struct Geometry {
    volume: Vec<ArcPixel>,
    signal: Vec<ArcPixel>,
    meter: Vec<ArcPixel>,
    ticks: Vec<Pixel>,
    pointers: [Vec<Pixel>; 101],
}
const TRACK: [u8; 3] = [65, 72, 81];
const BLUE: [u8; 3] = [53, 132, 228];
const GREEN: [u8; 3] = [55, 210, 115];

fn geometry() -> &'static Geometry {
    static SHAPES: OnceLock<Geometry> = OnceLock::new();
    SHAPES.get_or_init(|| {
        let mut ticks = PathBuilder::new();
        for n in 0..=10 {
            let angle = std::f32::consts::PI * n as f32 / 10.;
            ticks.move_to(64. - 46. * angle.cos(), 91. - 46. * angle.sin());
            ticks.line_to(64. - 49. * angle.cos(), 91. - 49. * angle.sin());
        }
        Geometry {
            volume: arc(41., 7., 64., 1.),
            signal: arc(52., 4., 64., 1.),
            meter: arc(52., 9., 100., 70. / 52.),
            ticks: mask(ticks.finish().unwrap(), 1.3),
            pointers: std::array::from_fn(|n| {
                let angle = std::f32::consts::PI * n as f32 / 100.;
                let mut line = PathBuilder::new();
                line.move_to(64., 91.);
                line.line_to(64. - 31. * angle.cos(), 91. - 31. * angle.sin());
                mask(line.finish().unwrap(), 3.)
            }),
        }
    })
}
fn arc(radius: f32, width: f32, center: f32, stretch: f32) -> Vec<ArcPixel> {
    let mut pixels = Vec::new();
    for y in 35..94 {
        for x in 0..200 {
            let mut thresholds = [255; 16];
            for sy in 0..4 {
                for sx in 0..4 {
                    let dx = (center - (x as f32 + (sx as f32 + 0.5) / 4.)) / stretch;
                    let dy = 91. - (y as f32 + (sy as f32 + 0.5) / 4.);
                    if dy >= 0. && (dx.hypot(dy) - radius).abs() <= width / 2. {
                        thresholds[sy * 4 + sx] = (dy.atan2(dx) / std::f32::consts::PI * 100.)
                            .ceil()
                            .clamp(1., 100.)
                            as u8;
                    }
                }
            }
            if thresholds.iter().any(|n| *n != 255) {
                pixels.push(ArcPixel {
                    offset: (y * 200 + x) * 3,
                    thresholds,
                });
            }
        }
    }
    pixels
}
fn mask(path: tiny_skia::Path, width: f32) -> Vec<Pixel> {
    let mut canvas = Pixmap::new(200, 100).unwrap();
    let mut paint = Paint::default();
    paint.set_color_rgba8(255, 255, 255, 255);
    paint.anti_alias = true;
    canvas.stroke_path(
        &path,
        &paint,
        &Stroke {
            width,
            ..Default::default()
        },
        Transform::identity(),
        None,
    );
    canvas
        .pixels()
        .iter()
        .enumerate()
        .filter_map(|(n, p)| {
            (p.alpha() > 0).then_some(Pixel {
                offset: n * 3,
                alpha: p.alpha(),
            })
        })
        .collect()
}
fn blend(rgb: &mut [u8], offset: usize, color: [u8; 3], alpha: u8) {
    for channel in 0..3 {
        rgb[offset + channel] = ((u16::from(color[channel]) * u16::from(alpha)
            + u16::from(rgb[offset + channel]) * (255 - u16::from(alpha))
            + 127)
            / 255) as u8;
    }
}
fn draw_arc(rgb: &mut [u8], pixels: &[ArcPixel], value: u8, color: [u8; 3]) {
    if value == 0 {
        return;
    }
    for pixel in pixels {
        let covered = pixel.thresholds.iter().filter(|n| **n <= value).count();
        if covered != 0 {
            blend(rgb, pixel.offset, color, ((covered * 255 + 8) / 16) as u8);
        }
    }
}
pub fn draw_base(rgb: &mut [u8], volume: Option<u8>, muted: bool, audio: bool) {
    let shapes = geometry();
    draw_arc(rgb, &shapes.volume, 100, TRACK);
    if audio {
        draw_arc(rgb, &shapes.signal, 100, TRACK);
    }
    for pixel in &shapes.ticks {
        blend(rgb, pixel.offset, [130, 140, 150], pixel.alpha);
    }
    if let Some(value) = volume {
        let value = value.min(100);
        draw_arc(
            rgb,
            &shapes.volume,
            value,
            if muted { [90, 110, 130] } else { BLUE },
        );
        for pixel in &shapes.pointers[usize::from(value)] {
            blend(
                rgb,
                pixel.offset,
                if muted { [120, 130, 140] } else { BLUE },
                pixel.alpha,
            );
        }
    }
}
pub fn draw_signal(rgb: &mut [u8], level: Option<u8>, muted: bool) {
    if muted {
        return;
    }
    if let Some(value) = level {
        let value = value.min(100);
        let color = crate::level_bar::signal_tint(value)
            .map(|color| color.map(|channel| channel as u8))
            .unwrap_or(GREEN);
        draw_arc(rgb, &geometry().signal, value, color);
    } else {
        // An unavailable measurement is not a measured silent signal.
        for y in 76..78 {
            for x in 59..69 {
                rgb[(y * 200 + x) * 3..(y * 200 + x) * 3 + 3].copy_from_slice(&[150, 160, 170]);
            }
        }
    }
}

/// Meter-only panels have one centered signal track, without a volume pointer.
pub fn draw_meter_base(rgb: &mut [u8]) {
    draw_arc(rgb, &geometry().meter, 100, TRACK);
}
pub fn draw_meter_signal(rgb: &mut [u8], level: Option<u8>, muted: bool) {
    if muted {
        return;
    }
    if let Some(value) = level {
        let color = crate::level_bar::signal_tint(value.min(100))
            .map(|color| color.map(|channel| channel as u8))
            .unwrap_or(GREEN);
        draw_arc(rgb, &geometry().meter, value.min(100), color);
    } else {
        // A missing measurement must remain distinct from measured silence.
        for y in 85..87 {
            for x in 95..105 {
                rgb[(y * 200 + x) * 3..(y * 200 + x) * 3 + 3].copy_from_slice(&[150, 160, 170]);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn all_values_are_bounded_and_mute_suppresses_live_signal() {
        let background = [30, 34, 39].repeat(200 * 100);
        for value in 0..=100 {
            let mut image = background.clone();
            draw_base(&mut image, Some(value), false, true);
            let base = image.clone();
            draw_signal(&mut image, Some(value), true);
            assert_eq!(image, base);
            draw_signal(&mut image, Some(value), false);
            assert_eq!(&image[..35 * 600], &background[..35 * 600]);
            assert_eq!(&image[94 * 600..], &background[94 * 600..]);
            for row in image.chunks_exact(600).zip(background.chunks_exact(600)) {
                assert_eq!(&row.0[119 * 3..], &row.1[119 * 3..]);
            }
        }
        let mut missing = background.clone();
        draw_base(&mut missing, None, false, true);
        let mut silent = missing.clone();
        draw_signal(&mut missing, None, false);
        draw_signal(&mut silent, Some(0), false);
        assert_ne!(missing, silent);
    }
    #[test]
    fn live_signal_changes_only_the_outer_arc() {
        let mut low = [30, 34, 39].repeat(200 * 100);
        draw_base(&mut low, Some(68), false, true);
        let mut high = low.clone();
        draw_signal(&mut low, Some(20), false);
        draw_signal(&mut high, Some(80), false);
        assert_ne!(low, high);
        for y in 0..100 {
            for x in 0..200 {
                let radius = (x as f32 + 0.5 - 64.).hypot(y as f32 + 0.5 - 91.);
                if !(49. ..=55.).contains(&radius) {
                    let offset = (y * 200 + x) * 3;
                    assert_eq!(&low[offset..offset + 3], &high[offset..offset + 3]);
                }
            }
        }
    }
    #[test]
    fn single_meter_preserves_title_status_and_mute_at_every_level() {
        let background = [30, 34, 39].repeat(200 * 100);
        let mut base = background.clone();
        draw_meter_base(&mut base);
        let mut missing = base.clone();
        draw_meter_signal(&mut missing, None, false);
        assert_ne!(
            missing, base,
            "missing measurement differs from silent signal"
        );
        for value in 0..=100 {
            let mut muted = base.clone();
            draw_meter_signal(&mut muted, Some(value), true);
            assert_eq!(muted, base);
            let mut live = base.clone();
            draw_meter_signal(&mut live, Some(value), false);
            assert_eq!(&live[..35 * 600], &background[..35 * 600]);
            assert_eq!(&live[94 * 600..], &background[94 * 600..]);
            for y in 60..92 {
                assert_eq!(
                    &live[(y * 200 + 52) * 3..(y * 200 + 148) * 3],
                    &base[(y * 200 + 52) * 3..(y * 200 + 148) * 3],
                    "live arc must leave the centered status clear"
                );
            }
        }
    }
}
