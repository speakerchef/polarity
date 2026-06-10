struct Vertex {
    @location(0) color: vec4<f32>,
    @builtin(position) position: vec4<f32>,
};

struct StereometerParams {
    @size(16) live_buffer_len: u32,
    @size(16) trace_buffer_len: u32,
    @size(16) color: vec4<f32>,
};

@group(0) @binding(0)
var<storage, read> sample_positions: array<vec2<f32>>;
@group(0) @binding(1)
var<uniform> params: StereometerParams;
@group(0) @binding(2)
var<storage, read> alphas: array<f32>;

const NUM_VERTICES: u32 = 6;

@vertex
fn vs_main(@builtin(vertex_index) v_idx: u32) -> Vertex {
    var vertex: Vertex;
    vertex.position = vec4f(sample_positions[v_idx], 0.0, 1.0);
    vertex.color = params.color;

    // Use decaying alpha values when rendering trace buffer only
    if v_idx >= params.live_buffer_len {
        vertex.color = vec4<f32>(params.color.rgb, alphas[v_idx - params.live_buffer_len]);
    }
    return vertex;
}

@fragment
fn fs_main(v: Vertex) -> @location(0) vec4<f32> {
    return v.color;
}
