// Vertex shader

struct Camera {
    view_proj: mat4x4<f32>,
}
@group(1) @binding(0)
var<uniform> camera: Camera;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
}
struct InstanceInput {
    @location(5) model_matrix_0: vec4<f32>,
    @location(6) model_matrix_1: vec4<f32>,
    @location(7) model_matrix_2: vec4<f32>,
    @location(8) model_matrix_3: vec4<f32>,
}

struct ValueInput {
    @location(9) value: f32
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) position:vec4<f32>,
    @location(2) value: f32
}

@vertex
fn vs_main(
    model: VertexInput,
    instance: InstanceInput,
    instance_value: ValueInput,
) -> VertexOutput {
    let model_matrix = mat4x4<f32>(
        instance.model_matrix_0,
        instance.model_matrix_1,
        instance.model_matrix_2,
        instance.model_matrix_3,
    );

    let scale_factor = instance_value.value * 0.5;
    let scaling = mat4x4<f32>(
        scale_factor, 0.0, 0.0, 0.0,
        0.0, scale_factor, 0.0, 0.0,
        0.0, 0.0, scale_factor, 0.0,
        0.0, 0.0, 0.0, 1.0
    );

    var out: VertexOutput;
    out.tex_coords = model.tex_coords;
    out.clip_position = camera.view_proj  * model_matrix * scaling *  vec4<f32>(model.position, 1.0);
//    out.clip_position = model_matrix * vec4<f32>(model.position, 1.0);

    out.position = instance_value.value  * model_matrix * vec4<f32>(model.position, 1.0);
    out.value = instance_value.value;
    return out;
}

// Fragment shader

@group(0) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(0)@binding(1)
var s_diffuse: sampler;

fn to_rainbow(v: f32) -> vec4f{
    // map v to a rainbow color in rgb
    let r = clamp(1.5 - abs(v - 0.75) * 4.0, 0.0, 1.0);
    let g = clamp(1.5 - abs(v - 0.50) * 4.0, 0.0, 1.0);
    let b = clamp(1.5 - abs(v - 0.25) * 4.0, 0.0, 1.0);
    return vec4<f32>(r, g, b, 1.0);
    }

fn to_fire(value: f32) -> vec4f{
    var colors= array<vec3f, 6> (
        vec3(0.0, 0.0, 0.0),     // Black (for value = 0)
        vec3(0.125, 0.0, 0.0),  // Dark red
        vec3(0.25, 0.0, 0.0),   // Darker red
        vec3(1.0, 0.25, 0.0),   // Orange
        vec3(1.0, 1.0, 0.0),    // Yellow
        vec3(1.0, 1.0, 1.0)    // White (for value = 1)
    );
    
    let index = value * 4.0;
    let lowerIndex = u32(floor(index));
    let upperIndex = u32(ceil(index));
    let fraction = index - f32(lowerIndex);
    
    let lowerColor = colors[lowerIndex];
    let upperColor = colors[upperIndex];
    
    return vec4<f32>(mix(lowerColor, upperColor, fraction), 1.0);

    }

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return to_rainbow(in.value);
}