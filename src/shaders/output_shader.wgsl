struct Params {
    @size(16) top_left: vec2f,
}

@group(0) @binding(0)
var tex: texture_2d<f32>;
@group(0) @binding(1)
var tex_sampler: sampler;
@group(0) @binding(2)
var<uniform> params: Params;

var<private> positions: array<vec2f, 6> = array(
    vec2f(1, 1),
    vec2f(-1, 1),
    vec2f(-1, -1),
    vec2f(1, 1),
    vec2f(1, -1),
    vec2f(-1, -1),
);

@vertex
fn vs_main(@builtin(vertex_index) idx: u32) -> @builtin(position) vec4f {
    return vec4f(positions[idx], 0.0, 1.0);
}

@fragment
fn fs_main(@builtin(position) pos: vec4f) -> @location(0) vec4<f32> {
    let size = vec2f(textureDimensions(tex));
    let uv = (pos.xy - params.top_left) / size;

    let color = textureSample(tex, tex_sampler, uv);

    return vec4f(color);
}
