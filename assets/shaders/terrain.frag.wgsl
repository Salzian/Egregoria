#include "render_params.wgsl"

struct FragmentOutput {
    @location(0) out_color: vec4<f32>,
}

@group(1) @binding(0) var<uniform> params: RenderParams;

@group(2) @binding(0) var t_terraindata: texture_2d<f32>;
@group(2) @binding(1) var s_terraindata: sampler;

@group(3) @binding(0) var t_ssao: texture_2d<f32>;
@group(3) @binding(1) var s_ssao: sampler;
@group(3) @binding(2) var t_bnoise: texture_2d<f32>;
@group(3) @binding(3) var s_bnoise: sampler;
@group(3) @binding(4) var t_sun_smap: texture_depth_2d;
@group(3) @binding(5) var s_sun_smap: sampler_comparison;

#include "shadow.wgsl"
#include "render.wgsl"

fn grid(in_wpos: vec3<f32>, wpos_fwidth_x: f32) -> f32 {
    let level: f32 = wpos_fwidth_x*20.0;//length(vec2(dFdx(in_wpos.x), dFdy(in_wpos.x))) * 0.02;

    var w: f32 = 10000.0;
    var isIn: f32 = 0.0;
    var curgrid: vec2<f32> = in_wpos.xy / 10000.0;

    while(w > level*100.0) {
        w /= 5.0;
        curgrid *= 5.0;
    }

    while(w > level) {
        let moved: vec2<f32> = fract(curgrid);
        let v: f32 = min(min(moved.x, moved.y), min(1.0 - moved.x, 1.0 - moved.y));

        let isOk: f32 = (1.0 - smoothstep(0.004, 0.00415, v)) * 2.0 * (1.0 - smoothstep(level*100.0*0.5, level*100.0, w));
        isIn = max(isIn, isOk);
        w /= 5.0;
        curgrid *= 5.0;
    }
    return isIn;
}

@fragment
fn frag(@location(0) in_normal: vec3<f32>,
        @location(1) in_wpos: vec3<f32>,
        @builtin(position) position: vec4<f32>) -> FragmentOutput {
    var ssao = 1.0;
    if (params.ssao_enabled != 0) {
       ssao = textureSample(t_ssao, s_ssao, position.xy / params.viewport).r;
/*
        if (position.x > params.viewport.x * 0.5) {
            out_color = vec4(vec3(ssao), 1);
            return;
        }*/
    }

    var shadow_v: f32 = 1.0;
    if (params.shadow_mapping_enabled != 0) {
        shadow_v = sampleShadow(in_wpos);
    }

    /*
    out_color = vec4(in_wpos * 0.001, 1);
    return;
    */
/*
    vec2 p = position.xy;
    if (p.x < 500 && p.y < 500) {
        out_color = vec4(vec3(texture(sampler2DShadow(t_sun_smap, s_sun_smap), vec3(p / 500, 1))), 1);
        return;
    }*/
    var c: vec4<f32> = params.grass_col;

    if (params.grid_enabled != 0) {
        c.g += grid(in_wpos, fwidth(in_wpos.x)) * 0.015;
    }

    c = mix(params.sand_col, c, smoothstep(-5.0, 0.0, in_wpos.z));
    c = mix(params.sea_col, c, smoothstep(-25.0, -20.0, in_wpos.z));

    let final_rgb: vec3<f32> = render(params.sun,
                                      params.cam_dir.xyz,
                                      in_wpos,
                                      position.xy,
                                      normalize(in_normal),
                                      c.rgb,
                                      params.sun_col.rgb,
                                      shadow_v,
                                      ssao);
    return FragmentOutput(vec4(final_rgb, c.a));
}
