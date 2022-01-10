use crate::commands::*;
use crate::graphics::*;
use crate::highlighting::*;
use crate::util::*;
use crate::view::*;
use winit::event_loop::ControlFlow;
use winit::window::Window;

// These will ultimately be used to some simple form of RPC. Additionally, structuring
// the code like this should in theory improve testability for command-driven
// components.
pub struct CommandHandler {
    // This should be global
    cache: GlyphCache,

    // This eventually should not be global
    view: View,
}

impl CommandHandler {
    pub fn new(text: String) -> Self {
        let mut cache = GlyphCache::new();
        let mut view = View::new(new_rect(35, 20), &text);

        return Self { cache, view };
    }

    pub fn run(&mut self, window: &Window, flow: &mut ControlFlow, command: TedCommand) {
        let mut commands = Vec::with_capacity(8);

        // TODO the pattern of passing a mutable Vec might not be the best. Although,
        // idk what else to use here. Ideally there would be less allocations,
        // not more.
        let mut buffer = &mut Vec::with_capacity(8);

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
                    let mut did_raster = false;
                    let glyphs_iter = text.into_iter().map(|c| {
                        let res = self.cache.translate_glyph(c);
                        did_raster = did_raster || res.did_raster;
                        return res.glyph;
                    });

                    let glyphs: Vec<Glyph> = glyphs_iter.collect();

                    let atlas_dims = self.cache.atlas_dims();
                    let atlas = did_raster.then(|| self.cache.atlas());

                    let fg_colors = fg_colors.into_iter().map(ColorData::new).collect();
                    let bg_colors = bg_colors.into_iter().map(ColorData::new).collect();

                    let result = TEXT_SHADER.with(|shader| -> Result<(), JsValue> {
                        shader.render(TextShaderInput {
                            is_lines,
                            atlas,
                            fg_colors,
                            bg_colors,
                            glyphs,
                            atlas_dims,
                            dims,
                        })?;

                        return Ok(());
                    });

                    expect(result);
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
