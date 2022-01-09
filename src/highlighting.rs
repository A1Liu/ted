use mint::*;

#[derive(Clone, Copy)]
#[repr(C)]
pub struct Color(Vector3<f32>);

// TODO Eventually we might wanna use trait objects and a highlighting allocator. For now, let's
// just use an enum

pub struct Region {}

pub struct SyntaxRule {
    matcher: Matcher,
    fg_color: Color,
}

impl SyntaxRule {
    pub fn exact(text: String, fg_color: Color) -> Self {
        return Self {
            matcher: Matcher::Exact { text },
            fg_color,
        };
    }

    pub fn check(&self) -> Option<Color> {
        return None;
    }
}

// TODO lots of small allocations man.
enum Matcher {
    Exact { text: String },
}
