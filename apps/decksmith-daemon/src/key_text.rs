//! Smooth key captions using bundled, unmodified OFL font outlines.
#![allow(dead_code)] // Legacy drawing helpers are also used by standalone examples.
#[derive(Clone, Copy)]
pub struct Typography {
    pub font: u8,
    pub scale: f32,
}
impl Default for Typography {
    fn default() -> Self {
        Self { font: 0, scale: 1. }
    }
}
use skrifa::{
    FontRef, MetadataProvider,
    instance::Size,
    outline::{DrawSettings, OutlinePen},
};
use tiny_skia::{FillRule, Paint, PathBuilder, Pixmap, Transform};
struct Pen {
    path: PathBuilder,
    x: f32,
    y: f32,
}
impl OutlinePen for Pen {
    fn move_to(&mut self, x: f32, y: f32) {
        self.path.move_to(self.x + x, self.y - y);
    }
    fn line_to(&mut self, x: f32, y: f32) {
        self.path.line_to(self.x + x, self.y - y);
    }
    fn quad_to(&mut self, cx: f32, cy: f32, x: f32, y: f32) {
        self.path
            .quad_to(self.x + cx, self.y - cy, self.x + x, self.y - y);
    }
    fn curve_to(&mut self, cx: f32, cy: f32, dx: f32, dy: f32, x: f32, y: f32) {
        self.path.cubic_to(
            self.x + cx,
            self.y - cy,
            self.x + dx,
            self.y - dy,
            self.x + x,
            self.y - y,
        );
    }
    fn close(&mut self) {
        self.path.close();
    }
}
pub fn draw(rgb: &mut [u8], label: &str, anchor: u8, artwork: bool, backing: bool, color: [u8; 3]) {
    draw_styled(
        rgb,
        label,
        anchor,
        artwork,
        backing,
        color,
        Typography::default(),
    );
}
pub fn draw_styled(
    rgb: &mut [u8],
    label: &str,
    anchor: u8,
    artwork: bool,
    backing: bool,
    color: [u8; 3],
    typography: Typography,
) {
    draw_canvas(
        rgb,
        label,
        color,
        Canvas {
            left_aligned: false,
            width: 120,
            height: 120,
            anchor,
            size: (if artwork { 24.0 } else { 34.0 }) * typography.scale,
            font: typography.font,
            backing: artwork && backing,
        },
    );
}

pub fn draw_strip(rgb: &mut [u8], label: &str, anchor: u8, color: [u8; 3]) {
    draw_strip_styled(rgb, label, anchor, color, Typography::default());
}
pub fn draw_strip_styled(
    rgb: &mut [u8],
    label: &str,
    anchor: u8,
    color: [u8; 3],
    typography: Typography,
) {
    draw_canvas(
        rgb,
        label,
        color,
        Canvas {
            left_aligned: false,
            width: 200,
            height: 100,
            anchor,
            size: 24.0 * typography.scale,
            font: typography.font,
            backing: false,
        },
    );
}

/// Start titles at x=44, leaving a consistent gap after the 36-pixel icon area.
#[allow(dead_code)] // The key-only caption example does not use audio titles.
pub fn draw_strip_audio_title(rgb: &mut [u8], label: &str) {
    draw_strip_title_styled(rgb, label, [240, 230, 215], Typography::default());
}
pub fn draw_strip_title_styled(
    rgb: &mut [u8],
    label: &str,
    color: [u8; 3],
    typography: Typography,
) {
    let mut title = Vec::with_capacity(164 * 100 * 3);
    for y in 0..100 {
        title.extend_from_slice(&rgb[(y * 200 + 36) * 3..(y * 200 + 200) * 3]);
    }
    draw_canvas(
        &mut title,
        label,
        color,
        Canvas {
            left_aligned: true,
            width: 164,
            height: 100,
            anchor: 0,
            size: 24. * typography.scale,
            font: typography.font,
            backing: false,
        },
    );
    for y in 0..34 {
        rgb[(y * 200 + 36) * 3..(y * 200 + 200) * 3]
            .copy_from_slice(&title[y * 164 * 3..(y + 1) * 164 * 3]);
    }
}

struct Canvas {
    left_aligned: bool,
    width: usize,
    height: usize,
    anchor: u8,
    size: f32,
    backing: bool,
    font: u8,
}
fn draw_canvas(rgb: &mut [u8], label: &str, color: [u8; 3], canvas: Canvas) {
    let Canvas {
        left_aligned,
        width,
        height: canvas_height,
        anchor,
        size: initial_size,
        backing,
        font: font_kind,
    } = canvas;
    let converted;
    let label = if font_kind == 13 {
        converted = decorative_runes(label);
        converted.as_str()
    } else {
        label
    };
    let bytes = font_bytes(font_kind);
    let font = FontRef::new(bytes).expect("valid bundled font");
    let location = font.axes().location([("wght", 700.0)]);
    let outlines = font.outline_glyphs();
    let charmap = font.charmap();
    let measure = |text: &str| {
        let metrics = font.glyph_metrics(Size::new(initial_size), &location);
        text.chars()
            .filter_map(|c| charmap.map(c))
            .map(|id| metrics.advance_width(id).unwrap_or(0.0))
            .sum::<f32>()
    };
    let split = if width == 120 && measure(label) > (width - 16) as f32 {
        label
            .match_indices(' ')
            .filter(|(i, _)| !label[..*i].trim().is_empty() && !label[*i + 1..].trim().is_empty())
            .min_by(|(a, _), (b, _)| {
                measure(label[..*a].trim())
                    .max(measure(label[*a + 1..].trim()))
                    .total_cmp(&measure(label[..*b].trim()).max(measure(label[*b + 1..].trim())))
            })
            .map(|(i, _)| i)
    } else {
        None
    };
    let lines = match split {
        Some(i) => vec![label[..i].trim(), label[i + 1..].trim()],
        None => vec![label],
    };
    let mut size = initial_size;
    let path = loop {
        let mut pen = Pen {
            path: PathBuilder::new(),
            x: 0.0,
            y: 0.0,
        };
        let metrics = font.glyph_metrics(Size::new(size), &location);
        for (line_index, line) in lines.iter().enumerate() {
            let advance = line
                .chars()
                .filter_map(|c| charmap.map(c))
                .map(|id| metrics.advance_width(id).unwrap_or(0.0))
                .sum::<f32>();
            pen.x = -advance / 2.0;
            pen.y = line_index as f32 * size * 1.12;
            for c in line.chars() {
                if let Some(id) = charmap.map(c) {
                    if let Some(glyph) = outlines.get(id) {
                        glyph
                            .draw(DrawSettings::unhinted(Size::new(size), &location), &mut pen)
                            .expect("valid bundled outline");
                    }
                    pen.x += metrics.advance_width(id).unwrap_or(0.0);
                }
            }
        }
        let Some(path) = pen.path.finish() else {
            return;
        };
        if path.bounds().width() <= (width - 16) as f32
            && path.bounds().height() <= if lines.len() > 1 { 80.0 } else { 40.0 }
        {
            break path;
        }
        size -= 0.5;
        if size < 3.0 {
            return;
        }
    };
    let bounds = path.bounds();
    let height = bounds.height();
    let left = if left_aligned {
        8.0
    } else {
        (width as f32 - bounds.width()) / 2.0
    };
    let top = match anchor {
        0 => 8.0,
        2 => canvas_height as f32 - 8.0 - height,
        _ => (canvas_height as f32 - height) / 2.0,
    };
    if backing {
        for y in (top as usize).saturating_sub(3)
            ..((top + height).ceil() as usize + 3).min(canvas_height)
        {
            for x in 4..width - 4 {
                for channel in &mut rgb[(y * width + x) * 3..(y * width + x) * 3 + 3] {
                    *channel /= 3;
                }
            }
        }
    }
    let mut pixels = Pixmap::new(width as u32, canvas_height as u32).unwrap();
    let mut paint = Paint::default();
    paint.set_color_rgba8(color[0], color[1], color[2], 255);
    paint.anti_alias = true;
    pixels.fill_path(
        &path,
        &paint,
        FillRule::Winding,
        Transform::from_translate(left - bounds.left(), top - bounds.top()),
        None,
    );
    for (index, pixel) in pixels.pixels().iter().enumerate() {
        let alpha = u16::from(pixel.alpha());
        for (channel, source) in [pixel.red(), pixel.green(), pixel.blue()]
            .into_iter()
            .enumerate()
        {
            rgb[index * 3 + channel] = (u16::from(source)
                + (u16::from(rgb[index * 3 + channel]) * (255 - alpha) + 127) / 255)
                .min(255) as u8;
        }
    }
}

fn font_bytes(kind: u8) -> &'static [u8] {
    match kind {
        1 => include_bytes!("../../../assets/fonts/LiberationSerif-Bold.ttf"),
        2 => include_bytes!("../../../assets/fonts/LiberationMono-Bold.ttf"),
        3 => include_bytes!("../../../assets/fonts/Roboto[wdth,wght].ttf"),
        4 => include_bytes!("../../../assets/fonts/OpenSans[wdth,wght].ttf"),
        5 => include_bytes!("../../../assets/fonts/Lato-Bold.ttf"),
        6 => include_bytes!("../../../assets/fonts/Montserrat[wght].ttf"),
        7 => include_bytes!("../../../assets/fonts/Oswald[wght].ttf"),
        8 => include_bytes!("../../../assets/fonts/Raleway[wght].ttf"),
        9 => include_bytes!("../../../assets/fonts/Poppins-Bold.ttf"),
        10 => include_bytes!("../../../assets/fonts/Nunito[wght].ttf"),
        11 => include_bytes!("../../../assets/fonts/Merriweather[opsz,wdth,wght].ttf"),
        12 => include_bytes!("../../../assets/fonts/SourceSans3[wght].ttf"),
        13 => include_bytes!("../../../assets/fonts/NotoSansRunic-Regular.ttf"),
        _ => include_bytes!("../../../assets/fonts/LiberationSans-Bold.ttf"),
    }
}

/// Decorative Latin-to-Younger-Futhark approximation, not a language translation.
/// The saved label remains unchanged; only rendered text is substituted.
fn decorative_runes(text: &str) -> String {
    let mut out = String::new();
    for c in text.chars() {
        let rune = match c.to_ascii_uppercase() {
            'A' => "ᛅ",
            'B' | 'P' => "ᛒ",
            'C' | 'G' | 'K' | 'Q' => "ᚴ",
            'D' | 'T' => "ᛏ",
            'E' | 'I' | 'J' => "ᛁ",
            'F' | 'V' => "ᚠ",
            'H' => "ᚼ",
            'L' => "ᛚ",
            'M' => "ᛘ",
            'N' => "ᚾ",
            'O' => "ᚬ",
            'R' => "ᚱ",
            'S' | 'Z' => "ᛋ",
            'U' | 'W' => "ᚢ",
            'X' => "ᚴᛋ",
            'Y' => "ᛦ",
            _ => {
                out.push(c);
                continue;
            }
        };
        out.push_str(rune);
    }
    out
}
#[cfg(test)]
mod font_tests {
    use super::*;
    #[test]
    fn every_font_renders_and_runic_retains_numbers() {
        for kind in 0..14 {
            let font = FontRef::new(font_bytes(kind)).unwrap();
            let text = if kind == 13 {
                decorative_runes("ABCDEFGHIJKLMNOPQRSTUVWXYZ 0123456789")
            } else {
                "Volume 75".into()
            };
            for c in text.chars() {
                assert!(font.charmap().map(c).is_some(), "font {kind}, missing {c}");
            }
            let mut pixels = vec![0; 120 * 120 * 3];
            draw_styled(
                &mut pixels,
                "Volume 75",
                1,
                false,
                false,
                [255, 255, 255],
                Typography {
                    font: kind,
                    scale: 1.,
                },
            );
            assert!(pixels.iter().any(|v| *v != 0), "font {kind}");
        }
    }
    #[test]
    fn runes_are_case_insensitive_and_keep_original_label_untouched() {
        let label = "Viking 42";
        assert_eq!(decorative_runes(label), "ᚠᛁᚴᛁᚾᚴ 42");
        assert_eq!(decorative_runes(label), decorative_runes("VIKING 42"));
        assert_eq!(label, "Viking 42");
    }
}
