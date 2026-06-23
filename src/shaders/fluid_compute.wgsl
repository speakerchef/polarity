struct Params {
    @size(16) dt: f32,
}

@group(0) @binding(0)
var<storage, read_write> positions: array<vec2f>;
@group(0) @binding(1)
var<storage, read_write> velocities: array<vec2f>;
@group(0) @binding(2)
var<uniform> params: Params;
@group(0) @binding(3)
var<storage, read_write> debug: array<f32>;
//@group(0) @binding(4) 
//var<storage, read_write> densities: array<f32>;
//@group(0) @binding(5) 
//var<storage, read_write> pressure: f32;

var<private> g: f32 = 0.00;
var<private> damp: f32 = 0.5;

const smoothing_radius: f32 = 0.5;
const PI: f32 = 3.1415;
const mass: f32 = 1.0;
const target_density: f32 = 277.0;
const pressure_multiplier: f32 = 5.0;

fn density_2_pressure(density: f32) -> f32 {
    let delta = density - target_density;
    let pressure = delta * pressure_multiplier;
    return pressure;
}

fn smoothing_kernel(radius: f32, dist: f32) -> f32 {
    let volume = PI * pow(radius, 5.0) / 10.0;
    let value = max(0, radius - dist);
    return value * value * value / volume;
}

fn smoothing_kernel_gradient(radius: f32, dist: f32) -> f32 {
    if dist > radius { return 0.0; }
    let delta = radius - dist;
    let gradient = -(delta * delta) * 30.0 / (PI * pow(radius, 5.0));
    return gradient;
}

fn calculate_density(point: vec2f) -> f32 {
    var density = 0.0;

    for (var i = 0u; i < arrayLength(&positions); i++) {
        let dist = length(positions[i] - point);
        let influence = smoothing_kernel(smoothing_radius, dist);
        density += mass * influence;
    }

    return density;
}

fn calculate_pressure_force(point_idx: u32) -> vec2f {
    // p = sum(props) * mass * / density * influence
    var pressure_force = vec2f(0);

    for (var i = 0u; i < arrayLength(&positions); i++) {

        if point_idx == i { continue; }

        let offset = positions[i] - positions[point_idx];
        let dist = length(offset);

        if dist < 0.0001 { continue; }

        let dir = offset / dist;
        let slope: vec2f = vec2f(smoothing_kernel_gradient(smoothing_radius, dist));
        let density = calculate_density(positions[i]);
        pressure_force += density_2_pressure(density) * dir * slope * mass / density;
    }

    return pressure_force;
}

@compute @workgroup_size(64)
fn cs_main(@builtin(global_invocation_id) id: vec3u) {
    let i = id.x;
    if i >= arrayLength(&positions) {
        return;
    }
    //densities[i] = calculate_density(positions[i]);
    if i == 0u {
        debug[0] = calculate_density(vec2f(0.0, 0.0));
    }

    velocities[i].y -= g * params.dt;
    positions[i] += velocities[i] * params.dt;

    let density = calculate_density(positions[i]);
    let pressure_force = calculate_pressure_force(i);
    let accel = pressure_force / density;
    velocities[i] += accel * params.dt;

    // bounce off floor
    if positions[i].y < -0.9 {
        positions[i].y = -0.9;
        velocities[i].y = -velocities[i].y * damp;
    }
    if positions[i].y > 0.9 {
        positions[i].y = 0.9;
        velocities[i].y = velocities[i].y * damp;
    }
    if positions[i].x > 0.9 {
        positions[i].x = 0.9;
        velocities[i].x = velocities[i].x * damp;
    }
    if positions[i].x < -0.9 {
        positions[i].x = -0.9;
        velocities[i].x = -velocities[i].x * damp;
    }
}
