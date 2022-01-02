use crate::graphics::*;
use crate::util::*;
use crate::view::*;
use winit::event_loop::ControlFlow;
use winit::window::Window;

#[cfg_attr(debug_assertions, derive(PartialEq))]
pub enum TedCommand {
    DrawView {
        is_lines: bool,
        block_types: Vec<BlockType>,
        text: Vec<char>,
        dims: Rect,
    },

    RequestRedraw,
    Exit,

    ForView {
        command: ViewCommand,
    },
}

#[inline(always)]
pub fn for_view(command: ViewCommand) -> TedCommand {
    return TedCommand::ForView { command };
}

#[cfg_attr(debug_assertions, derive(PartialEq))]
pub enum ViewCommand {
    CursorMove(Direction),
    ToggleCursorBlink,
    Insert { text: String },
    DeleteAfterCursor,
    FlowCursor { index: usize },
    SetContents(SetContents),
    Draw,
}

#[cfg_attr(debug_assertions, derive(PartialEq))]
pub struct SetContents {
    pub start: usize,
    pub start_line: usize,
    pub text: String,
}

#[cfg_attr(debug_assertions, derive(PartialEq))]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    pub fn from_arrow_key(key: winit::event::VirtualKeyCode) -> Option<Self> {
        return match key {
            winit::event::VirtualKeyCode::Up => Some(Self::Up),
            winit::event::VirtualKeyCode::Down => Some(Self::Down),
            winit::event::VirtualKeyCode::Left => Some(Self::Left),
            winit::event::VirtualKeyCode::Right => Some(Self::Right),
            _ => None,
        };
    }
}

// These will ultimately be used to some simple form of RPC. Additionally, structuring
// the code like this should in theory improve testability for command-driven
// components.
pub struct CommandHandler {
    // This should be global
    cache: GlyphCache,

    // This eventually should not be global
    view: View,
}

// TODO this should use a stack instead of a queue, and also the pattern of passing
// a mutable Vec might not be the best once the switch to a stack happens. Maybe
// just return a Vec?
impl CommandHandler {
    pub fn new(text: String) -> Self {
        let mut cache = GlyphCache::new();
        let mut view = View::new(new_rect(35, 20), &text);

        return Self { cache, view };
    }

    pub fn run(&mut self, window: &Window, flow: &mut ControlFlow, command: TedCommand) {
        let mut commands = Vec::with_capacity(8);
        let mut buffer = &mut Vec::with_capacity(8);

        commands.push(command);

        while let Some(command) = commands.pop() {
            match command {
                TedCommand::RequestRedraw => window.request_redraw(),
                TedCommand::Exit => *flow = ControlFlow::Exit,

                TedCommand::DrawView {
                    is_lines,
                    block_types,
                    text,
                    dims,
                } => {
                    let mut did_raster = false;
                    let glyphs_iter = text.into_iter().map(|c| {
                        let res = self.cache.translate_glyph(c);
                        did_raster = did_raster || res.did_raster;
                        return res.glyph;
                    });
                    let block_types_iter = block_types.into_iter().map(BlockTypeData::new);

                    let glyphs: Vec<Glyph> = glyphs_iter.collect();
                    let block_types: Vec<BlockTypeData> = block_types_iter.collect();

                    let atlas_dims = self.cache.atlas_dims();
                    let atlas = did_raster.then(|| self.cache.atlas());

                    let result = TEXT_SHADER.with(|shader| -> Result<(), JsValue> {
                        shader.render(TextShaderInput {
                            is_lines,
                            atlas,
                            block_types,
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

pub struct Command<'a, Value> {
    pub buffer: &'a mut Vec<TedCommand>,
    pub value: Value,
}
