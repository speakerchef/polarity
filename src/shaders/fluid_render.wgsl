struct Params {
    @size(16) dt: f32,
    @size(16) g: f32,
    @size(16) pressure_multiplier: f32,
    @size(16) target_density: f32,
    @size(16) smoothing_radius: f32,
    @size(16) near_pressure_multiplier: f32,
    @size(16) viscosity_strength: f32,
    @size(16) envelope: f32,
    @size(16) point_size: f32,
    @size(16) is_gradient_mode: u32, // !0 == true, 0 == false
    @size(16) uniform_color: vec4f,
    @size(16) is_obstacle: u32,
    @size(16) is_force_outward: u32,
    @size(16) vignette: f32,
    @size(16) edge_damping_factor: f32,
    @size(16) color_invert: u32,
    @size(16) color_arrangement: u32,
    @size(16) luminance_mode: u32,
    @size(16) luminance_floor: f32,
}

struct VertOut {
    @builtin(position) position: vec4f,
    @location(0) uv: vec2f,
    @location(1) speed: f32,
    @location(2) speaker_pos: f32,
    @location(3) is_edge: u32,
    @location(4) edge_bounds_diff: f32,
}

@group(0) @binding(0)
var<storage, read> positions: array<vec2f>;
@group(0) @binding(1)
var<storage, read> velocities: array<vec2f>;
@group(0) @binding(2)
var<uniform> params: Params;
@group(0) @binding(3)
var<storage, read> speaker_position: array<f32>;
//@group(0) @binding(3) 
//var<storage, read> pressure: f32;

//const obstacle_w: f32 = 0.0025;
const obstacle_w: f32 = 0.25;
const obstacle_h: f32 = 0.25;
const obstacle: array<vec2f, 6> = array(vec2f(obstacle_w, obstacle_h), vec2f(obstacle_w, -obstacle_h), vec2f(-obstacle_w, -obstacle_h), vec2f(obstacle_w, obstacle_h), vec2f(-obstacle_w, obstacle_h), vec2f(-obstacle_w, -obstacle_h));
@vertex
fn vs_main(@builtin(vertex_index) i: u32) -> VertOut {
    let ps = params.point_size;
    let quad_pos = array(vec2f(ps, ps), vec2f(-ps, -ps), vec2f(-ps, ps), vec2f(ps, -ps), vec2f(ps, ps), vec2f(-ps, -ps), vec2f(ps, -ps), vec2f(-ps, ps));
    var out: VertOut;
    let particle_id = i / 6u;
    let corner = i % 6u;

    out.position = vec4f(positions[particle_id] + quad_pos[corner], 0.0, 1.0);

    if i >= arrayLength(&positions) * 6 {
        out.position = vec4f(0, 0, 0, 1);
    }
    out.uv = quad_pos[corner];
    out.speed = length(velocities[particle_id]);
    let absx = abs(positions[particle_id] + quad_pos[corner]).x;
    let absy = abs(positions[particle_id] + quad_pos[corner]).y;
    let edge_bounds = 0.97 - params.smoothing_radius;

    // Edge antialiasing
    var edge_diff = 0.0;
    if absx >= edge_bounds {
        edge_diff = smoothstep(0.0, edge_bounds, absx);
    } else if absy >= edge_bounds {
        edge_diff = smoothstep(0.0, edge_bounds, absy);
    }
    // add vignette
    out.edge_bounds_diff = max(smoothstep(edge_bounds - params.vignette, edge_bounds, max(absx, absy)), edge_diff);

    return out;
}

@fragment
fn fs_main(v: VertOut) -> @location(0) vec4f {
    let len = length(v.uv);
    var col = vec4f(1);

    if len > params.point_size {
        discard;
    }

    const LUM_FLOOR_MAX: f32 = 1000;
    var r: f32 = v.speed;
    var g: f32 = v.speed / 4.0;
    var b: f32 = 1.0 - v.speed;
    if params.luminance_mode != 0 {
        r = v.speed - 0.5;
        g = v.speed / 4.0;
        b = pow(min(v.speed, 0.999), max(1.0, pow(params.luminance_floor / 100.0, 3.0) * LUM_FLOOR_MAX));
    }
    if params.color_invert != 0 {
        r = 1.0 - r;
        g = 1.0 - g;
        b = 1.0 - b;
    }

    let alpha = 1.0 - v.edge_bounds_diff;
    if params.is_gradient_mode != 0 {
        // velocity gradient
        switch params.color_arrangement {
            case 0: {
                return vec4f(r, g, b, alpha);
            }
            case 1: {
                return vec4f(g, r, b, alpha);
            }
            case 2: {
                return vec4f(g, b, r, alpha);
            }
            case 3: {
                return vec4f(b, g, r, alpha);
            }
            case 4: {
                return vec4f(b, r, g, alpha);
            }
            case 5: {
                return vec4f(r, b, g, alpha);
            }
            default: {
                return vec4f(r, g, b, alpha);
            }
        }
    } else {
        return vec4f(params.uniform_color.rgb, alpha);
    }
}
