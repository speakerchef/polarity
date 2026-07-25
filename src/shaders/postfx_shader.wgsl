struct Params {
    @size(16) use_bloom: u32,
    @size(16) bloom_amt: f32,
    @size(16) use_vignette: u32,
    @size(16) vignette: f32,
    @size(16) use_chroma: u32,
    @size(16) chroma_shift: f32,
    @size(16) chroma_blur: f32,
    @size(16) chroma_direction: u32,
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

const RADIUS: i32 = 3;
const BLOOM_WEIGHTS: array<f32, 7> = array<f32, 7>(
    0.1096489736,
    0.1407920690,
    0.1635770468,
    0.1719638213,
    0.1635770468,
    0.1407920690,
    0.1096489736,
);

@vertex
fn vs_main(@builtin(vertex_index) idx: u32) -> @builtin(position) vec4f {
    return vec4f(positions[idx], 0.0, 1.0);
}

/* Separable bloom stages for performance boost */
@fragment
fn bloom_horizontal(@builtin(position) pos: vec4f) -> @location(0) vec4f {
    let size = vec2f(textureDimensions(tex));
    let uv = pos.xy / size;
    let original = textureSample(tex, tex_sampler, uv);
    let pixel_size = 1 / size;
    var acc = vec4f(0.0);
    var total = 0.0;

    for (var x = -RADIUS; x <= RADIUS; x++) {
        let dist = length(vec2f(f32(x), 0.0));
        let w = BLOOM_WEIGHTS[x + RADIUS];
        let offset = vec2f(f32(x), 0.0) * pixel_size;
        acc += (textureSample(tex, tex_sampler, uv + offset) * w).rgba;
        total += w;
    }
    return original + acc / total * (params.bloom_amt / 4);
}

fn bloom_vertical(uv: vec2f, pixel_size: vec2f) -> vec3f {
    var acc = vec3f(0.0);
    var total = 0.0;

    //  r * r = gaussian kernel
    for (var y = -RADIUS; y <= RADIUS; y++) {
        let dist = length(vec2f(0.0, f32(y)));
        let w = BLOOM_WEIGHTS[y + RADIUS];
        let offset = vec2f(0.0, f32(y)) * pixel_size;
        acc += (textureSample(tex, tex_sampler, uv + offset) * w).rgb;
        total += w;
    }
    return acc / total;
}

fn apply_vignette(uv: vec2f) -> f32 {
    const EDGE: f32 = 0.97;
    var diff_x = 0.0;
    var diff_y = 0.0;
    let absx = uv.x;
    let absy = uv.y;
    let v = min(params.vignette, 0.5);
    if absx >= 1.0 - v {
        diff_x = smoothstep(1.0 - v, EDGE, absx);
    }
    else if absx <= v {
        diff_x = smoothstep(v, 0.0, absx);
    }
    if absy >= 1.0 - v {
        diff_y = smoothstep(1.0 - v, EDGE, absy);
    }
    else if absy <= v {
        diff_y = smoothstep(v, 0.0, absy);
    }
    return max(diff_x, diff_y);
}

@fragment
fn chromatic_aberration(@builtin(position) pos: vec4f) -> @location(0) vec4f {
    let size = vec2f(textureDimensions(tex));
    let uv = pos.xy / size;

    if params.use_chroma == 0 { return textureSample(tex, tex_sampler, uv); }

    var color_sum = vec3f(0.0);
    var weight_sum = vec3f(0.0);

    let samples = max(1.0, params.chroma_blur);
    for (var i = 0.0; i <= 1.0; i += 1.0 / samples) {
        var coord = uv;

        switch params.chroma_direction {
            // linear
            case 0: {
                coord = uv + (i - 0.5) * params.chroma_shift;
            }
            // radial
            case 1: {
                coord = mix(uv, vec2f(0.5), (i - 0.5) * params.chroma_shift);
            }
            default: {
                coord = mix(uv, vec2f(0.5), (i - 0.5) * params.chroma_shift);
            }
        }
        let color = textureSample(tex, tex_sampler, coord).rgb;

        let weight = vec3f(i, 1.0 - abs(i * 2.0 - 1.0), 1.0 - i);
        color_sum += color * color * weight;

        weight_sum += weight;
    }

    return vec4f(sqrt(color_sum / weight_sum), 1.0);
}

@fragment
fn fs_main(@builtin(position) pos: vec4f) -> @location(0) vec4<f32> {
    let size = vec2f(textureDimensions(tex));
    let uv = pos.xy / size;
    let pixel_size = 1.0 / size;

    var color = textureSample(tex, tex_sampler, uv).rgb;
    if params.use_bloom != 0 && params.bloom_amt != 0.0 {
        color += bloom_vertical(uv, pixel_size) * (params.bloom_amt / 4.0);
    }

    var vignette_diff = 0.0;
    if params.use_vignette != 0 {
        vignette_diff = apply_vignette(uv);
    }
    let alpha = 1.0 - vignette_diff;
    return vec4f(color, alpha);
}
