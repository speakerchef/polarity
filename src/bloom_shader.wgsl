struct Params {
    @size(16) bloom_amt: f32,
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

fn bloom_sample(uv: vec2f, pixel_size: vec2f, radius: i32) -> vec3f {
    var acc = vec3f(0.0);
    var total = 0.0;

    //  r * r = gaussian kernel
    for (var x = -radius; x <= radius; x++) {
        for (var y = -radius; y <= radius; y++) {
            let dist = length(vec2f(f32(x), f32(y)));
            let w = exp(-dist * dist * 0.05);
            let offset = vec2f(f32(x), f32(y)) * pixel_size;
            acc += textureSample(tex, tex_sampler, uv + offset).rgb * w;
            total += w;
        }
    }
    return acc / total;
}

const RADIUS: i32 = 10;

@fragment
fn fs_main(@builtin(position) pos: vec4f) -> @location(0) vec4<f32> {
    let size = vec2f(textureDimensions(tex));
    let uv = (pos.xy) / size;
    let pixel_size = 1.0 / size;

    let src = textureSample(tex, tex_sampler, uv).rgb;
    let bloom = bloom_sample(uv, pixel_size, RADIUS);

    let color = src + bloom * params.bloom_amt;

    return vec4f(color, 1.0);
}
