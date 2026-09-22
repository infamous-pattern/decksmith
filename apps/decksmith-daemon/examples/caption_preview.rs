#[path = "../src/key_text.rs"]
mod key_text;
#[path = "../src/level_bar.rs"]
mod level_bar;
fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut image = image::open(&args[1]).unwrap().to_rgb8();
    assert_eq!(image.dimensions(), (120, 120));
    key_text::draw(image.as_mut(), "Volume Up", 1, true, false, [255, 225, 60]);
    image.save(&args[2]).unwrap();
    if let Some(output) = args.get(3) {
        let mut strip = image::RgbImage::new(800, 100);
        for (i, label) in ["Volume", "Display Brightness", "Brightness", "Output"]
            .iter()
            .enumerate()
        {
            let mut panel = [30, 34, 39].repeat(200 * 100);
            key_text::draw_strip(&mut panel, label, 0, [240, 230, 215]);
            key_text::draw_strip(
                &mut panel,
                &format!("{}%", [0, 45, 68, 100][i]),
                1,
                [90, 170, 255],
            );
            level_bar::draw(&mut panel, Some([0, 45, 68, 100][i]), false);
            let panel = image::RgbImage::from_raw(200, 100, panel).unwrap();
            image::imageops::replace(&mut strip, &panel, (i * 200) as i64, 0);
        }
        strip.save(output).unwrap();
    }
}
