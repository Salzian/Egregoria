struct Uniforms {
    u_view_proj: mat4x4<f32>,
}

struct VertexOutput {
    @location(0) out_color: vec4<f32>,
    @location(1) out_normal: vec3<f32>,
    @location(2) out_wpos: vec3<f32>,
    @location(3) out_uv: vec2<f32>,
    @builtin(position) member: vec4<f32>,
}

@group(0) @binding(0) var<uniform> global: Uniforms;

@vertex
fn vert(@location(0) in_position: vec3<f32>,
        @location(1) in_normal: vec3<f32>,
        @location(2) in_uv: vec2<f32>,
        @location(3) in_color: vec4<f32>) -> VertexOutput {
    let position = global.u_view_proj * vec4(in_position, 1.0);
    return VertexOutput(in_color, in_normal, in_position, in_uv, position);
}
