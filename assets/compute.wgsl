@group(0) @binding(0) var<uniform> params: Params;
@group(0) @binding(1) var output_tex: texture_storage_2d<rgba8unorm, read_write>;
@group(0) @binding(2) var<storage, read> spheres: array<Sphere>;
@group(0) @binding(3) var<storage, read> triangle_vertices: array<vec4f>;
@group(0) @binding(4) var<storage, read> triangle_meshes: array<TriangleMesh>;

struct Params {
    camera_pos: vec3f,
    random_seed: f32,
    camera_dir: mat3x3f,
    light_dir: vec3f,
    accumulated_frames: u32,
    width: u32,
    height: u32,
};

struct Material {
    diffuse_color: vec3f,
    smoothness: f32,
    emission_color: vec3f,
    emission_strength: f32,
    refractive_index: f32,
    flag: u32,
};

struct Sphere {
    position: vec3f,
    radius: f32,
    material: Material,
};

struct TriangleMesh {
    _pad: vec2f,
    start_index: u32,
    vertex_count: u32,
    aabb: Aabb,
    material: Material,
};

struct Aabb {
    min: vec4f,
    max: vec4f,
};

struct Ray {
    origin: vec3f,
    direction: vec3f,
};

struct RayHit {
    distance: f32,
    position: vec3f,
    normal: vec3f,
    is_backface: bool,
    material: Material,
    hit: bool,
};

const PI: f32 = 3.141592;

const MAX_BOUNCES: u32 = 5u;
const RAYS_PER_PIXEL: u32 = 10u;

const GROUND_COLOR: vec3f = vec3f(0.35, 0.3, 0.35);
const SKY_COLOR_HORIZON: vec3f = vec3f(1.0, 1.0, 1.0);
const SKY_COLOR_ZENITH: vec3f = vec3f(0.08, 0.37, 0.73);

const SUN_INTENSITY: f32 = 10.0;
const SUN_FOCUS: f32 = 500.0;
const SUN_COLOR: vec3f = vec3f(1.0, 0.9, 0.6);

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
            (*ray).origin = hit.position;
            let diffuse = normalize(hit.normal + random_direction(state));
            let specular = reflect((*ray).direction, hit.normal, 0.0);

            if hit.material.flag == 1u {
                if hit.is_backface {
                    let refracted = refract((*ray).direction, hit.normal, hit.material.refractive_index);
                    let kr = pow(1.0 - max(dot(-(*ray).direction, hit.normal), 0.0), 5.0);
                    (*ray).direction = normalize(mix(refracted, specular, kr));
                }
            } else {
                (*ray).direction = mix(diffuse, specular, hit.material.smoothness);
            }

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

fn reflect(I: vec3f, N: vec3f, index: f32) -> vec3f {
    return I - 2.0 * dot(N, I) * N;
}

fn refract(I: vec3f, N: vec3f, index: f32) -> vec3f {
    var cosi = clamp(dot(I, N), -1.0, 1.0);
    let etai = 1.0;
    let etat = index;
    var n = N;
    var eta = etai / etat;
    if cosi < 0.0 {
        cosi = -cosi;
    } else {
        n = -N;
    }
    let k = 1.0 - eta * eta * (1.0 - cosi * cosi);
    if k < 0.0 {
        return vec3f(0.0, 0.0, 0.0);
    } else {
        return eta * I + (eta * cosi - sqrt(k)) * n;
    }
}

fn calculate_collision(ray: Ray) -> RayHit {
    var closest_hit: RayHit;
    for (var i: u32 = 0u; i < arrayLength(&spheres); i = i + 1u) {
        let hit = sphere_intersect(ray, spheres[i]);
        if hit.hit && (!closest_hit.hit || hit.distance < closest_hit.distance) {
            closest_hit = hit;
        }
    }
    for (var i: u32 = 0u; i < arrayLength(&triangle_meshes); i = i + 1u) {
        let tri_mesh = triangle_meshes[i];
        if aabb_intersect(ray, tri_mesh.aabb) {
            for (var j: u32 = 0u; j < tri_mesh.vertex_count / 3u; j = j + 1u) {
                let hit = triangle_intersect(ray, tri_mesh.start_index + j * 3u, tri_mesh.material.flag == 0u);
                if hit.hit && (!closest_hit.hit || hit.distance < closest_hit.distance) {
                    closest_hit = hit;
                    closest_hit.material = tri_mesh.material;
                }
            }
        }
    }
    return closest_hit;
}

fn aabb_intersect(ray: Ray, aabb: Aabb) -> bool {
    let inv_dir = 1.0 / ray.direction;
    let t1 = (aabb.min.xyz - ray.origin) * inv_dir;
    let t2 = (aabb.max.xyz - ray.origin) * inv_dir;

    let tmin = max(max(min(t1.x, t2.x), min(t1.y, t2.y)), min(t1.z, t2.z));
    let tmax = min(min(max(t1.x, t2.x), max(t1.y, t2.y)), max(t1.z, t2.z));

    return tmax >= max(tmin, 0.0);
}

fn sphere_intersect(ray: Ray, sphere: Sphere) -> RayHit {
    var hit: RayHit;
    let oc = ray.origin - sphere.position;
    let a = dot(ray.direction, ray.direction);
    let b = 2.0 * dot(oc, ray.direction);
    let c = dot(oc, oc) - sphere.radius * sphere.radius;
    let discriminant = b * b - 4.0 * a * c;
    if discriminant >= 0.0 {
        let t = (-b - sqrt(discriminant)) / (2.0 * a);
        if t > 1e-5 {
            hit.distance = t;
            hit.material = sphere.material;
            hit.position = ray.origin + t * ray.direction;
            hit.normal = normalize(hit.position - sphere.position);
            hit.is_backface = dot(ray.direction, hit.normal) < 0.0;
            hit.hit = true;
        }
    }
    return hit;
}

// moller-trumbore algorithm
fn triangle_intersect(ray: Ray, v0_idx: u32, detect_backface: bool) -> RayHit {
    var hit: RayHit;
    let v0 = triangle_vertices[v0_idx].xyz;
    let v1 = triangle_vertices[v0_idx + 1u].xyz;
    let v2 = triangle_vertices[v0_idx + 2u].xyz;

    let edge1 = v1 - v0;
    let edge2 = v2 - v0;
    let h = cross(ray.direction, edge2);
    let a = dot(edge1, h);

    if abs(a) < 0.0001 {
        return hit;
    }

    let f = 1.0 / a;
    let s = ray.origin - v0;
    let u = f * dot(s, h);

    if u < 0.0 || u > 1.0 {
        return hit;
    }

    let q = cross(s, edge1);
    let v = f * dot(ray.direction, q);

    if v < 0.0 || u + v > 1.0 {
        return hit;
    }

    let w = 1.0 - u - v;

    let t = f * dot(edge2, q);

    let tri_face_vector = cross(edge1, edge2);
    let determinant = dot(tri_face_vector, ray.direction);
    var is_valid: bool;
    if detect_backface {
        is_valid = abs(determinant) >= 1e-8;
    } else {
        is_valid = determinant >= 1e-8;
    }

    hit.hit = is_valid && t > 1e-5 && u >= 0.0 && v >= 0.0 && w >= 0.0;
    hit.normal = normalize(tri_face_vector) * -sign(determinant);
    hit.distance = t;
    hit.position = ray.origin + t * ray.direction;
    hit.is_backface = determinant > 0.0;
    return hit;
}

@compute @workgroup_size(4, 4)
fn main(@builtin(global_invocation_id) global_ix: vec3<u32>) {
    let frag_coord = vec2f(global_ix.xy) / vec2f(f32(params.width), f32(params.height)) ;

    var state = hash(frag_coord.xy + params.random_seed);

    let aspect_ratio = f32(params.width) / f32(params.height);
    let half_fov_tan = tan(radians(60.0) * 0.5);
    let px = (2.0 * frag_coord.x - 1.0) * half_fov_tan * aspect_ratio;
    let py = (1.0 - 2.0 * frag_coord.y) * half_fov_tan;
    let ray_dir = normalize(params.camera_dir * vec3f(px, py, -1.0) + random_direction(&state) * 0.001);
    var ray = Ray(params.camera_pos, ray_dir);
    let last_frame = textureLoad(output_tex, vec2i(global_ix.xy)).rgb;
    var frag_color = trace(&ray, &state);

    if params.accumulated_frames > 5u {
        frag_color = mix(last_frame, frag_color, 1.0 / f32(params.accumulated_frames - 5u));
    }

    textureStore(output_tex, vec2i(global_ix.xy), vec4f(frag_color, 1.0));
}