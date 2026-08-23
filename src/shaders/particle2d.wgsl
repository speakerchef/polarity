struct Vertex {
    @location(0) color: vec4<f32>,
    @location(1) local_pos: vec2<f32>,
    @location(2) aa: u32,
    @builtin(position) position: vec4<f32>,
};

struct Params {
    @size(16) live_buffer_len: u32,
    @size(16) trace_buffer_len: u32,
    @size(16) live_mb_buffer_len: u32,
    @size(16) trace_mb_buffer_len: u32,
    @size(16) fs_color: vec4<f32>,
    @size(16) lb_color: vec4<f32>,
    @size(16) mb_color: vec4<f32>,
    @size(16) hb_color: vec4<f32>,
    @size(16) is_mb: u32, // true = !0, false = 0
    @size(16) point_size: f32,
    @size(16) add_border: u32,
};

@group(0) @binding(0)
var<storage, read> sample_positions: array<vec2<f32>>;
@group(0) @binding(1)
var<uniform> params: Params;
@group(0) @binding(2)
var<storage, read> alphas: array<f32>;

fn quad_local_pos(corner: u32) -> vec2<f32> {
    switch corner % 6 {
        case 0u: { return vec2f(1.0, 1.0); }  // l+s, r+s
        case 1u: { return vec2f(-1.0, -1.0); }  // l-s, r-s
        case 2u: { return vec2f(-1.0, 1.0); }  // l-s, r+s
        case 3u: { return vec2f(1.0, -1.0); }  // l+s, r-s
        case 4u: { return vec2f(1.0, 1.0); }  // l+s, r+s
        case 5u: { return vec2f(-1.0, -1.0); }  // l-s, r-s
        default: { return vec2f(0.0, 0.0); }
    }
}

@vertex
fn vs_main(@builtin(vertex_index) v_idx: u32) -> Vertex {
    var vertex: Vertex;
    let ps = params.point_size;
    let quad_pos = array(vec2f(ps, ps), vec2f(-ps, -ps), vec2f(-ps, ps), vec2f(ps, -ps), vec2f(ps, ps), vec2f(-ps, -ps));
    let particle_id = v_idx / 6;
    let corner = v_idx % 6;
    var pos = sample_positions[particle_id] + quad_pos[corner];

    vertex.position = vec4f(pos, 0.0, 1.0);
    vertex.local_pos = quad_local_pos(corner);

    let live_cap = params.live_mb_buffer_len * 3;
    if params.is_mb != 0 {
        if particle_id < live_cap {
            if particle_id < params.live_mb_buffer_len {
                vertex.color = params.lb_color;
            } else if (particle_id >= params.live_mb_buffer_len) && (particle_id < params.live_mb_buffer_len * 2) {
                vertex.color = params.mb_color;
            } else {
                vertex.color = params.hb_color;
            }
        } else {
            if particle_id < (live_cap + params.trace_mb_buffer_len) {
                vertex.color = vec4f(params.lb_color.rgb, alphas[particle_id - live_cap]);
            } else if (particle_id >= (live_cap + params.trace_mb_buffer_len)) && (particle_id < (live_cap + params.trace_mb_buffer_len * 2)) {
                vertex.color = vec4f(params.mb_color.rgb, alphas[particle_id - (live_cap + params.trace_mb_buffer_len)]);
            } else if particle_id > (live_cap + params.trace_mb_buffer_len * 2) {
                vertex.color = vec4f(params.hb_color.rgb, alphas[particle_id - (live_cap + (params.trace_mb_buffer_len * 2))]);
            }
        }
    } else {
        vertex.color = params.fs_color;

        // Use decaying alpha values when rendering trace buffer only
        if particle_id >= params.live_buffer_len {
            vertex.color = vec4<f32>(params.fs_color.rgb, alphas[particle_id - params.live_buffer_len]);
        }
    }
    vertex.aa = 1;
    return vertex;
}

@fragment
fn fs_main(v: Vertex) -> @location(0) vec4<f32> {
    let len = length(v.local_pos);
    var col = v.color;

    if v.aa != 0 && len > 1.0 {
        discard;
    }

    if params.add_border != 0 {
        if len >= 0.85 {
            col = vec4f(0.0);
        } else if len >= 0.8 {
            col = vec4f(col.rgb, col.a * 0.38);
        }
    }

    // anti aliasing + hide corners to get circle
    let alpha = 1 - smoothstep(0.8, 1.0, len);

    return vec4f(col.rgb, v.color.a * alpha);
}
