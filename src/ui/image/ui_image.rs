use crate::{prelude::*, render_assets::{BindGroup, Buffer, RenderAsset}};

/// An image UI node component.
#[derive(Clone, Debug)]
pub struct UiImage {
    pub image: Handle<Image>,
    /// Image color gets multiplied by this tint color, defaults to white
    pub tint: Color,
    pub flip_x: bool,
    pub flip_y: bool,
}

impl UiImage {
    pub fn new(image: Handle<Image>) -> Self {
        Self {
            image,
            tint: color::WHITE,
            flip_x: false,
            flip_y: false,
        }
    }

    /// Set a tint color
    pub fn with_tint(mut self, tint: Color) -> Self {
        self.tint = tint;
        self
    }

    /// Set `flip_x` to true
    pub fn flip_x(mut self) -> Self {
        self.flip_x = true;
        self
    }

    /// Set `flip_y` to true
    pub fn flip_y(mut self) -> Self {
        self.flip_y = true;
        self
    }

    fn uniform_data(&self) -> Vec<u8> {
        let mut data = Vec::new();

        data.extend_from_slice(bytemuck::bytes_of(&self.tint));
        
        let booleans = self.flip_x as u32 |
            (self.flip_y as u32) << 1;
        data.extend_from_slice(bytemuck::cast_slice(&[
            booleans, 0, 0, 0
        ]));

        data
    }
}

impl RenderAsset<Buffer> for UiImage {
    fn create_render_asset(
        &self, 
        ctx: &mut SystemsContext,
        _: Option<&EntityId>
    ) -> Buffer {
        Buffer::new("ui_image")
            .create_uniform_buffer(&self.uniform_data(), None, ctx.renderer.device())
    }
}

impl RenderAsset<BindGroup> for UiImage {
    fn create_render_asset(
        &self, 
        ctx: &mut SystemsContext,
        _: Option<&EntityId>
    ) -> BindGroup {
        let image = Some(self.image.clone());

        let buffer: Buffer = self.create_render_asset(ctx, None);
        let uniform = buffer.uniform.expect("UiImage buffer should be uniform");

        BindGroup::build("ui_image")
            .add_texture(&image, ctx, color::WHITE, None, None)
            .add_uniform_buffer(&uniform, wgpu::ShaderStages::VERTEX_FRAGMENT)
            .finish(ctx)
    }
}
