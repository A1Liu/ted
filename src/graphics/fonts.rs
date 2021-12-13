use crate::util::*;
use font_rs::geom::{affine_pt, Affine, Point};
use font_rs::raster::Raster;
use std::collections::hash_map::HashMap;
use ttf_parser as ttf;

// const COURIER: &[u8] = &[0];
const MAX_ATLAS_WIDTH: u32 = 4096;
const COURIER: &[u8] = core::include_bytes!("./cour.ttf");

// These affect how the font looks I think? I'm not really sure tbh.
//                                  - Albert Liu, Dec 11, 2021 Sat 22:44 EST
const SIZE: usize = 128; // some kind of font thing idk.
const PAD_L: u32 = 8; // in pixels or something?
const PAD_R: u32 = 4; // in pixels
const PAD_T: u32 = 4; // in pixels
const PAD_B: u32 = 8; // in pixels

const DEFAULT_CHARS: &'static str = core::concat!(
    " ABCDEFGHIJKLMNOPQRSTUVWXYZ",
    "abcdefghijklmnopqrstuvwxyz",
    "0123456789",
    r#"`~!@#$%^&*()_+-=[]{};':",.<>/?\|"#
);

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct Glyph {
    pub x: u32,
    pub y: u32,
}

pub struct GlyphData {
    pub data: Vec<u8>,
    pub width: u32,
    pub height: u32,
}

pub struct GlyphCache {
    descriptors: HashMap<char, Glyph>,
    atlas: Vec<u8>,
    glyph_width: u32,
    glyph_height: u32,
    atlas_current_row_width: u32,
    atlas_width: u32,
    atlas_height: u32,
}

pub struct GlyphList {
    pub did_raster: bool,
    pub glyphs: Vec<Glyph>,
}

impl GlyphList {
    pub fn new(did_raster: bool, glyphs: Vec<Glyph>) -> Self {
        return Self { did_raster, glyphs };
    }
}

impl GlyphCache {
    pub fn new() -> GlyphCache {
        return GlyphCache {
            descriptors: HashMap::new(),
            atlas: Vec::new(),
            glyph_width: 0,
            glyph_height: 0,
            atlas_current_row_width: MAX_ATLAS_WIDTH,
            atlas_width: MAX_ATLAS_WIDTH,
            atlas_height: 0,
        };
    }

    pub fn atlas_dims(&self) -> Rect {
        return Rect::new(self.atlas_width, self.atlas_height);
    }

    pub fn atlas(&self) -> &[u8] {
        return &self.atlas;
    }

    pub fn translate_glyphs(&mut self, characters: &str) -> GlyphList {
        let mut chars = characters.chars();
        let mut glyphs = Vec::new();

        let char_count = characters.len();
        glyphs.reserve(char_count * 6); // each glyph is 2 trianges of 3 points each

        let character;
        'fast_path: loop {
            while let Some(c) = chars.next() {
                if let Some(&glyph) = self.descriptors.get(&c) {
                    self.add_glyph_to_list(&mut glyphs, glyph);
                    continue;
                }

                character = c;
                break 'fast_path;
            }

            return GlyphList::new(false, glyphs);
        }

        let face = ttf::Face::from_slice(COURIER, 0).unwrap();
        if face.is_variable() || !face.is_monospaced() {
            panic!("Can't handle variable fonts");
        }

        let ppem = face.units_per_em();
        let scale = (SIZE as f32) / (ppem as f32);

        let (mut width, mut height) = (0, 0);
        for c in characters.chars().chain(DEFAULT_CHARS.chars()) {
            let (w, h) = char_dimensions(&face, scale, c);

            width = core::cmp::max(width, w);
            height = core::cmp::max(height, h);
        }

        let (width, height) = (width + PAD_L + PAD_R, height + PAD_T + PAD_B);

        if width < self.glyph_width && height < self.glyph_height {
            let glyph = self.add_char(&face, scale, character);
            self.add_glyph_to_list(&mut glyphs, glyph);

            while let Some(c) = chars.next() {
                let glyph = self.add_char(&face, scale, c);
                self.add_glyph_to_list(&mut glyphs, glyph);
            }

            return GlyphList::new(true, glyphs);
        }

        let chars = ();
        let character = ();

        glyphs.clear();
        self.descriptors.clear();
        self.atlas.clear();

        self.glyph_width = width;
        self.glyph_height = height;
        self.atlas_width = MAX_ATLAS_WIDTH / self.glyph_width * self.glyph_width;
        self.atlas_current_row_width = self.atlas_width;
        self.atlas_height = 0;

        for c in DEFAULT_CHARS.chars() {
            self.add_char(&face, scale, c);
        }

        for c in characters.chars() {
            let glyph = self.add_char(&face, scale, c);
            self.add_glyph_to_list(&mut glyphs, glyph);
        }

        return GlyphList::new(true, glyphs);
    }

    fn add_glyph_to_list(&self, list: &mut Vec<Glyph>, mut glyph: Glyph) {
        let top_left = glyph;

        glyph.x += self.glyph_width;
        let top_right = glyph;

        glyph.y += self.glyph_height;
        let bot_right = glyph;

        glyph.x -= self.glyph_width;
        let bot_left = glyph;

        list.push(top_left);
        list.push(top_right);
        list.push(bot_left);

        list.push(bot_left);
        list.push(top_right);
        list.push(bot_right);
    }

    fn add_char(&mut self, face: &ttf::Face, scale: f32, c: char) -> Glyph {
        if let Some(&glyph) = self.descriptors.get(&c) {
            if (self.atlas_height * self.atlas_width) != (self.atlas.len() as u32) {
                panic!("atlas is in invalid state");
            }

            return glyph;
        }

        if self.atlas_current_row_width + self.glyph_width >= self.atlas_width {
            let glyph_row_size = self.atlas_width * self.glyph_height;

            self.atlas.reserve(glyph_row_size as usize);
            for _ in 0..glyph_row_size {
                self.atlas.push(0);
            }

            self.atlas_current_row_width = 0;
            self.atlas_height += self.glyph_height;
        }

        let glyph_id = face.glyph_index(c).unwrap();
        let glyph_data = rasterize_glyph(face, scale, glyph_id);

        let x = self.atlas_current_row_width;
        self.atlas_current_row_width += self.glyph_width;
        let y = self.atlas_height - self.glyph_height;

        let glyph = Glyph { x, y };
        self.write_glyph_data(glyph, glyph_data);
        self.descriptors.insert(c, glyph);

        if (self.atlas_height * self.atlas_width) != (self.atlas.len() as u32) {
            panic!("atlas is in invalid state");
        }

        return glyph;
    }

    fn write_glyph_data(&mut self, glyph: Glyph, data: GlyphData) {
        let glyph_x = glyph.x as usize;
        let glyph_y = glyph.y as usize;

        let atlas_height = self.atlas_height as usize;
        let atlas_width = self.atlas_width as usize;
        let glyph_height = self.glyph_height as usize;
        let glyph_width = self.glyph_width as usize;
        let data_height = data.height as usize;
        let data_width = data.width as usize;
        let data = data.data;

        let (pad_l, pad_r) = (PAD_L as usize, PAD_R as usize);
        let (pad_t, pad_b) = (PAD_T as usize, PAD_B as usize);

        let data_begin_row = atlas_height - data_height - pad_b;
        let data_end_row = atlas_height - pad_b;
        let data_begin_col = glyph_x + pad_l;
        let data_end_col = glyph_x + pad_l + data_width;

        let glyph_begin_row = glyph_y + pad_t;
        // glyph_end_row == data_end_row
        // glyph_begin_col == data_begin_col
        let glyph_end_col = glyph_x + glyph_width - pad_r;

        // Could probably skip this step
        // Empty out any space that might've been filled by a previous glyph
        for row in glyph_begin_row..data_begin_row {
            let begin = row * atlas_width + data_begin_col;
            let end = begin + glyph_width;

            self.atlas[begin..end].fill(0);
        }

        for (data_row, atlas_row) in (data_begin_row..data_end_row).enumerate() {
            let begin = atlas_row * atlas_width + data_begin_col;
            let end = begin + data_width;

            let data_begin = data_row * data_width;
            let data_end = data_begin + data_width;

            self.atlas[begin..end].copy_from_slice(&data[data_begin..data_end]);

            // Could probably skip this step
            let begin = atlas_row * atlas_width + data_end_col;
            let end = atlas_row * atlas_width + glyph_end_col;
            self.atlas[begin..end].fill(0);
        }
    }
}

fn char_dimensions(face: &ttf::Face, scale: f32, c: char) -> (u32, u32) {
    let glyph_id = face.glyph_index(c).unwrap();
    let rect = match face.glyph_bounding_box(glyph_id) {
        Some(rect) => rect,
        None => return (0, 0),
    };

    let (metrics, z) = metrics_and_affine(rect, scale);
    return (metrics.width(), metrics.height());
}

fn rasterize_glyph(face: &ttf::Face, scale: f32, id: ttf::GlyphId) -> GlyphData {
    let rect = match face.glyph_bounding_box(id) {
        Some(rect) => rect,
        None => {
            return GlyphData {
                width: 0,
                height: 0,
                data: Vec::new(),
            }
        }
    };

    let (metrics, z) = metrics_and_affine(rect, scale);
    let (width, height) = (metrics.width(), metrics.height());

    let mut builder = Builder::new(width, height, z);
    face.outline_glyph(id, &mut builder);

    let data = builder.raster.get_bitmap();
    return GlyphData {
        width,
        height,
        data,
    };
}

pub struct Builder {
    raster: Raster,
    current: Point,
    affine: Affine,
}

impl Builder {
    pub fn new(w: u32, h: u32, affine: Affine) -> Builder {
        return Builder {
            raster: Raster::new(w as usize, h as usize),
            current: Point { x: 0.0, y: 0.0 },
            affine,
        };
    }
}

impl ttf::OutlineBuilder for Builder {
    fn move_to(&mut self, x: f32, y: f32) {
        self.current.x = x;
        self.current.y = y;
    }

    fn line_to(&mut self, x: f32, y: f32) {
        let pt = Point::new(x, y);
        let z = &self.affine;

        self.raster
            .draw_line(&affine_pt(z, &self.current), &affine_pt(z, &pt));

        self.current = pt;
    }

    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        let p1 = Point::new(x1, y1);
        let dest = Point::new(x, y);
        let z = &self.affine;

        self.raster.draw_quad(
            &affine_pt(z, &self.current),
            &affine_pt(z, &p1),
            &affine_pt(z, &dest),
        );

        self.current = dest;
    }

    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        // x1,y1 and x2,y2 are control points
        let dest = Point::new(x, y);
        let z = &self.affine;
        self.raster
            .draw_line(&affine_pt(z, &self.current), &affine_pt(z, &dest));

        self.current = dest;
    }

    fn close(&mut self) {}
}

fn metrics_and_affine(rect: ttf::Rect, scale: f32) -> (Metrics, Affine) {
    let (xmin, ymin, xmax, ymax) = (rect.x_min, rect.y_min, rect.x_max, rect.y_max);
    let l = (xmin as f32 * scale).floor() as i32;
    let t = (ymax as f32 * -scale).floor() as i32;
    let r = (xmax as f32 * scale).ceil() as i32;
    let b = (ymin as f32 * -scale).ceil() as i32;
    let metrics = Metrics { l, t, r, b };
    let z = Affine::new(scale, 0.0, 0.0, -scale, -l as f32, -t as f32);
    (metrics, z)
}

struct Metrics {
    l: i32,
    t: i32,
    r: i32,
    b: i32,
}

impl Metrics {
    fn width(&self) -> u32 {
        (self.r - self.l) as u32
    }

    fn height(&self) -> u32 {
        (self.b - self.t) as u32
    }
}
