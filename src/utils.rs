use glam::*;

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct IParams {
    pub camera_pos: Vec3,
    pub random_seed: f32,
    pub camera_dir: Mat3A,
    pub light_dir: Vec3,
    pub accumulated_frames: u32,
    pub width: u32,
    pub height: u32,
    pub triangle_mesh_count: u32,
    pub sphere_count: u32,
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Sphere {
    pub position: Vec3,
    pub radius: f32,
    pub material: Material,
}

#[repr(C)]
#[derive(Default, Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GPUTriangleMesh {
    pub _pad: Vec2,
    pub start_index: u32,
    pub vertex_count: u32,
    pub aabb: Aabb,
    pub material: Material,
}

pub struct TriangleMesh {
    pub vertices: Vec<Vec3>,
    pub aabb: Aabb,
    pub material: Material,
}

#[repr(C)]
#[derive(Default, Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Aabb {
    pub min: Vec4,
    pub max: Vec4,
}

#[repr(C)]
#[derive(Default, Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Material {
    pub diffuse_color: Vec3,
    pub smoothness: f32,
    pub emission_color: Vec3,
    pub emission_strength: f32,
}
