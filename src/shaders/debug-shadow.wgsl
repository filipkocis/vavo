struct Output {
    @location(0) uv: vec2<f32>,
    @builtin(position) clip_position: vec4<f32>,
};

@vertex
fn vs_main(
  @builtin(vertex_index) vi: u32,
) -> Output {
  let uv = vec2<f32>(
    f32((vi << 1u) & 2u),
    f32(vi & 2u),
  );

  var out: Output;
  out.uv = uv;
  out.clip_position = vec4<f32>(uv * 2.0 - 1.0, 0.0, 1.0);
  return out;
}

@group(0) @binding(0) var shadow_atlas: texture_depth_2d;
@group(0) @binding(1) var shadow_atlas_sampler: sampler_comparison;

@fragment
fn fs_main(input: Output) -> @location(0) vec4<f32> {
  let uv = vec2<f32>(input.uv.x, 1.0 - input.uv.y);

  // Directly sample the depth value from the texture
  let depth_value = 1.0 - textureLoad(shadow_atlas, vec2<i32>(uv * vec2<f32>(textureDimensions(shadow_atlas))), 0);

  // Render depth as grayscale
  return vec4<f32>(depth_value, depth_value, depth_value, 1.0);
}
