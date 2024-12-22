struct Input {
  @location(0) pos: vec3<f32>,
  @location(1) color: vec4<f32>,
  @location(2) normal: vec3<f32>,
  @location(3) uv: vec2<f32>,
}

struct LightData {
  color: vec4<f32>,
  intensity: f32,
  flags: f32,
  range: f32,
  inner_angle: f32,
  outer_angle: f32,
  view_proj: mat4x4<f32>,
}

@group(0) @binding(0) var<storage, read> transform: array<mat4x4<f32>>; 
@group(1) @binding(0) var<storage, read> lights: array<LightData>;

var<push_constant> light_index: u32;

@vertex
fn vs_main(
  @builtin(instance_index) instance_index: u32,
  input: Input,
) -> @builtin(position) vec4<f32> {
  let light = lights[light_index];

  let world_pos = transform[instance_index] * vec4<f32>(input.pos, 1.0);
  let light_space_pos = light.view_proj * world_pos;
  return light_space_pos;
}
