use crate::editor::fonts::*;
use crate::editor::types::*;
use crate::editor::view::*;
use crate::util::*;
use winit::event_loop::ControlFlow;
use winit::window::Window;

pub trait Platform {
    fn render_text(&mut self, input: TextShaderInput);
}

// These will ultimately be used to some simple form of RPC. Additionally, structuring
// the code like this should in theory improve testability for command-driven
// components.
pub struct CommandHandler<P: Platform> {
    // This should be global
    cache: GlyphCache,

    // This eventually should not be global
    view: View,

    platform: P,
}

impl<P: Platform> CommandHandler<P> {
    pub fn new(platform: P, text: String) -> Self {
        let cache = GlyphCache::new();
        let view = View::new(new_rect(35, 20), &text);

        return Self {
            cache,
            view,
            platform,
        };
    }

    pub fn run(&mut self, window: &Window, flow: &mut ControlFlow, command: TedCommand) {
        let mut commands = Vec::with_capacity(8);

        // TODO the pattern of passing a mutable Vec might not be the best. Although,
        // idk what else to use here. Ideally there would be less allocations,
        // not more.
        let buffer = &mut Vec::with_capacity(8);

        commands.push(command);

        while let Some(command) = commands.pop() {
            match command {
                TedCommand::RequestRedraw => window.request_redraw(),
                TedCommand::Exit => *flow = ControlFlow::Exit,

                TedCommand::DrawView {
                    is_lines,
                    fg_colors,
                    bg_colors,
                    text,
                    dims,
                } => {
                    let text_len = text.len();
                    let mut glyphs = Pod::with_capacity(text_len);
                    for c in text.into_iter() {
                        let glyph = self.cache.translate_glyph(c);
                        glyphs.push(glyph);
                    }

                    let atlas_dims = self.cache.atlas_dims();
                    let atlas = self.cache.atlas_data();

                    let color_len = fg_colors.len() * 6;
                    let (fg, bg) = (fg_colors, bg_colors);

                    let mut fg_colors = Pod::with_capacity(color_len);
                    for color in fg {
                        for _ in 0..6 {
                            fg_colors.push(color);
                        }
                    }

                    let mut bg_colors = Pod::with_capacity(color_len);
                    for color in bg {
                        for _ in 0..6 {
                            bg_colors.push(color);
                        }
                    }

                    self.platform.render_text(TextShaderInput {
                        is_lines,
                        atlas,
                        fg_colors,
                        bg_colors,
                        glyphs,
                        atlas_dims,
                        dims,
                    });
                }

                TedCommand::ForView { command } => {
                    let cmd = Command {
                        buffer,
                        value: command,
                    };

                    self.view.run(cmd)
                }
            }

            commands.extend(buffer.drain(..).rev());
        }
    }
}
