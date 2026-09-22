//! Render an offline contact sheet using the same caption code as the device.
#[path = "../src/key_text.rs"]
mod key_text;
fn main() {
    let names = [
        "Sans",
        "Serif",
        "Monospace",
        "Roboto",
        "Open Sans",
        "Lato",
        "Montserrat",
        "Oswald",
        "Raleway",
        "Poppins",
        "Nunito",
        "Merriweather",
        "Source Sans 3",
        "Viking Runes",
    ];
    let mut sheet = image::RgbImage::new(800, 800);
    for (kind, name) in names.iter().enumerate() {
        let mut title = vec![26; 200 * 100 * 3];
        key_text::draw_strip_styled(
            &mut title,
            name,
            0,
            [200, 200, 200],
            key_text::Typography {
                font: 0,
                scale: 0.75,
            },
        );
        let title = image::RgbImage::from_raw(200, 100, title).unwrap();
        let mut key = vec![26; 120 * 120 * 3];
        key_text::draw_styled(
            &mut key,
            if kind == 13 { "Viking 42" } else { "Volume Up" },
            1,
            false,
            false,
            [245, 220, 170],
            key_text::Typography {
                font: kind as u8,
                scale: 1.,
            },
        );
        let key = image::RgbImage::from_raw(120, 120, key).unwrap();
        image::imageops::replace(
            &mut sheet,
            &title,
            (kind % 4 * 200) as i64,
            (kind / 4 * 200) as i64,
        );
        image::imageops::replace(
            &mut sheet,
            &key,
            (kind % 4 * 200 + 40) as i64,
            (kind / 4 * 200 + 50) as i64,
        );
    }
    sheet
        .save(std::env::args().nth(1).expect("output PNG path"))
        .unwrap();
}
