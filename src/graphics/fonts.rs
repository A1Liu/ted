use crate::util::*;
use mint::Point2;
use std::collections::hash_map::HashMap;
use ttf_parser as ttf;

const MAX_ATLAS_WIDTH: u32 = 4096;
const COURIER: &[u8] = core::include_bytes!("./cour.ttf");

// These affect font spacing and resolution. They're measured in pixels when writing
// to the texture, but then that texture is sampled to whatever display resolution
// we're targeting.
//                      - Albert Liu, Jan 06, 2022 Thu 01:22 EST
const SIZE: usize = 64; // Raster size in pixels
const PAD_L: u32 = 4; // Raster size in pixels
const PAD_R: u32 = 0; // Raster size in pixels
const PAD_T: u32 = 8; // Raster size in pixels
const PAD_B: u32 = 4; // Raster size in pixels

pub const EMPTY_GLYPH: Glyph = Glyph {
    top_left_1: Point2 { x: 0, y: 0 },
    top_right_1: Point2 { x: 0, y: 0 },
    bot_left_1: Point2 { x: 0, y: 0 },
    top_right_2: Point2 { x: 0, y: 0 },
    bot_left_2: Point2 { x: 0, y: 0 },
    bot_right_2: Point2 { x: 0, y: 0 },
};

const DEFAULT_CHARS: &'static str = core::concat!(
    " ABCDEFGHIJKLMNOPQRSTUVWXYZ",
    "abcdefghijklmnopqrstuvwxyz",
    "0123456789",
    r#"`~!@#$%^&*()_+-=[]{};':",.<>/?\|"#
);

#[derive(Clone, Copy)]
#[repr(C)]
pub struct Glyph {
    // each glyph is 2 trianges of 3 points each
    top_left_1: Point2<u32>,
    top_right_1: Point2<u32>,
    bot_left_1: Point2<u32>,
    top_right_2: Point2<u32>,
    bot_left_2: Point2<u32>,
    bot_right_2: Point2<u32>,
}

impl PartialEq for Glyph {
    fn eq(&self, other: &Self) -> bool {
        return self.top_left_1 == other.top_left_1;
    }
}

pub struct GlyphData {
    pub data: Vec<u8>,
    pub top_offset: isize,
    pub dims: Rect,
}

pub struct GlyphCache {
    descriptors: HashMap<char, Point2<u32>>,
    atlas: Vec<u8>,
    glyph_dims: Rect,
    atlas_dims: Rect,
    atlas_current_row_width: u32,
}

pub struct GlyphResult {
    pub did_raster: bool,
    pub glyph: Glyph,
}

impl GlyphResult {
    pub fn new(did_raster: bool, glyph: Glyph) -> Self {
        return Self { did_raster, glyph };
    }
}

// Actual explanations:
// https://gitlab.redox-os.org/redox-os/rusttype/-/blob/master/src/lib.rs#L175
// https://freetype.org/freetype2/docs/glyphs/glyphs-3.html
impl GlyphCache {
    pub fn new() -> GlyphCache {
        return GlyphCache {
            descriptors: HashMap::new(),
            atlas: Vec::new(),
            glyph_dims: new_rect(0, 0),
            atlas_dims: new_rect(MAX_ATLAS_WIDTH, 0),
            atlas_current_row_width: MAX_ATLAS_WIDTH,
        };
    }

    pub fn atlas_dims(&self) -> Rect {
        return self.atlas_dims;
    }

    pub fn atlas(&self) -> &[u8] {
        return &self.atlas;
    }

    pub fn translate_glyph(&mut self, c: char) -> GlyphResult {
        if let Some(&pos) = self.descriptors.get(&c) {
            let glyph = self.make_glyph(pos);
            return GlyphResult::new(false, glyph);
        }

        // glyph isn't in cache so we don't raster it.
        let face = ttf::Face::from_slice(COURIER, 0).unwrap();
        if face.is_variable() || !face.is_monospaced() {
            panic!("Can't handle variable fonts");
        }

        let (ascent, descent) = (face.ascender(), face.descender());
        let line_gap = face.line_gap();
        let fheight = f32::from(face.ascender()) - f32::from(face.descender());
        let scale = (SIZE as f32) / fheight;

        let height = (ascent - descent + line_gap) as f32 * scale;
        let height = height as u32;

        let mut width = 0;
        for c in DEFAULT_CHARS.chars() {
            // TODO it's monospaced, so use the width of ' ' or something idk
            let glyph_id = face.glyph_index(c).unwrap();
            let w = match face.glyph_bounding_box(glyph_id) {
                None => 0,
                Some(rect) => {
                    let (metrics, z) = metrics_and_affine(rect, scale);
                    metrics.width()
                }
            };

            width = core::cmp::max(width, w);
        }

        let (width, height) = (width + PAD_L + PAD_R, height + PAD_T + PAD_B);

        let descent = (descent as f32 * -scale) as i32;
        let descent = expect(descent.try_into());
        if width < self.glyph_dims.x && height < self.glyph_dims.y {
            let pos = self.add_char(&face, scale, descent, c);
            let glyph = self.make_glyph(pos);
            return GlyphResult::new(true, glyph);
        }

        self.descriptors.clear();
        self.atlas.clear();

        self.glyph_dims = Rect {
            x: width,
            y: height,
        };
        self.atlas_dims.x = MAX_ATLAS_WIDTH / width * width;
        self.atlas_dims.y = 0;
        self.atlas_current_row_width = self.atlas_dims.x;

        for c in DEFAULT_CHARS.chars() {
            self.add_char(&face, scale, descent, c);
        }

        let pos = self.add_char(&face, scale, descent, c);
        let glyph = self.make_glyph(pos);
        return GlyphResult::new(true, glyph);
    }

    fn make_glyph(&self, mut glyph: Point2<u32>) -> Glyph {
        let top_left = glyph;

        glyph.x += self.glyph_dims.x;
        let top_right = glyph;

        glyph.y += self.glyph_dims.y;
        let bot_right = glyph;

        glyph.x -= self.glyph_dims.x;
        let bot_left = glyph;

        return Glyph {
            top_left_1: top_left,
            top_right_1: top_right,
            bot_left_1: bot_left,
            top_right_2: top_right,
            bot_left_2: bot_left,
            bot_right_2: bot_right,
        };
    }

    fn add_char(&mut self, face: &ttf::Face, scale: f32, descent: usize, c: char) -> Point2<u32> {
        if let Some(&glyph) = self.descriptors.get(&c) {
            if (self.atlas_dims.y * self.atlas_dims.x) != (self.atlas.len() as u32) {
                panic!("atlas is in invalid state");
            }

            return glyph;
        }

        if self.atlas_current_row_width + self.glyph_dims.x >= self.atlas_dims.x {
            let glyph_row_size = self.atlas_dims.x * self.glyph_dims.y;

            self.atlas.reserve(glyph_row_size as usize);
            for _ in 0..glyph_row_size {
                self.atlas.push(0);
            }

            self.atlas_current_row_width = 0;
            self.atlas_dims.y += self.glyph_dims.y;
        }

        let glyph_id = face.glyph_index(c).unwrap();
        let glyph_data = rasterize_glyph(face, scale, glyph_id);

        let x = self.atlas_current_row_width;
        self.atlas_current_row_width += self.glyph_dims.x;
        let y = self.atlas_dims.y - self.glyph_dims.y;

        let glyph = Point2 { x, y };
        self.write_glyph_data(glyph, descent, glyph_data);
        self.descriptors.insert(c, glyph);

        if (self.atlas_dims.y * self.atlas_dims.x) != (self.atlas.len() as u32) {
            panic!("atlas is in invalid state");
        }

        return glyph;
    }

    fn write_glyph_data(&mut self, glyph: Point2<u32>, descent: usize, data: GlyphData) {
        let glyph_x = glyph.x as usize;
        let glyph_y = glyph.y as usize;

        let atlas_height = self.atlas_dims.y as usize;
        let atlas_width = self.atlas_dims.x as usize;
        let glyph_height = self.glyph_dims.y as usize;
        let glyph_width = self.glyph_dims.x as usize;
        let data_height = data.dims.y as usize;
        let data_width = data.dims.x as usize;
        let top_offset = data.top_offset;
        let data = data.data;

        let (pad_l, pad_r) = (PAD_L as usize, PAD_R as usize);
        let (pad_t, pad_b) = (PAD_T as usize, PAD_B as usize);

        let data_baseline = atlas_height - pad_b - descent;
        let data_begin_row = (data_baseline as isize - top_offset) as usize;
        let data_end_row = data_begin_row + data_height;
        let data_begin_col = glyph_x + pad_l;
        let data_end_col = glyph_x + pad_l + data_width;

        for (data_row, atlas_row) in (data_begin_row..data_end_row).enumerate() {
            let begin = atlas_row * atlas_width + data_begin_col;
            let end = begin + data_width;

            let data_begin = data_row * data_width;
            let data_end = data_begin + data_width;

            self.atlas[begin..end].copy_from_slice(&data[data_begin..data_end]);
        }
    }
}

fn rasterize_glyph(face: &ttf::Face, scale: f32, id: ttf::GlyphId) -> GlyphData {
    let rect = match face.glyph_bounding_box(id) {
        Some(rect) => rect,
        None => {
            return GlyphData {
                dims: new_rect(0, 0),
                top_offset: 0,
                data: Vec::new(),
            }
        }
    };

    // TODO I hate that this uses Affine in this file instead of just doing a
    // straightforward translation. This code was copied from font-rs so idk
    // what to do about it.
    //                      - Albert Liu, Jan 06, 2022 Thu 01:30 EST
    let (metrics, z) = metrics_and_affine(rect, scale);
    let (width, height) = (metrics.width(), metrics.height());

    let mut builder = Builder::new(width, height, z);
    face.outline_glyph(id, &mut builder);

    let data = builder.raster.get_bitmap();

    return GlyphData {
        dims: new_rect(width, height),
        top_offset: expect((-metrics.t).try_into()),
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
        let pt = Point { x, y };
        let z = self.affine;

        self.raster
            .draw_line(affine_pt(z, self.current), affine_pt(z, pt));

        self.current = pt;
    }

    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        let p1 = Point { x: x1, y: y1 };
        let dest = Point { x, y };
        let z = self.affine;

        self.raster.draw_quad(
            affine_pt(z, self.current),
            affine_pt(z, p1),
            affine_pt(z, dest),
        );

        self.current = dest;
    }

    // TODO This doesn't actually do anything real right now.
    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        // x1,y1 and x2,y2 are control points
        let dest = Point { x, y };
        self.raster.draw_line(self.current, dest);

        self.current = dest;
    }

    fn close(&mut self) {}
}

pub struct Raster {
    w: usize,
    h: usize,
    a: Vec<f32>,
}

impl Raster {
    pub fn new(w: usize, h: usize) -> Raster {
        Raster {
            w: w,
            h: h,
            a: vec![0.0; w * h + 4],
        }
    }

    pub fn draw_line(&mut self, p0: Point, p1: Point) {
        if (p0.y - p1.y).abs() <= core::f32::EPSILON {
            return;
        }
        let (dir, p0, p1) = if p0.y < p1.y {
            (1.0, p0, p1)
        } else {
            (-1.0, p1, p0)
        };
        let dxdy = (p1.x - p0.x) / (p1.y - p0.y);
        let mut x = p0.x;
        let y0 = p0.y as usize; // note: implicit max of 0 because usize (TODO: really true?)
        if p0.y < 0.0 {
            x -= p0.y * dxdy;
        }
        for y in y0..self.h.min(p1.y.ceil() as usize) {
            let linestart = y * self.w;
            let dy = ((y + 1) as f32).min(p1.y) - (y as f32).max(p0.y);
            let xnext = x + dxdy * dy;
            let d = dy * dir;
            let (x0, x1) = if x < xnext { (x, xnext) } else { (xnext, x) };
            let x0floor = x0.floor();
            let x0i = x0floor as i32;
            let x1ceil = x1.ceil();
            let x1i = x1ceil as i32;
            if x1i <= x0i + 1 {
                let xmf = 0.5 * (x + xnext) - x0floor;
                let linestart_x0i = linestart as isize + x0i as isize;
                if linestart_x0i < 0 {
                    continue; // oob index
                }
                self.a[linestart_x0i as usize] += d - d * xmf;
                self.a[linestart_x0i as usize + 1] += d * xmf;
            } else {
                let s = (x1 - x0).recip();
                let x0f = x0 - x0floor;
                let a0 = 0.5 * s * (1.0 - x0f) * (1.0 - x0f);
                let x1f = x1 - x1ceil + 1.0;
                let am = 0.5 * s * x1f * x1f;
                let linestart_x0i = linestart as isize + x0i as isize;
                if linestart_x0i < 0 {
                    continue; // oob index
                }
                self.a[linestart_x0i as usize] += d * a0;
                if x1i == x0i + 2 {
                    self.a[linestart_x0i as usize + 1] += d * (1.0 - a0 - am);
                } else {
                    let a1 = s * (1.5 - x0f);
                    self.a[linestart_x0i as usize + 1] += d * (a1 - a0);
                    for xi in x0i + 2..x1i - 1 {
                        self.a[linestart + xi as usize] += d * s;
                    }
                    let a2 = a1 + (x1i - x0i - 3) as f32 * s;
                    self.a[linestart + (x1i - 1) as usize] += d * (1.0 - a2 - am);
                }
                self.a[linestart + x1i as usize] += d * am;
            }
            x = xnext;
        }
    }

    pub fn draw_quad(&mut self, p0: Point, p1: Point, p2: Point) {
        let devx = p0.x - 2.0 * p1.x + p2.x;
        let devy = p0.y - 2.0 * p1.y + p2.y;
        let devsq = devx * devx + devy * devy;
        if devsq < 0.333 {
            self.draw_line(p0, p2);
            return;
        }
        let tol = 3.0;
        let n = 1 + (tol * (devx * devx + devy * devy)).sqrt().sqrt().floor() as usize;
        let mut p = p0;
        let nrecip = recip(n as f32);
        let mut t = 0.0;
        for _i in 0..n - 1 {
            t += nrecip;
            let pn = lerp(t, lerp(t, p0, p1), lerp(t, p1, p2));
            self.draw_line(p, pn);
            p = pn;
        }
        self.draw_line(p, p2);
    }

    pub fn get_bitmap(&self) -> Vec<u8> {
        let mut acc = 0.0;
        let size = self.w * self.h;
        let mut output = Vec::with_capacity(size);
        for &c in &self.a[0..size] {
            acc += c;
            let y = acc.abs();
            let y = if y < 1.0 { y } else { 1.0 };
            output.push((255.0 * y) as u8);
        }

        return output;
    }
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

fn recip(x: f32) -> f32 {
    x.recip()
}

#[derive(Clone, Copy)]
pub struct Affine {
    a: f32,
    b: f32,
    c: f32,
    d: f32,
    e: f32,
    f: f32,
}

impl Affine {
    fn new(a: f32, b: f32, c: f32, d: f32, e: f32, f: f32) -> Affine {
        return Affine { a, b, c, d, e, f };
    }
}

#[derive(Clone, Copy)]
pub struct Point {
    x: f32,
    y: f32,
}

pub fn lerp(t: f32, p0: Point, p1: Point) -> Point {
    Point {
        x: p0.x + t * (p1.x - p0.x),
        y: p0.y + t * (p1.y - p0.y),
    }
}

pub fn affine_pt(z: Affine, p: Point) -> Point {
    Point {
        x: z.a * p.x + z.c * p.y + z.e,
        y: z.b * p.x + z.d * p.y + z.f,
    }
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
