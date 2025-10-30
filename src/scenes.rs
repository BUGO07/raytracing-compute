use glam::*;

use crate::utils::*;

pub fn spheres() -> (Vec<Sphere>, Vec<TriangleMesh>) {
    let spheres = vec![
        Sphere {
            position: Vec3::new(-4.0, 0.4, -0.4),
            radius: 0.4,
            material: Material {
                diffuse_color: Vec3::new(0.2, 0.2, 0.2),
                emission_color: Vec3::new(0.0, 0.0, 0.0),
                emission_strength: 0.0,
                smoothness: 0.0,
            },
        },
        Sphere {
            position: Vec3::new(-2.5, 0.75, -0.2),
            radius: 0.75,
            material: Material {
                diffuse_color: Vec3::new(0.13, 0.51, 0.95),
                emission_color: Vec3::new(0.0, 0.0, 0.0),
                emission_strength: 0.0,
                smoothness: 0.0,
            },
        },
        Sphere {
            position: Vec3::new(-0.5, 1.0, 0.0),
            radius: 1.0,
            material: Material {
                diffuse_color: Vec3::new(0.28, 0.94, 0.07),
                emission_color: Vec3::new(0.23, 1.0, 0.01),
                emission_strength: 0.0,
                smoothness: 0.0,
            },
        },
        Sphere {
            position: Vec3::new(2.0, 1.25, -0.2),
            radius: 1.25,
            material: Material {
                diffuse_color: Vec3::new(1.0, 0.06, 0.06),
                emission_color: Vec3::new(0.0, 0.0, 0.0),
                emission_strength: 0.0,
                smoothness: 0.0,
            },
        },
        Sphere {
            position: Vec3::new(5.5, 2.0, -0.4),
            radius: 2.0,
            material: Material {
                diffuse_color: Vec3::new(1.0, 1.0, 1.0),
                emission_color: Vec3::new(1.0, 1.0, 1.0),
                emission_strength: 0.0,
                smoothness: 0.0,
            },
        },
        Sphere {
            position: Vec3::new(0.0, -100.0, 0.0),
            radius: 100.0,
            material: Material {
                diffuse_color: Vec3::new(0.38, 0.16, 0.81),
                emission_color: Vec3::new(0.38, 0.16, 0.81),
                emission_strength: 0.0,
                smoothness: 0.0,
            },
        },
    ];

    let triangle_meshes = vec![TriangleMesh {
        vertices: vec![
            Vec3::new(-3.0, 0.0, -3.0),
            Vec3::new(-1.0, 0.0, -3.0),
            Vec3::new(-2.0, 2.0, -3.0),
        ],
        aabb: Aabb {
            min: Vec3::new(-3.0, 0.0, -3.0).extend(0.0),
            max: Vec3::new(-1.0, 2.0, -3.0).extend(0.0),
        },
        material: Material {
            diffuse_color: Vec3::new(1.0, 0.5, 0.0),
            emission_color: Vec3::new(1.0, 0.5, 0.0),
            emission_strength: 0.5,
            smoothness: 0.0,
        },
    }];

    (spheres, triangle_meshes)
}

pub fn cornell_box() -> (Vec<Sphere>, Vec<TriangleMesh>) {
    let spheres = vec![
        Sphere {
            position: Vec3::new(-3.0, 0.0, 0.0),
            radius: 1.0,
            material: Material {
                diffuse_color: Vec3::new(1.0, 1.0, 0.0),
                emission_color: Vec3::new(1.0, 1.0, 0.0),
                emission_strength: 0.2,
                smoothness: 0.2,
            },
        },
        Sphere {
            position: Vec3::new(0.0, 0.0, 0.0),
            radius: 1.0,
            material: Material {
                diffuse_color: Vec3::new(1.0, 1.0, 1.0),
                emission_color: Vec3::new(1.0, 1.0, 1.0),
                emission_strength: 0.0,
                smoothness: 1.0,
            },
        },
        Sphere {
            position: Vec3::new(3.0, 0.0, 0.0),
            radius: 1.0,
            material: Material {
                diffuse_color: Vec3::new(0.0, 1.0, 0.0),
                emission_color: Vec3::new(0.0, 1.0, 0.0),
                emission_strength: 0.2,
                smoothness: 0.1,
            },
        },
    ];

    let triangle_meshes = vec![
        // bottom
        TriangleMesh {
            vertices: vec![
                Vec3::new(-10.0, -10.0, -10.0),
                Vec3::new(10.0, -10.0, -10.0),
                Vec3::new(10.0, -10.0, 10.0),
                Vec3::new(-10.0, -10.0, -10.0),
                Vec3::new(10.0, -10.0, 10.0),
                Vec3::new(-10.0, -10.0, 10.0),
            ],
            aabb: Aabb {
                min: Vec3::new(-10.0, -10.0, -10.0).extend(0.0),
                max: Vec3::new(10.0, -10.0, 10.0).extend(0.0),
            },
            material: Material {
                diffuse_color: Vec3::new(0.8, 0.8, 0.8),
                emission_color: Vec3::new(0.0, 0.0, 0.0),
                emission_strength: 0.0,
                smoothness: 0.0,
            },
        },
        // top
        TriangleMesh {
            vertices: vec![
                Vec3::new(-10.0, 10.0, -10.0),
                Vec3::new(10.0, 10.0, -10.0),
                Vec3::new(10.0, 10.0, 10.0),
                Vec3::new(-10.0, 10.0, -10.0),
                Vec3::new(10.0, 10.0, 10.0),
                Vec3::new(-10.0, 10.0, 10.0),
            ],
            aabb: Aabb {
                min: Vec3::new(-10.0, 10.0, -10.0).extend(0.0),
                max: Vec3::new(10.0, 10.0, 10.0).extend(0.0),
            },
            material: Material {
                diffuse_color: Vec3::new(0.8, 0.8, 0.8),
                emission_color: Vec3::new(0.0, 0.0, 0.0),
                emission_strength: 0.0,
                smoothness: 0.0,
            },
        },
        // back
        TriangleMesh {
            vertices: vec![
                Vec3::new(-10.0, -10.0, -10.0),
                Vec3::new(10.0, -10.0, -10.0),
                Vec3::new(10.0, 10.0, -10.0),
                Vec3::new(-10.0, -10.0, -10.0),
                Vec3::new(10.0, 10.0, -10.0),
                Vec3::new(-10.0, 10.0, -10.0),
            ],
            aabb: Aabb {
                min: Vec3::new(-10.0, -10.0, -10.0).extend(0.0),
                max: Vec3::new(10.0, 10.0, -10.0).extend(0.0),
            },
            material: Material {
                diffuse_color: Vec3::new(0.8, 0.8, 0.8),
                emission_color: Vec3::new(0.0, 0.0, 0.0),
                emission_strength: 0.0,
                smoothness: 0.0,
            },
        },
        // left (red)
        TriangleMesh {
            vertices: vec![
                Vec3::new(-10.0, -10.0, -10.0),
                Vec3::new(-10.0, -10.0, 10.0),
                Vec3::new(-10.0, 10.0, 10.0),
                Vec3::new(-10.0, -10.0, -10.0),
                Vec3::new(-10.0, 10.0, 10.0),
                Vec3::new(-10.0, 10.0, -10.0),
            ],
            aabb: Aabb {
                min: Vec3::new(-10.0, -10.0, -10.0).extend(0.0),
                max: Vec3::new(-10.0, 10.0, 10.0).extend(0.0),
            },
            material: Material {
                diffuse_color: Vec3::new(0.8, 0.0, 0.0),
                emission_color: Vec3::new(0.0, 0.0, 0.0),
                emission_strength: 0.0,
                smoothness: 0.0,
            },
        },
        // right (green)
        TriangleMesh {
            vertices: vec![
                Vec3::new(10.0, -10.0, -10.0),
                Vec3::new(10.0, -10.0, 10.0),
                Vec3::new(10.0, 10.0, 10.0),
                Vec3::new(10.0, -10.0, -10.0),
                Vec3::new(10.0, 10.0, 10.0),
                Vec3::new(10.0, 10.0, -10.0),
            ],
            aabb: Aabb {
                min: Vec3::new(10.0, -10.0, -10.0).extend(0.0),
                max: Vec3::new(10.0, 10.0, 10.0).extend(0.0),
            },
            material: Material {
                diffuse_color: Vec3::new(0.0, 0.8, 0.0),
                emission_color: Vec3::new(0.0, 0.0, 0.0),
                emission_strength: 0.0,
                smoothness: 0.0,
            },
        },
        // front
        TriangleMesh {
            vertices: vec![
                Vec3::new(-10.0, -10.0, 10.0),
                Vec3::new(10.0, -10.0, 10.0),
                Vec3::new(10.0, 10.0, 10.0),
                Vec3::new(-10.0, -10.0, 10.0),
                Vec3::new(10.0, 10.0, 10.0),
                Vec3::new(-10.0, 10.0, 10.0),
            ],
            aabb: Aabb {
                min: Vec3::new(-10.0, -10.0, 10.0).extend(0.0),
                max: Vec3::new(10.0, 10.0, 10.0).extend(0.0),
            },
            material: Material {
                diffuse_color: Vec3::new(0.8, 0.8, 0.8),
                emission_color: Vec3::new(0.0, 0.0, 0.0),
                emission_strength: 0.0,
                smoothness: 0.0,
            },
        },
        // light
        TriangleMesh {
            vertices: vec![
                Vec3::new(-2.0, 9.9, -2.0),
                Vec3::new(2.0, 9.9, -2.0),
                Vec3::new(2.0, 9.9, 2.0),
                Vec3::new(-2.0, 9.9, -2.0),
                Vec3::new(2.0, 9.9, 2.0),
                Vec3::new(-2.0, 9.9, 2.0),
            ],
            aabb: Aabb {
                min: Vec3::new(-2.0, 9.9, -2.0).extend(0.0),
                max: Vec3::new(2.0, 10.0, 2.0).extend(0.0),
            },
            material: Material {
                diffuse_color: Vec3::new(1.0, 1.0, 1.0),
                emission_color: Vec3::new(1.0, 1.0, 1.0),
                emission_strength: 5.0,
                smoothness: 0.0,
            },
        },
    ];
    (spheres, triangle_meshes)
}
