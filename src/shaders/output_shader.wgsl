struct Vertex {
    @location(0) is_meter: u32,
    @location(1) meter_color: vec4f,
    @location(2) meter_level: f32,
    @builtin(position) pos: vec4f,
}

struct Params {
    @size(16) top_left: vec2f,
    @size(16) use_meter: u32,
    @size(16) meter_level: vec2f,
    @size(16) meter_color: vec4<f32>,
    @size(16) use_gradient: u32,
    @size(16) compensate_height: u32,
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

const METER_WIDTH = 0.030;
@vertex
fn vs_main(@builtin(vertex_index) idx: u32) -> Vertex {
    var v: Vertex;
    var pos = positions[idx];
    if params.use_meter != 0 {
        let original_x = pos.x;
        let new_x = pos.x - METER_WIDTH / 2.0;
        pos.x = new_x;
        let diff = abs(original_x - new_x);
        pos *= 1 - diff;
    }
    v.pos = vec4f(pos, 0.0, 1.0);
    v.is_meter = 0;
    return v;
}

@vertex
fn render_meter(@builtin(vertex_index) v: u32) -> Vertex {

    var vertex: Vertex;
    let half = METER_WIDTH / 2.0;
    let gap = METER_WIDTH / 20.0;

    let compensation = select(0.0, half, params.compensate_height != 0);
    let scale = select(1.0, (1.0 - half), params.compensate_height != 0);
    let hl = min(params.meter_level.x * scale, 1.0);
    let hr = min(params.meter_level.y * scale, 1.0);

    let br_r = vec2f(1.0, -1.0 + compensation);
    let bl_r = vec2f(1.0 - (half - gap), -1.0 + compensation);
    let tr_r = vec2f(1.0, hr);
    let tl_r = vec2f(1.0 - (half - gap), hr);

    let br_l = vec2f(1.0 - (half + gap), -1.0 + compensation);
    let bl_l = vec2f(1.0 - METER_WIDTH, -1.0 + compensation);
    let tr_l = vec2f(1.0 - (half + gap), hl);
    let tl_l = vec2f(1.0 - METER_WIDTH, hl);

    let positions_r = array(bl_r, br_r, tr_r, bl_r, tl_r, tr_r);
    let positions_l = array(bl_l, br_l, tr_l, bl_l, tl_l, tr_l);

    if v < 6 {
        vertex.pos = vec4f(positions_l[v], 0, 1);
        vertex.meter_color = meter_color_gradient(hl);
    } else {
        vertex.pos = vec4f(positions_r[v % 6], 0, 1);
        vertex.meter_color = meter_color_gradient(hr);
    }
    vertex.is_meter = 1;
    vertex.meter_level = min(((vertex.pos.y + 1.0) / 2.0) / scale, 1.0);
    return vertex;
}

fn meter_color_gradient(l: f32) -> vec4f {
    let scaled = pow(l, 2.0);
    let r = scaled * 0.8;
    let g = (1.0 - scaled * scaled) * 0.6;
    let b = max(0.5 - scaled / 2.0, 0.0) * 0.6;
    let a = 1.0;
    return vec4f(r, g, b, a);
}

@fragment
fn fs_main(v: Vertex) -> @location(0) vec4<f32> {
    let size = vec2f(textureDimensions(tex));
    let uv = (v.pos.xy - params.top_left) / size;

    var meter_color = select(params.meter_color, meter_color_gradient(v.meter_level), params.use_gradient != 0);
    if v.meter_level >= 1.0 {
        meter_color = vec4f(1, 0, 0, 1);
    }
    let color = select(textureSample(tex, tex_sampler, uv), meter_color, v.is_meter != 0);

    return vec4f(color);
}
