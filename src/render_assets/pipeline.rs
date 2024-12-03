pub struct Pipeline {
    inner: wgpu::RenderPipeline,
}

impl Pipeline {
    pub fn render_pipeline(&self) -> &wgpu::RenderPipeline {
        &self.inner
    }
}
