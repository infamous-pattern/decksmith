//! Small original vector symbols for audio target categories, shared by every renderer.
use serde::{Deserialize, Serialize};
use tiny_skia::{Paint, PathBuilder, Pixmap, Stroke, Transform};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Kind {
    #[default]
    Unknown,
    Speaker,
    Headphones,
    Microphone,
    Application,
    Brightness,
}

#[allow(dead_code)] // Legacy palette remains available for fixtures.
pub fn draw(rgb: &mut [u8], kind: Kind, muted: bool, available: bool) {
    draw_styled(
        rgb,
        kind,
        muted,
        available,
        [224, 234, 240],
        [30, 34, 39],
        None,
    );
}
#[allow(clippy::too_many_arguments)]
pub fn draw_styled(
    rgb: &mut [u8],
    kind: Kind,
    muted: bool,
    available: bool,
    foreground: [u8; 3],
    background: [u8; 3],
    application_rgba: Option<&[u8]>,
) {
    let mut pixels = Pixmap::new(200, 100).unwrap();
    let mut path = PathBuilder::new();
    match kind {
        Kind::Brightness => {
            path.move_to(14., 24.);
            path.cubic_to(14., 20., 9., 18., 9., 12.);
            path.cubic_to(9., 0., 29., 0., 29., 12.);
            path.cubic_to(29., 18., 24., 20., 24., 24.);
            path.close();
            path.move_to(14., 28.);
            path.line_to(24., 28.);
            path.move_to(17., 32.);
            path.line_to(21., 32.);
            path.move_to(19., 23.);
            path.line_to(19., 15.);
            path.move_to(15., 12.);
            path.line_to(19., 16.);
            path.line_to(23., 12.);
        }

        Kind::Microphone => {
            path.move_to(15., 8.);
            path.cubic_to(15., 3., 23., 3., 23., 8.);
            path.line_to(23., 17.);
            path.cubic_to(23., 22., 15., 22., 15., 17.);
            path.close();
            path.move_to(11., 16.);
            path.cubic_to(11., 28., 27., 28., 27., 16.);
            path.move_to(19., 25.);
            path.line_to(19., 30.);
            path.move_to(14., 30.);
            path.line_to(24., 30.);
        }
        Kind::Headphones => {
            path.move_to(8., 24.);
            path.line_to(8., 15.);
            path.cubic_to(8., 2., 30., 2., 30., 15.);
            path.line_to(30., 24.);
            path.push_rect(tiny_skia::Rect::from_xywh(8., 17., 5., 11.).unwrap());
            path.push_rect(tiny_skia::Rect::from_xywh(25., 17., 5., 11.).unwrap());
        }
        Kind::Speaker => {
            path.move_to(8., 13.);
            path.line_to(14., 13.);
            path.line_to(21., 7.);
            path.line_to(21., 29.);
            path.line_to(14., 23.);
            path.line_to(8., 23.);
            path.close();
            path.move_to(25., 12.);
            path.cubic_to(29., 15., 29., 21., 25., 24.);
            path.move_to(29., 7.);
            path.cubic_to(36., 13., 36., 23., 29., 29.);
        }
        Kind::Application => {
            path.push_rect(tiny_skia::Rect::from_xywh(7., 6., 25., 23.).unwrap());
            path.move_to(7., 12.);
            path.line_to(32., 12.);
            path.move_to(12., 9.);
            path.line_to(14., 9.);
            path.move_to(18., 17.);
            path.line_to(24., 21.);
            path.line_to(18., 25.);
            path.close();
        }
        Kind::Unknown => {
            // Neutral audio waveform: do not guess a physical transducer from a name.
            for (x, top, bottom) in [
                (8., 15., 23.),
                (14., 9., 29.),
                (20., 5., 32.),
                (26., 11., 27.),
                (32., 16., 22.),
            ] {
                path.move_to(x, top);
                path.line_to(x, bottom);
            }
        }
    }
    let mut paint = Paint::default();
    let c = if available {
        foreground
    } else {
        [125, 135, 145]
    };
    paint.set_color_rgba8(c[0], c[1], c[2], 255);
    paint.anti_alias = true;
    let stroke = Stroke {
        width: 2.,
        line_cap: tiny_skia::LineCap::Round,
        line_join: tiny_skia::LineJoin::Round,
        ..Default::default()
    };
    if let Some(rgba) = application_rgba.filter(|v| v.len() == 32 * 32 * 4) {
        for (index, color) in rgba.chunks_exact(4).enumerate() {
            let alpha = u16::from(color[3]);
            pixels.pixels_mut()[(index / 32 + 2) * 200 + index % 32 + 3] =
                tiny_skia::PremultipliedColorU8::from_rgba(
                    (u16::from(color[0]) * alpha / 255) as u8,
                    (u16::from(color[1]) * alpha / 255) as u8,
                    (u16::from(color[2]) * alpha / 255) as u8,
                    color[3],
                )
                .unwrap();
        }
    } else {
        pixels.stroke_path(
            &path.finish().unwrap(),
            &paint,
            &stroke,
            Transform::identity(),
            None,
        );
    }
    if muted {
        let mut slash = PathBuilder::new();
        slash.move_to(6., 31.);
        slash.line_to(33., 5.);
        let slash = slash.finish().unwrap();
        paint.set_color_rgba8(background[0], background[1], background[2], 255);
        pixels.stroke_path(
            &slash,
            &paint,
            &Stroke {
                width: 5.,
                ..stroke.clone()
            },
            Transform::identity(),
            None,
        );
        paint.set_color_rgba8(255, 75, 85, 255);
        pixels.stroke_path(
            &slash,
            &paint,
            &Stroke {
                width: 2.8,
                ..stroke
            },
            Transform::identity(),
            None,
        );
    }
    for (i, p) in pixels.pixels().iter().enumerate() {
        let a = u16::from(p.alpha());
        for (channel, value) in [p.red(), p.green(), p.blue()].into_iter().enumerate() {
            rgb[i * 3 + channel] = (u16::from(value)
                + (u16::from(rgb[i * 3 + channel]) * (255 - a) + 127) / 255)
                .min(255) as u8;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn symbols_are_distinct_and_mute_is_reversible() {
        let mut previous = Vec::new();
        for kind in [
            Kind::Unknown,
            Kind::Speaker,
            Kind::Headphones,
            Kind::Microphone,
            Kind::Application,
            Kind::Brightness,
        ] {
            let mut normal = [30, 34, 39].repeat(200 * 100);
            draw(&mut normal, kind, false, true);
            assert!(!previous.contains(&normal));
            let mut muted = [30, 34, 39].repeat(200 * 100);
            draw(&mut muted, kind, true, true);
            assert_ne!(normal, muted);
            assert!(muted.chunks_exact(3).any(|p| p == [255, 75, 85]));
            assert_eq!(
                &muted[(50 * 200 + 1) * 3..(50 * 200 + 1) * 3 + 3],
                &[30, 34, 39]
            );
            assert_eq!(
                &normal[(50 * 200 + 1) * 3..(50 * 200 + 1) * 3 + 3],
                &[30, 34, 39]
            );
            let mut restored = [30, 34, 39].repeat(200 * 100);
            draw(&mut restored, kind, false, true);
            assert_eq!(normal, restored);
            previous.push(normal);
        }
    }
}
