struct Params {
    @size(16) dt: f32,
    @size(16) g: f32,
    @size(16) pressure_multiplier: f32,
    @size(16) target_density: f32,
    @size(16) smoothing_radius: f32,
    @size(16) near_pressure_multiplier: f32,
    @size(16) viscosity_strength: f32,
}

@group(0) @binding(0)
var<storage, read_write> positions: array<vec2f>;
@group(0) @binding(1)
var<storage, read_write> velocities: array<vec2f>;
@group(0) @binding(2)
var<uniform> params: Params;
@group(0) @binding(3)
var<storage, read_write> debug: array<f32>;
@group(0) @binding(4) 
var<storage, read_write> densities: array<vec2f>;
@group(0) @binding(5) 
var<storage, read_write> predicted_positions: array<vec2f>;

var<private> damp: f32 = 0.5;

const PI: f32 = 3.14159265;
const mass: f32 = 1.0;
//const smoothing_radius: f32 = 0.1;
//const target_density: f32 = 200.0;
//const pressure_multiplier: f32 = 5.0;
//const g: f32 = 0.0;

fn density_2_pressure(density: f32) -> f32 {
    let delta = density - params.target_density;
    let pressure = delta * params.pressure_multiplier;
    return pressure;
}

fn near_density_smoothing_kernel(radius: f32, dist: f32) -> f32 {
    let volume = PI * pow(radius, 5.0) / 10.0;
    let v = max(0, radius - dist);
    return v * v * v / volume;
}

fn near_density_smoothing_kernel_gradient(radius: f32, dist: f32) -> f32 {
    if dist > radius { return 0.0; }
    let delta = radius - dist;
    let gradient = -(delta * delta) * 30.0 / (PI * pow(radius, 5.0));
    return gradient;
}
fn density_smoothing_kernel(radius: f32, dist: f32) -> f32 {
    let volume = PI * pow(radius, 4.0) / 6.0;
    let value = max(0, radius - dist);
    return value * value / volume;
}

fn density_smoothing_kernel_gradient(radius: f32, dist: f32) -> f32 {
    if dist > radius { return 0.0; }
    let delta = radius - dist;
    return -delta * 12.0 / (PI * pow(radius, 4.0));
}

fn calculate_density(point: vec2f) -> vec2f {
    var density = 0.0;
    var near_density = 0.0;

    for (var i = 0u; i < arrayLength(&predicted_positions); i++) {
        let dist = length(predicted_positions[i] - point);
        density += mass * density_smoothing_kernel(params.smoothing_radius, dist);
        near_density += mass * near_density_smoothing_kernel(params.smoothing_radius, dist);
    }

    return vec2f(density, near_density);
}

fn calculate_pressure_force(point_idx: u32) -> vec2f {
    var pressure_force = vec2f(0);

    let point_density = densities[point_idx].x;
    let point_near_density = densities[point_idx].y;
    let pressure = density_2_pressure(point_density);
    let near_pressure = params.near_pressure_multiplier * point_near_density;

    for (var i = 0u; i < arrayLength(&predicted_positions); i++) {

        if point_idx == i { continue; }

        let offset = predicted_positions[i] - predicted_positions[point_idx];
        let dist = length(offset);
        let dir = select(vec2f(0.0, 1.0), offset / dist, dist > 0.0);

        let n_density = max(0.0001, densities[i].x);
        let n_near_density = max(0.0001, densities[i].y);
        let n_pressure = density_2_pressure(n_density);
        let n_near_pressure = params.near_pressure_multiplier * n_near_density;

        let shared_pressure = (pressure + n_pressure) * 0.5;
        let shared_near_pressure = (near_pressure + n_near_pressure) * 0.5;

        pressure_force += dir * density_smoothing_kernel_gradient(params.smoothing_radius, dist) * shared_pressure / n_density;
        pressure_force += dir * near_density_smoothing_kernel_gradient(params.smoothing_radius, dist) * shared_near_pressure / n_near_density;
    }

    return pressure_force;
}
fn viscosity_smoothing_kernel(radius: f32, dist: f32) -> f32 {
    if dist >= radius { return 0.0; }
    let volume = (PI * pow(radius, 8.0)) / 4.0;
    let v = radius * radius - dist * dist;
    return v * v * v / volume;
}

fn calculate_viscosity(point_idx: u32) -> vec2f {
    var viscosity_force = vec2f(0);
    let vel = velocities[point_idx];
    let pos = predicted_positions[point_idx];

    for (var i = 0u; i < arrayLength(&predicted_positions); i++) {
        if point_idx == i { continue; }
        let dist = length(predicted_positions[i] - pos);
        let velocity_diff = velocities[i] - vel;
        viscosity_force += velocity_diff * viscosity_smoothing_kernel(params.smoothing_radius, dist);
    }

    return viscosity_force;
}

@compute @workgroup_size(128)
fn cs_calculate_predicted_positions(@builtin(global_invocation_id) id: vec3u) {
    let i = id.x;
    if i >= arrayLength(&positions) { return; }
    velocities[i].y -= -params.g * params.dt;
    let prediction_factor = 1.0 / 120.0;
    predicted_positions[i] = positions[i] + velocities[i] * prediction_factor;
}

@compute @workgroup_size(128)
fn cs_calculate_densities(@builtin(global_invocation_id) id: vec3u) {
    let i = id.x;
    //densities[i] = calculate_density(positions[i]);
    densities[i] = calculate_density(predicted_positions[i]);
    if i == 0u {
        debug[0] = densities[0].x;
    }
}

@compute @workgroup_size(128)
fn cs_calculate_pressure(@builtin(global_invocation_id) id: vec3u) {
    let i = id.x;
    if i >= arrayLength(&positions) { return; }

    let density = densities[i].x;
    let pressure_force = calculate_pressure_force(i);
    let accel = pressure_force / max(density, 0.0001);
    velocities[i] += accel * params.dt;
}

@compute @workgroup_size(128)
fn cs_calculate_viscosity(@builtin(global_invocation_id) id: vec3u) {
    let i = id.x;
    if i >= arrayLength(&positions) { return; }

    velocities[i] += calculate_viscosity(i) * params.viscosity_strength * params.dt;
}

@compute @workgroup_size(128)
fn cs_main(@builtin(global_invocation_id) id: vec3u) {
    let i = id.x;
    if i >= arrayLength(&positions) { return; }

    positions[i] += velocities[i] * params.dt;

    if positions[i].y < -0.9 {
        positions[i].y = -0.9;
        velocities[i].y = -velocities[i].y * damp;
    }
    if positions[i].y > 0.9 {
        positions[i].y = 0.9;
        velocities[i].y = -velocities[i].y * damp;
    }
    if positions[i].x > 0.9 {
        positions[i].x = 0.9;
        velocities[i].x = -velocities[i].x * damp;
    }
    if positions[i].x < -0.9 {
        positions[i].x = -0.9;
        velocities[i].x = -velocities[i].x * damp;
    }
}
