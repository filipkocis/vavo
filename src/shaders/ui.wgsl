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

@vertex
fn vs_main(
  input: Input,
) -> Output {
  var out: Output;
  out.color = input.color;

  // let screen_pos = vec3<f32>(
  //   input.pos.x / window_size.width * 2.0 - 1.0,
  //   input.pos.y / window_size.height * 2.0 - 1.0,
  //   input.pos.z / 1000.0,
  // );

  var world_pos = transforms[input.transform_index] * vec4<f32>(input.pos, 1.0);
  // out.clip = camera.view_proj * world_pos;

  let mil = 1000000.0;
  let screen_pos = vec4<f32>(
    world_pos.x / window_size.width * 2.0 - 1.0,
    (window_size.height - world_pos.y) / window_size.height * 2.0 - 1.0,
    // we have to flip the z axis for correct z ordering based on z_index
    // z_index may start at 0, so we add 1 to avoid clipping
    // then we convert to NDC with a fake hardcoded far plane at 1mil
    (mil - world_pos.z - 1.0) / mil,
    world_pos.w,
  );
  out.clip = screen_pos;

  return out;
}

@fragment
fn fs_main(input: Output) -> @location(0) vec4<f32> {
  return input.color;
}
