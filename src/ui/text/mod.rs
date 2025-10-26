use std::sync::Mutex;

use glyphon::{Attrs, Buffer, FontSystem, Metrics, Shaping};

use crate::{
    macros::{Component, RenderAsset},
    prelude::{Color, Resource},
    render_assets::IntoRenderAsset,
};

// TODO: use newtype pattern and derive
impl Resource for glyphon::FontSystem {}
impl Resource for glyphon::TextRenderer {}
impl Resource for glyphon::TextAtlas {}
impl Resource for glyphon::SwashCache {}
impl Resource for glyphon::Viewport {}

#[derive(Component)]
pub struct Text {
    pub content: String,
    pub font_size: f32,
    pub line_height: f32,
    pub attrs: Attrs<'static>,
    pub shaping: Shaping,
}

#[derive(RenderAsset)]
pub struct TextBuffer {
    pub buffer: Mutex<Buffer>,
}

impl TextBuffer {
    /// Set buffer size
    pub fn set_size(&self, font_system: &mut FontSystem, width: Option<f32>, height: Option<f32>) {
        self.buffer
            .lock()
            .unwrap()
            .set_size(font_system, width, height);
    }

    /// Returns buffer width
    pub fn width(&self) -> f32 {
        self.buffer
            .lock()
            .unwrap()
            .layout_runs()
            .map(|line| line.line_w)
            .reduce(f32::max) // .max() workaround for f32
            .unwrap_or_default()
    }

    /// Returns buffer height
    pub fn height(&self) -> f32 {
        self.buffer
            .lock()
            .unwrap()
            .layout_runs()
            .map(|line| line.line_height)
            .sum::<f32>()
    }
}

impl Text {
    pub fn new(content: impl ToString) -> Self {
        Self {
            content: content.to_string(),
            font_size: 16.0,
            line_height: 1.5,
            attrs: Attrs::new(),
            shaping: Shaping::Advanced,
        }
    }

    /// Set font size in pixels
    pub fn font_size(&mut self, size: f32) -> &mut Self {
        self.font_size = size;
        self
    }

    /// Set multiplier for the line height based on font size, not the absolute pixel height
    pub fn line_height(&mut self, height: f32) -> &mut Self {
        self.line_height = height;
        self
    }

    /// Set text color
    pub fn color(&mut self, color: Color) -> &mut Self {
        self.attrs.color_opt = Some(color.into());
        self
    }

    /// Set text shaping strategy
    pub fn shaping(&mut self, shaping: Shaping) -> &mut Self {
        self.shaping = shaping;
        self
    }
}

impl IntoRenderAsset<TextBuffer> for Text {
    fn create_render_asset(
        &self,
        ctx: &mut crate::prelude::SystemsContext,
        _: Option<crate::prelude::EntityId>,
    ) -> TextBuffer {
        let mut font_system = ctx.resources.get_mut::<FontSystem>();

        let metrics = Metrics::relative(self.font_size, self.line_height);

        let mut buffer = Buffer::new(&mut font_system, metrics);
        let mut borrowed_buffer = buffer.borrow_with(&mut font_system);

        borrowed_buffer.set_size(None, None);
        borrowed_buffer.set_text(&self.content, &self.attrs, self.shaping);
        borrowed_buffer.shape_until_scroll(true);

        // borrowed_buffer.set_wrap(Wrap::WordOrGlyph);

        // borrowed_buffer.lines.iter_mut().for_each(|line| {
        //     line.set_align(Some(Align::Center));
        // });

        TextBuffer {
            buffer: Mutex::new(buffer),
        }
    }
}
