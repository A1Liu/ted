use image::{ColorType, ImageFormat};
use ttf_parser as ttf;

const FONT_SIZE: f64 = 128.0;
const COLUMNS: u32 = 100;

const COURIER: &[u8] = core::include_bytes!("./cour.ttf");

pub fn process() {
    let id = 6;

    let gid = ttf::GlyphId(id);

    let face = ttf::Face::from_slice(COURIER, 0).unwrap();
    if face.is_variable() {
        panic!("Can't handle variable fonts");
    }

    let size = 200;
    let ppem = face.units_per_em();
    let scale = (size as f32) / (ppem as f32);

    let rect = face.glyph_bounding_box(gid).unwrap();
    let (xmin, ymin, xmax, ymax) = (rect.x_min, rect.y_min, rect.x_max, rect.y_max);
    let (metrics, z) = metrics_and_affine(xmin, ymin, xmax, ymax, scale);

    let height = face.units_per_em() as usize;
    let mut builder = Builder::new(metrics.width(), metrics.height(), z);
    face.outline_glyph(gid, &mut builder);
    let (w, h) = (builder.raster.w as u32, builder.raster.h as u32);
    println!("w: {} h: {}", w, h);
    let buffer = builder.raster.get_bitmap();
    image::save_buffer_with_format("output.png", &buffer, w, h, ColorType::L8, ImageFormat::Png)
        .unwrap();
}

struct Metrics {
    l: i32,
    t: i32,
    r: i32,
    b: i32,
}

impl Metrics {
    fn width(&self) -> usize {
        (self.r - self.l) as usize
    }

    fn height(&self) -> usize {
        (self.b - self.t) as usize
    }
}

fn metrics_and_affine(xmin: i16, ymin: i16, xmax: i16, ymax: i16, scale: f32) -> (Metrics, Affine) {
    let l = (xmin as f32 * scale).floor() as i32;
    let t = (ymax as f32 * -scale).floor() as i32;
    let r = (xmax as f32 * scale).ceil() as i32;
    let b = (ymin as f32 * -scale).ceil() as i32;
    let metrics = Metrics { l, t, r, b };
    let z = Affine::new(scale, 0.0, 0.0, -scale, -l as f32, -t as f32);
    (metrics, z)
}

pub struct Builder {
    raster: Raster,
    current: Point,
    affine: Affine,
}

impl Builder {
    pub fn new(w: usize, h: usize, affine: Affine) -> Builder {
        return Builder {
            raster: Raster::new(w, h),
            current: Point { x: 0.0, y: 0.0 },
            affine,
        };
    }

    pub fn print(&self) {
        let map = self.raster.get_bitmap();

        let height = self.raster.h;
        for row in 0..height {
            let begin = row * height;
            let end = begin + height;
            let row = &map[begin..end];
            println!("{}", unsafe { core::str::from_utf8_unchecked(row) });
        }
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

// Temporary, from font-rs

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
        //println!("draw_quad {} {} {}", p0, p1, p2);
        let devx = p0.x - 2.0 * p1.x + p2.x;
        let devy = p0.y - 2.0 * p1.y + p2.y;
        let devsq = devx * devx + devy * devy;
        if devsq < 0.333 {
            self.draw_line(p0, p2);
            return;
        }
        let tol = 3.0;
        let n = 1 + (tol * (devx * devx + devy * devy)).sqrt().sqrt().floor() as usize;
        //println!("n = {}", n);
        let mut p = p0;
        let nrecip = recip(n as f32);
        let mut t = 0.0;
        for _i in 0..n - 1 {
            t += nrecip;
            let pn = Point::lerp(t, Point::lerp(t, p0, p1), Point::lerp(t, p1, p2));
            self.draw_line(p, pn);
            p = pn;
        }
        self.draw_line(p, p2);
    }

    pub fn get_bitmap(&self) -> Vec<u8> {
        accumulate(&self.a[0..self.w * self.h])
    }
}

fn accumulate(src: &[f32]) -> Vec<u8> {
    let mut acc = 0.0;
    src.iter()
        .map(|c| {
            acc += c;
            let y = acc.abs();
            let y = if y < 1.0 { y } else { 1.0 };
            (255.0 * y) as u8
        })
        .collect()
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

impl Point {
    pub fn lerp(t: f32, p0: Self, p1: Self) -> Self {
        Point {
            x: p0.x + t * (p1.x - p0.x),
            y: p0.y + t * (p1.y - p0.y),
        }
    }
}
pub fn affine_pt(z: Affine, p: Point) -> Point {
    Point {
        x: z.a * p.x + z.c * p.y + z.e,
        y: z.b * p.x + z.d * p.y + z.f,
    }
}
