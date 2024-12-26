struct Camera {
  view_proj: mat4x4<f32>,
  view_pos: vec3<f32>,
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
  @location(1) uv: vec2<f32>,
  @location(2) world: vec3<f32>,
  @location(3) world_normal: vec3<f32>,
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
  out.world = world_pos.xyz;
  out.world_normal = normalize((transform[instance_index].srt * vec4<f32>(input.normal, 0.0)).xyz);
  out.clip = camera.view_proj * world_pos; 
  out.uv = input.uv;

  return out;
}

struct Material {
  color: vec4<f32>,
  emissive: vec4<f32>,
  emissive_exposure_weight: f32,
  perceptual_roughness: f32,
  metallic: f32,
  reflectance: f32,
  booleans: u32, // flip, cull, unlit
}

@group(0) @binding(0) var base_texture: texture_2d<f32>;
@group(0) @binding(1) var base_texture_sampler: sampler;
@group(0) @binding(2) var normal_map: texture_2d<f32>;
@group(0) @binding(3) var normal_map_sampler: sampler;
@group(0) @binding(4) var<uniform> material: Material;

struct LightData {
  view_proj: mat4x4<f32>,
  color: vec4<f32>,

  intensity: f32,
  range: f32,
  inner_angle: f32,
  outer_angle: f32,

  flags: u32,
  shadow_map_index: u32,

  pos: vec3<f32>,
  direction: vec3<f32>,
}

struct PushConstant {
  light_count: u32,
}
var<push_constant> pc: PushConstant;

@group(3) @binding(0) var<storage, read> lights: array<LightData>;
@group(3) @binding(1) var directional_shadow_map: texture_depth_2d_array;
@group(3) @binding(2) var point_shadow_map: texture_depth_cube_array;
@group(3) @binding(3) var spot_shadow_map: texture_depth_2d_array;
@group(3) @binding(4) var shadow_map_sampler: sampler_comparison;

@fragment 
fn fs_main(in: Output) -> @location(0) vec4<f32> {
  // let flip_normal_map_y = (material.booleans & 1) != 0;
  // let cull_back_faces = (material.booleans & 2) != 0;
  let unlit = (material.booleans & 4) != 0;

  let material_color: vec4<f32> = textureSample(base_texture, base_texture_sampler, in.uv);
  let object_normal: vec4<f32> = textureSample(normal_map, normal_map_sampler, in.uv);

  if (unlit) {
    return material_color;
  }

  return calculate_final_color(in, material_color);
}

const AMBIENT: u32 = 1;
const DIRECTIONAL: u32 = 2;
const POINT: u32 = 4;
const SPOT: u32 = 8;
const VISIBLE: u32 = 16;
const SHADOW: u32 = 32;

fn calculate_attenuation(light_distance: f32, range: f32, flags: u32) -> f32 {
  if ((flags & POINT) != 0) || ((flags & SPOT) != 0) {
    let constant = 1.0;
    let linear = 0.09;
    let quadratic = 0.032;
    return 1.0 / (constant + linear * light_distance + quadratic * light_distance * light_distance);
  }

  return 1.0;
}

fn calculate_spotlight_intensity(in: Output, light: LightData) -> f32 {
  if ((light.flags & SPOT) == 0) {
    // Not a spotlight
    return 1.0;
  }

  let light_to_frag = normalize(in.world - light.pos);
  let cos_angle = dot(light.direction, light_to_frag);

  let cos_inner_cone = cos(radians(light.inner_angle));
  let cos_outer_cone = cos(radians(light.outer_angle));

  if cos_angle >= cos_inner_cone {
    // Inside the inner cone, full intensity
    return 1.0;
  }
  
  if cos_angle >= cos_outer_cone {
    // Inside the outer cone, interpolate intensity
    let factor = smoothstep(cos_outer_cone, cos_inner_cone, cos_angle);
    return factor;
  }

  // Outside the outer cone
  return 0.0;
}

fn calc_light_dir(light: LightData, world_pos: vec3<f32>, flags: u32) -> vec3<f32> {
  if ((flags & AMBIENT) != 0) {
    return vec3<f32>(0.0);
  }

  if ((flags & DIRECTIONAL) != 0) {
    return normalize(-light.direction);
  }

  return normalize(light.pos - world_pos);
}

fn calculate_light_contribution(material_color: vec3<f32>, in: Output, light_i: u32) -> vec3<f32> {
  let light = lights[light_i];

  // Check if light is visible
  if ((light.flags & VISIBLE) == 0) {
    return vec3<f32>(0.0);
  }

  // Emissive contribution
  let emissive = material.emissive * material.emissive_exposure_weight;

  // Ambient contribution
  if ((light.flags & AMBIENT) != 0) {
    return light.color.rgb * light.intensity * material_color.rgb + emissive.rgb;
  }

  // Light direction and attenuation
  let light_dir = calc_light_dir(light, in.world, light.flags);
  // let distance = length(light.pos - in.world);
  let light_distance = distance(light.pos, in.world);
  let attenuation = calculate_attenuation(light_distance, light.range, light.flags);

  // Diffuse contribution
  let diffuse_strength = max(dot(in.world_normal, light_dir), 0.0);

  // Specular contribution
  let view_dir = normalize(camera.view_pos.xyz - in.world);
  let half_dir = normalize(view_dir + light_dir);
  let shininess = mix(128.0, 4.0, pow(material.perceptual_roughness, 2.0));
  let specular_strength = pow(max(dot(in.world_normal, half_dir), 0.0), shininess);

  // Metalness
  let diffuse_color = mix(material_color, vec3<f32>(0.0), material.metallic);
  let specular_color = mix(vec3<f32>(material.reflectance), material_color, material.metallic);

  // Intensity
  let spotlight_intensity = calculate_spotlight_intensity(in, light);
  let intensity = light.intensity * spotlight_intensity;

  if intensity <= 0.0 {
    // spotlight is outside the cone or intensity is 0, no need for shadow calculations
    return vec3<f32>(0.0);
  }

  // Shadow
  let shadow = calculate_shadow(light, in, light_i, light_dir);

  // Calculate final color
  return shadow * light.color.rgb * attenuation * intensity * (
    diffuse_color * diffuse_strength + specular_color * specular_strength
  ) + emissive.rgb;
}

fn calculate_final_color(in: Output, material_color: vec4<f32>) -> vec4<f32> {
  var final_color: vec3<f32> = vec3<f32>(0.0);

  for (var i = 0u; i < pc.light_count; i = i + 1u) {
    let contribution = calculate_light_contribution(material_color.rgb, in, i);
    final_color += contribution;
  }

  return vec4<f32>(final_color, material_color.a);
}

fn calculate_shadow(light: LightData, in: Output, light_i: u32, light_dir: vec3<f32>) -> f32 {
  if ((light.flags & SHADOW) == 0) {
    return 1.0;
  }

  let homogeneous_coords = light.view_proj * vec4<f32>(in.world, 1.0);
  let flip_correction = vec2<f32>(0.5, -0.5);
  let proj_correction = 1.0 / homogeneous_coords.w;
  let light_local = homogeneous_coords.xy * flip_correction * proj_correction + vec2<f32>(0.5, 0.5);

  if (homogeneous_coords.w <= 0.0) {
    // light is behind the camera
    return 0.0;
  } else if (light_local.x < 0.0 || light_local.x > 1.0 || light_local.y < 0.0 || light_local.y > 1.0) {
    // light is outside the light frustum
    return 0.0;
  }

  let min_bias = 0.001;
  let max_bias = 0.005;

  let n_dot_l = dot(in.world_normal, light_dir);
  let slope_bias = mix(max_bias, min_bias, n_dot_l);
  let depth = homogeneous_coords.z - slope_bias;

  // full shadow if the surface is 90 degrees to the light
  if n_dot_l <= 0.0 {
    return 0.0;
  }

  let layer = light.shadow_map_index;

  if ((light.flags & DIRECTIONAL) != 0) {
    return textureSampleCompareLevel(directional_shadow_map, shadow_map_sampler, light_local, layer, depth * proj_correction);
  }

  if ((light.flags & POINT) != 0) {
    let coords = vec3<f32>(light_local, f32(light.shadow_map_index % 6));
    return textureSampleCompareLevel(point_shadow_map, shadow_map_sampler, coords, layer, depth * proj_correction);
  }

  if ((light.flags & SPOT) != 0) {
    return textureSampleCompareLevel(spot_shadow_map, shadow_map_sampler, light_local, layer, depth * proj_correction);
  }
  
  // no shadow, should be unreachable
  return 1.0;
}
