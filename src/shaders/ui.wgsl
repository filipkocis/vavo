struct Camera {
  view_proj: mat4x4<f32>,
  view_pos: vec3<f32>,
}

@group(0) @binding(0) var<storage, read> transforms: array<mat4x4<f32>>;
@group(1) @binding(0) var<uniform> camera: Camera; 

struct Input {
  @location(1) color: vec4<f32>,
  @location(0) pos: vec3<f32>,
  @location(2) transform_index: u32,
}

struct Output {
  @builtin(position) clip: vec4<f32>,
  @location(0) color: vec4<f32>,
};

struct WindowSize {
  width: f32,
  height: f32,
}
var<push_constant> window_size: WindowSize;

fn calc_clip_pos(input: Input) -> vec4<f32> {
  var world_pos = transforms[input.transform_index] * vec4<f32>(input.pos, 1.0);
  // out.clip = camera.view_proj * world_pos;

  let mil = 1000000.0;
  let screen_pos = vec4<f32>(
    world_pos.x / window_size.width * 2.0 - 1.0,
    (window_size.height - world_pos.y) / window_size.height * 2.0 - 1.0,
    // we have to flip the z axis for correct z ordering based on z_index
    // z_index may start at 0, so we add 1 to avoid clipping
    // then we convert to NDC with a fake hardcoded far plane at 1mil
    (mil - input.pos.z - 1.0) / mil,
    world_pos.w,
  );
  // out.clip = screen_pos;
  return screen_pos;
}

@vertex
fn vs_main(input: Input) -> Output {
  var out: Output;
  out.color = input.color;

  out.clip = calc_clip_pos(input);   

  return out;
}

@fragment
fn fs_main(input: Output) -> @location(0) vec4<f32> {
  return input.color;
}

@group(2) @binding(0) var texture: texture_2d<f32>;
@group(2) @binding(1) var texture_sampler: sampler;
@group(2) @binding(2) var<uniform> image_data: ImageData; 

struct ImageData {
  tint: vec4<f32>,
  flags: u32,
}

struct ImageOutput {
  @builtin(position) clip: vec4<f32>,
  @location(0) uv: vec2<f32>,
}

@vertex
fn vs_image(
  @builtin(vertex_index) index: u32,
  input: Input,
) -> ImageOutput {
  let flip_x = (image_data.flags & 1u) != 0u;
  let flip_y = (image_data.flags & 2u) != 0u;

  let uv_coords = array<vec2<f32>, 4>(
    vec2<f32>(0.0, 1.0), // Top-left
    vec2<f32>(1.0, 1.0), // Top-right
    vec2<f32>(1.0, 0.0), // Bottom-right
    vec2<f32>(0.0, 0.0), // Bottom-left
  );
  var uv = uv_coords[index % 4];

  // Apply flipping
  if flip_x {
      uv.x = 1.0 - uv.x;
  }
  if flip_y {
      uv.y = 1.0 - uv.y;
  }

  var out: ImageOutput;
  out.clip = calc_clip_pos(input);
  out.uv = uv;

  return out;
}

@fragment
fn fs_image(input: ImageOutput) -> @location(0) vec4<f32> {
  let color = textureSample(texture, texture_sampler, input.uv) * image_data.tint;

  // hack since images get rendered before ui
  if color.a == 0.0 {
    discard;
  }

  return color;
}
