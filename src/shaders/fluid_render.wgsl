struct Params {
    @size(16) dt: f32,
    @size(16) debug: f32,
}

struct VertOut {
    @builtin(position) position: vec4f,
    @location(0) uv: vec2f,
    @location(1) density: f32,
}

@group(0) @binding(0)
var<storage, read> positions: array<vec2f>;
@group(0) @binding(1)
var<storage, read> velocities: array<vec2f>;
@group(0) @binding(2)
var<uniform> params: Params;
//@group(0) @binding(3) 
//var<storage, read> pressure: f32;

const point_size: f32 = 0.01;
var<private> quad_pos = array(vec2f(point_size, point_size), vec2f(-point_size, -point_size), vec2f(-point_size, point_size), vec2f(point_size, -point_size), vec2f(point_size, point_size), vec2f(-point_size, -point_size), vec2f(point_size, -point_size), vec2f(-point_size, point_size));

@vertex
fn vs_main(@builtin(vertex_index) i: u32) -> VertOut {
    var out: VertOut;
    let particle_id = i / 6u;
    let corner = i % 6u;
    out.position = vec4f(positions[particle_id] + quad_pos[corner], 0.0, 1.0);
    out.uv = quad_pos[corner];
    out.density = velocities[0].x;
    return out;
}

@fragment
fn fs_main(v: VertOut) -> @location(0) vec4f {
    let len = length(v.uv);
    var col = vec4f(1);

    if len > point_size {
        discard;
    }

    //if pressure < 0 {
    //    return vec4f(0, 0.25, 0.75, 1);
    //} else if pressure > 0 {
    //    return vec4f(0.75, 0.10, 0, 1);
    //} else {
    //    return vec4f(1);
    //}

    return vec4f(1, 0.5, 1, 1);
}
