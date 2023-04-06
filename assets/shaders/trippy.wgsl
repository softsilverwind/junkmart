#import bevy_sprite::mesh2d_view_bindings
#import bevy_pbr::utils

@group(1) @binding(0)
var texture: texture_2d<f32>;

@group(1) @binding(1)
var our_sampler: sampler;

@fragment
fn fragment(
    @builtin(position) position: vec4<f32>,
    #import bevy_sprite::mesh2d_vertex_output
) -> @location(0) vec4<f32> {
    let uv = coords_to_viewport_uv(position.xy, view.viewport);

    #ifndef IS_TRIPPY
        let r = textureSample(texture, our_sampler, uv).r;
        let g = textureSample(texture, our_sampler, uv).g;
        let b = textureSample(texture, our_sampler, uv).b;
        return vec4<f32>(r, g, b, 1.0);
    #else
        let offset = sin(globals.time) * 0.002;
        let r = textureSample(texture, our_sampler, uv + vec2<f32>(offset, -offset)).r;
        let g = textureSample(texture, our_sampler, uv + vec2<f32>(offset, 0.0)).g;
        let b = textureSample(texture, our_sampler, uv + vec2<f32>(0.0, offset)).b;
        return vec4<f32>(g, b, r, 1.0);
    #endif
}
