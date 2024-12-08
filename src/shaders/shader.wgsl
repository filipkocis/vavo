struct Camera {
  view_proj: mat4x4<f32>,
}

@group(2) @binding(0) var<uniform> camera: Camera; 

struct Input {
  @location(0) pos: vec3<f32>,
  @location(1) color: vec4<f32>,
  @location(2) normal: vec3<f32>,
  @location(3) uv: vec2<f32>,
}

struct Output {
  @builtin(position) clip: vec4<f32>,
  @location(0) color: vec4<f32>,
};

struct Transform {
  srt: mat4x4<f32>,
}

@group(1) @binding(0) var<storage, read> transform: array<Transform>;

@vertex
fn vs_main(
  @builtin(instance_index) instance_index: u32,
  input: Input,
) -> Output {
  var out: Output;
  out.color = input.color;

  var world_pos = transform[instance_index].srt * vec4<f32>(input.pos, 1.0);
  out.clip = camera.view_proj * world_pos; 

  return out;
}

@fragment 
fn fs_main(in: Output) -> @location(0) vec4<f32> {
  return in.color;
}
