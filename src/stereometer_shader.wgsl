struct Vertex {
    @location(0) color: vec4<f32>,
    @builtin(position) position: vec4<f32>,
};

struct StereometerParams {
    @size(16) live_buffer_len: u32,
    @size(16) trace_buffer_len: u32,
    @size(16) live_mb_buffer_len: u32,
    @size(16) trace_mb_buffer_len: u32,
    @size(16) fs_color: vec4<f32>,
    @size(16) lb_color: vec4<f32>,
    @size(16) mb_color: vec4<f32>,
    @size(16) hb_color: vec4<f32>,
    @size(16) is_mb: u32, // true = !0, false = 0
};

@group(0) @binding(0)
var<storage, read> sample_positions: array<vec2<f32>>;
@group(0) @binding(1)
var<uniform> params: StereometerParams;
@group(0) @binding(2)
var<storage, read> alphas: array<f32>;

@vertex
fn vs_main(@builtin(vertex_index) v_idx: u32) -> Vertex {
    var vertex: Vertex;
    let pos = sample_positions[v_idx];
    vertex.position = vec4f(pos, 0.0, 1.0);

    let live_cap = params.live_mb_buffer_len * 3;
    if params.is_mb != 0 {
        if v_idx < live_cap {
            if v_idx < params.live_mb_buffer_len {
                vertex.color = params.lb_color;
            } else if (v_idx >= params.live_mb_buffer_len) && (v_idx < params.live_mb_buffer_len * 2) {
                vertex.color = params.mb_color;
            } else {
                vertex.color = params.hb_color;
            }
        } else {
            if v_idx < (live_cap + params.trace_mb_buffer_len) {
                vertex.color = vec4f(params.lb_color.rgb, alphas[v_idx - live_cap]);
            } else if (v_idx >= (live_cap + params.trace_mb_buffer_len)) && (v_idx < (live_cap + params.trace_mb_buffer_len * 2)) {
                vertex.color = vec4f(params.mb_color.rgb, alphas[v_idx - (live_cap + params.trace_mb_buffer_len)]);
            } else if v_idx > (live_cap + params.trace_mb_buffer_len * 2) {
                vertex.color = vec4f(params.hb_color.rgb, alphas[v_idx - (live_cap + (params.trace_mb_buffer_len * 2))]);
            }
        }
    } else {
        vertex.color = params.fs_color;

        // Use decaying alpha values when rendering trace buffer only
        if v_idx >= params.live_buffer_len {
            vertex.color = vec4<f32>(params.fs_color.rgb, alphas[v_idx - params.live_buffer_len]);
        }
    }
    return vertex;
}

@fragment
fn fs_main(v: Vertex) -> @location(0) vec4<f32> {
    return v.color;
}
