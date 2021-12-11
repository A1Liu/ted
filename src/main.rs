use editor::fonts::GlyphCache;
use image::{ColorType, ImageFormat};

fn main() {
    let mut cache = GlyphCache::new();

    let result = cache.all_glyphs();
    let glyphs = result.glyphs;

    let (w, h) = {
        let (mut width, mut height) = (0, 0);
        for glyph in &glyphs {
            if width < glyph.width {
                width = glyph.width;
            }

            if height < glyph.height {
                height = glyph.height;
            }
        }

        (width + 5, height + 10)
    };
    let count = glyphs.len();
    let mut buffer = Vec::with_capacity(count * w * h);

    let g_columns = 100;
    let g_rows = (count - 1) / g_columns + 1;

    for g_row in 0..g_rows {
        let g_begin = g_row * g_columns;
        let g_end = g_begin + g_columns;
        let upper = core::cmp::min(g_end, glyphs.len());

        for row in 0..h {
            for glyph in &glyphs[g_begin..upper] {
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

            for _ in upper..g_end {
                for _ in 0..w {
                    buffer.push(0);
                }
            }
        }
    }

    let (w, h) = (w as u32, h as u32);
    let w = w * (g_columns as u32);
    let h = h * (g_rows as u32);
    let path = "output.png";
    image::save_buffer_with_format(path, &buffer, w, h, ColorType::L8, ImageFormat::Png).unwrap();
}
