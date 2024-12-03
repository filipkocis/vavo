pub struct Texture {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
    pub size: wgpu::Extent3d,
}

pub struct Image {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,

    pub data: Vec<u8>,
    pub size: wgpu::Extent3d,
}
