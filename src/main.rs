use editor::fonts::GlyphCache;
use image::{ColorType, ImageFormat};

fn main() {
    let mut cache = GlyphCache::new();

    let text = "Hello World!";
    let result = cache.translate_glyphs(text);
    let glyphs = result.glyphs;
    let (mut width, mut height) = (0, 0);
    for glyph in &glyphs {
        if width < glyph.width {
            width = glyph.width;
        }

        if height < glyph.height {
            height = glyph.height;
        }
    }

    let (w, h) = (width, height);
    let count = glyphs.len();
    let mut buffer = Vec::with_capacity(count * w * h);
    let size = w * h;

    for row in 0..h {
        for glyph in &glyphs {
            let height_diff = h - glyph.height;

            if row < height_diff {
                for _ in 0..w {
                    buffer.push(0);
                }

                continue;
            }

            let width = glyph.width;
            let offset = glyph.offset + (row - height_diff) * width;
            buffer.extend_from_slice(&cache.data[offset..(offset + width)]);
            for _ in 0..w - width {
                buffer.push(0);
            }
        }
    }

    let (w, h, count) = (w as u32, h as u32, count as u32);
    let w = w * count;
    let path = "output.png";
    image::save_buffer_with_format(path, &buffer, w, h, ColorType::L8, ImageFormat::Png).unwrap();
}
