struct Vertex {
    @builtin(position) pos: vec4f,
}

struct Params {
    @size(16) goomba: u32,
}

@group(0) @binding(0)
var tex: texture_2d<f32>;
@group(0) @binding(1)
var tex_sampler: sampler;
@group(0) @binding(2)
var<uniform> params: Params;

const positions: array<vec2f, 6> = array(
    vec2f(1, 1),
    vec2f(-1, 1),
    vec2f(-1, -1),
    vec2f(1, 1),
    vec2f(1, -1),
    vec2f(-1, -1),
);

@vertex
fn vs_main(@builtin(vertex_index) idx: u32) -> Vertex {
    var v: Vertex;
    var pos = positions[idx];
    v.pos = vec4f(pos, 0.0, 1.0);
    return v;
}

@fragment
fn fs_main(v: Vertex) -> @location(0) vec4f {
    let size = vec2f(textureDimensions(tex));
    let uv = v.pos.xy / size;
    let color = textureSample(tex, tex_sampler, uv);

    return color;
}
