struct Params {
    camera_pos: vec3f,
    _pad1: u32,
    light_dir: vec3f,
    _pad2: u32,
    width: u32,
    height: u32,
    iTime: f32,
    sphere_count: u32,
};

struct Material {
    diffuse_color: vec3f,
    smoothness: f32,
    emission_color: vec3f,
    emission_strength: f32,
};

struct Sphere {
    position: vec3f,
    radius: f32,
    material: Material,
};

@group(0) @binding(0) var<uniform> params: Params;
@group(0) @binding(1) var outputTex: texture_storage_2d<rgba8unorm, write>;
@group(0) @binding(2) var<storage, read> spheres: array<Sphere>;

const PI: f32 = 3.141592;

const MAX_BOUNCES: u32 = 5u;
const RAYS_PER_PIXEL: u32 = 5u;

struct Ray {
    origin: vec3f,
    direction: vec3f,
};

struct RayHit {
    distance: f32,
    position: vec3f,
    normal: vec3f,
    material: Material,
    hit: bool,
};

fn hash(seed: vec2f) -> u32 {
    var h = u32(seed.x * 73856093.0) ^ u32(seed.y * 19349663.0);
    h = (h ^ (h >> 16u)) * 0x45d9f3bu;
    h = (h ^ (h >> 16u)) * 0x45d9f3bu;
    h = h ^ (h >> 16u);
    return h;
}

fn next_random(state: ptr<function, u32>) -> u32 {
    (*state) = (*state) * 1664525u + 1013904223u;
    let result = ((*state >> ((*state >> 28u) + 4u)) ^ *state) * 277803737u;
    return (result >> 22u) ^ result;
}

fn random_value(state: ptr<function, u32>) -> f32 {
    return f32(next_random(state)) / 4294967295.0;
}

fn random_normal_distribution(state: ptr<function, u32>) -> f32 {
    let theta = 2.0 * PI * random_value(state);
    let rho = sqrt(-2.0 * log(random_value(state)));
    return rho * cos(theta);
}

fn random_direction(state: ptr<function, u32>) -> vec3f {
    let x = random_normal_distribution(state);
    let y = random_normal_distribution(state);
    let z = random_normal_distribution(state);
    return normalize(vec3f(x, y, z));
}
const GROUND_COLOR: vec3f = vec3f(0.35, 0.3, 0.35);
const SKY_COLOR_HORIZON: vec3f = vec3f(1.0, 1.0, 1.0);
const SKY_COLOR_ZENITH: vec3f = vec3f(0.8, 0.8, 0.8);

const SUN_INTENSITY: f32 = 10.0;
const SUN_FOCUS: f32 = 500.0;
const SUN_COLOR: vec3f = vec3f(1.0, 0.9, 0.6);

fn get_environment_light(ray: Ray, light_dir: vec3f) -> vec3f {
    let sky_gradient = mix(SKY_COLOR_HORIZON,
        SKY_COLOR_ZENITH,
        pow(smoothstep(0.0, 0.4, ray.direction.y), 0.35));

    let sun = pow(max(dot(ray.direction, light_dir), 0.0), SUN_FOCUS) * SUN_INTENSITY;

    let ground_to_sky = smoothstep(-0.01, 0.0, ray.direction.y);
    let sun_mask = ground_to_sky >= 1.0;

    return mix(GROUND_COLOR, sky_gradient, ground_to_sky) + sun * SUN_COLOR * f32(u32(sun_mask));
}

fn smoothstep(edge0: f32, edge1: f32, x: f32) -> f32 {
    let t = clamp((x - edge0) / (edge1 - edge0), 0.0, 1.0);
    return t * t * (3.0 - t * 2.0);
}

fn trace(ray: ptr<function, Ray>, state: ptr<function, u32>) -> vec3f {
    var total_light = vec3f(0.0);
    for (var i = 0u; i < RAYS_PER_PIXEL; i = i + 1u) {
        var r = *ray;
        total_light = total_light + trace_single(&r, state);
    }

    return total_light / f32(RAYS_PER_PIXEL);
}

fn trace_single(ray: ptr<function, Ray>, state: ptr<function, u32>) -> vec3f {
    var light = vec3f(0.0, 0.0, 0.0);
    var color = vec3f(1.0, 1.0, 1.0);

    for (var bounce: u32 = 0u; bounce < MAX_BOUNCES; bounce = bounce + 1u) {
        let hit = calculate_collision(*ray);
        if hit.hit {
            (*ray).origin = hit.position + hit.normal * 0.001;
            let diffuse = normalize(hit.normal + random_direction(state));
            let specular = reflect((*ray).direction, hit.normal);

            (*ray).direction = mix(diffuse, specular, hit.material.smoothness);

            let emitted = hit.material.emission_color * hit.material.emission_strength;
            color = color * hit.material.diffuse_color;
            light = light + color * emitted;
        } else {
            light = light + get_environment_light(*ray, params.light_dir) * color;
            break;
        }
    }

    return light;
}

fn reflect(I: vec3f, N: vec3f) -> vec3f {
    return I - 2.0 * dot(N, I) * N;
}

fn calculate_collision(ray: Ray) -> RayHit {
    var closest_hit: RayHit;
    for (var i: u32 = 0u; i < params.sphere_count; i = i + 1u) {
        let hit = sphere_intersect(ray, spheres[i]);
        if hit.hit && (!closest_hit.hit || hit.distance < closest_hit.distance) {
            closest_hit = hit;
        }
    }
    return closest_hit;
}

fn sphere_intersect(ray: Ray, sphere: Sphere) -> RayHit {
    var hit: RayHit;
    let oc = ray.origin - sphere.position;
    let a = dot(ray.direction, ray.direction);
    let b = 2.0 * dot(oc, ray.direction);
    let c = dot(oc, oc) - sphere.radius * sphere.radius;
    let discriminant = b * b - 4.0 * a * c;
    if discriminant < 0.0 {
        return hit;
    } else {
        let t = (-b - sqrt(discriminant)) / (2.0 * a);
        if t > 0.0 {
            hit.distance = t;
            hit.material = sphere.material;
            hit.position = ray.origin + t * ray.direction;
            hit.normal = normalize(hit.position - sphere.position);
            hit.hit = true;
            return hit;
        } else {
            return hit;
        }
    }
}

@compute @workgroup_size(16, 16)
fn main(@builtin(global_invocation_id) global_ix: vec3<u32>) {
    let fragCoord = vec2f(global_ix.xy) / vec2f(f32(params.width), f32(params.height)) ;

    let aspect_ratio = f32(params.width) / f32(params.height);
    let half_fov_tan = tan(radians(60.0) * 0.5);
    let px = (2.0 * fragCoord.x - 1.0) * half_fov_tan * aspect_ratio;
    let py = (1.0 - 2.0 * fragCoord.y) * half_fov_tan;
    let ray_dir = normalize(vec3f(px, py, -1.0));
    var ray = Ray(params.camera_pos, ray_dir);

    var state = hash(fragCoord.xy + params.iTime);


    let fragColor = vec4f(trace(&ray, &state), 1.0);

    textureStore(outputTex, vec2i(global_ix.xy), fragColor);
}