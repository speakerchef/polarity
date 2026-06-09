struct VertexOut {
    @location(0) color: vec4<f32>,
    @builtin(position) position: vec4<f32>,
};

struct Uniforms {
    @size(16) live_density: u32,
    @size(16) color: vec4<f32>};

@group(0) @binding(0)
var<storage, read> sample_positions: array<vec4<f32>>;
@group(0) @binding(1)
var<uniform> uniforms: Uniforms;

var<private> v_colors: array<vec4<f32>, 3> = array<vec4<f32>, 3>(
    vec4<f32>(1.0, 0.0, 0.0, 1.0),
    vec4<f32>(0.0, 1.0, 0.0, 1.0),
    vec4<f32>(0.0, 0.0, 1.0, 1.0),
);
const NUM_VERTICES: u32 = 3;

@vertex
fn vs_main(@builtin(vertex_index) v_idx: u32) -> VertexOut {
    var index: u32;
    if v_idx > uniforms.live_density * NUM_VERTICES {
        index = uniforms.live_density;
    } else {
        index = v_idx;
    }
    var out: VertexOut;
    out.position = vec4<f32>(sample_positions[index]);
    // out.color = vec4f(0.0, 1.0, 0.0, 1.0);
    out.color = uniforms.color;

    return out;
}

@fragment
fn fs_main(in: VertexOut) -> @location(0) vec4<f32> {
    return in.color;
}
