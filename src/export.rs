use image::ColorType;
use image::codecs::jpeg::JpegEncoder;
use resvg::tiny_skia::{Color, Pixmap, Transform};
use resvg::usvg::{Options, Tree};

const JPEG_QUALITY: u8 = 95;
const EXPORT_SCALE: f32 = 2.0;
const EXPORT_BACKGROUND: [u8; 4] = [255, 255, 255, 255];
const EXPORT_FONT: &[u8] = include_bytes!("../fonts/Roboto-Regular.ttf");

pub async fn save_svg_as_jpeg(svg_bytes: Vec<u8>) -> Result<(), String> {
    let Some(file) = rfd::AsyncFileDialog::new()
        .set_title("Сохранить карту ЭЭГ")
        .set_file_name("eeg-map.jpg")
        .add_filter("JPEG", &["jpg", "jpeg"])
        .save_file()
        .await
    else {
        return Ok(());
    };

    let mut path = file.path().to_path_buf();
    let has_jpeg_extension = path
        .extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| {
            extension.eq_ignore_ascii_case("jpg") || extension.eq_ignore_ascii_case("jpeg")
        });

    if !has_jpeg_extension {
        path.set_extension("jpg");
    }

    let jpeg_bytes = render_svg_as_jpeg(&svg_bytes)?;
    std::fs::write(&path, jpeg_bytes)
        .map_err(|error| format!("Не удалось сохранить {}: {error}", path.display()))
}

fn render_svg_as_jpeg(svg_bytes: &[u8]) -> Result<Vec<u8>, String> {
    let tree = parse_svg_tree(svg_bytes)?;
    let width = (tree.size().width() * EXPORT_SCALE).ceil() as u32;
    let height = (tree.size().height() * EXPORT_SCALE).ceil() as u32;
    let mut pixmap = Pixmap::new(width, height)
        .ok_or_else(|| "Не удалось создать изображение нужного размера".to_owned())?;

    pixmap.fill(Color::from_rgba8(
        EXPORT_BACKGROUND[0],
        EXPORT_BACKGROUND[1],
        EXPORT_BACKGROUND[2],
        EXPORT_BACKGROUND[3],
    ));

    {
        let mut pixmap_mut = pixmap.as_mut();
        resvg::render(
            &tree,
            Transform::from_scale(EXPORT_SCALE, EXPORT_SCALE),
            &mut pixmap_mut,
        );
    }

    let mut rgb = Vec::with_capacity(width as usize * height as usize * 3);
    for rgba in pixmap.data().chunks_exact(4) {
        rgb.extend_from_slice(&rgba[..3]);
    }

    let mut jpeg_bytes = Vec::new();
    JpegEncoder::new_with_quality(&mut jpeg_bytes, JPEG_QUALITY)
        .encode(&rgb, width, height, ColorType::Rgb8.into())
        .map_err(|error| format!("Не удалось закодировать JPEG: {error}"))?;

    Ok(jpeg_bytes)
}

fn parse_svg_tree(svg_bytes: &[u8]) -> Result<Tree, String> {
    let mut options = Options {
        font_family: "Roboto".to_owned(),
        ..Options::default()
    };
    options.fontdb_mut().load_font_data(EXPORT_FONT.to_vec());

    Tree::from_data(svg_bytes, &options)
        .map_err(|error| format!("Не удалось подготовить SVG: {error}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_svg_as_jpeg() {
        let svg = include_bytes!("../images/map.svg");
        let jpeg = render_svg_as_jpeg(svg).unwrap();
        let rendered = image::load_from_memory_with_format(&jpeg, image::ImageFormat::Jpeg)
            .unwrap()
            .to_rgb8();

        assert!(jpeg.starts_with(&[0xff, 0xd8, 0xff]));
        assert!(jpeg.ends_with(&[0xff, 0xd9]));
        assert!(
            rendered
                .get_pixel(0, 0)
                .0
                .iter()
                .all(|channel| *channel >= 250)
        );
    }

    #[test]
    fn renders_embedded_font() {
        let svg = br#"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="60"><text x="10" y="40" font-family="Roboto" font-size="32">Cz</text></svg>"#;
        let jpeg = render_svg_as_jpeg(svg).unwrap();
        let rendered = image::load_from_memory_with_format(&jpeg, image::ImageFormat::Jpeg)
            .unwrap()
            .to_rgb8();

        assert!(
            rendered
                .pixels()
                .any(|pixel| pixel.0.iter().any(|channel| *channel < 128))
        );
    }
}
