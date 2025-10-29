struct Params {
    width: u32,
    height: u32,
    iTime: f32,
};

@group(0) @binding(0) var<uniform> params: Params;
@group(0) @binding(1) var outputTex: texture_storage_2d<rgba8unorm, write>;

struct Ray {
    origin: vec3f,
    direction: vec3f,
};

struct RayHit {
    distance: f32,
    position: vec3f,
    normal: vec3f,
    hit: bool,
};

fn sphere_intersect(ray: Ray, center: vec3f, radius: f32) -> RayHit {
    let oc = ray.origin - center;
    let a = dot(ray.direction, ray.direction);
    let b = 2.0 * dot(oc, ray.direction);
    let c = dot(oc, oc) - radius * radius;
    let discriminant = b * b - 4.0 * a * c;
    if discriminant < 0.0 {
        return RayHit(0.0, vec3f(0.0), vec3f(0.0), false);
    } else {
        let t = (-b - sqrt(discriminant)) / (2.0 * a);
        if t > 0.0 {
            let position = ray.origin + t * ray.direction;
            let normal = normalize(position - center);
            return RayHit(t, position, normal, true);
        } else {
            return RayHit(0.0, vec3f(0.0), vec3f(0.0), false);
        }
    }
}

const PI: f32 = 3.14159265;

@compute @workgroup_size(16, 16)
fn main(@builtin(global_invocation_id) global_ix: vec3<u32>) {
    let fragCoord = vec2f(global_ix.xy) / vec2f(f32(params.width), f32(params.height)) - vec2f(0.5, 0.5);

    let camera_pos = vec3f(0.0, 0.0, 5.0);
    let aspect_ratio = f32(params.width) / f32(params.height);
    let fov = 60.0 * PI / 180.0;
    let px = (2.0 * (fragCoord.x + 0.5) - 1.0) * tan(fov / 2.0) * aspect_ratio;
    let py = (1.0 - 2.0 * (fragCoord.y + 0.5)) * tan(fov / 2.0);
    let ray_dir = normalize(vec3f(px, py, -1.0));
    let ray = Ray(camera_pos, ray_dir);

    let hit = f32(u32(sphere_intersect(ray, vec3f(0.0), 1.0).hit));

    let fragColor = vec4f(hit, hit, hit, 1.0);

    textureStore(outputTex, vec2i(global_ix.xy), fragColor);
}