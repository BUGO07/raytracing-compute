use std::{collections::HashSet, sync::Arc};

use glam::{Mat3A, Quat, Vec2, Vec3};
use wgpu::util::DeviceExt;

use winit::{
    dpi::PhysicalSize,
    event::{Event, KeyEvent, WindowEvent},
    event_loop::EventLoop,
    keyboard::{KeyCode, PhysicalKey},
    window::{CursorGrabMode, Window, WindowBuilder},
};

use crate::utils::*;

mod scenes;
mod utils;

async fn run(event_loop: EventLoop<()>, window: Window) {
    let window = Arc::new(window);
    let window_clone = window.clone();
    let instance = wgpu::Instance::new(&Default::default());
    let surface = instance.create_surface(&window).unwrap();
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: Default::default(),
            force_fallback_adapter: false,
            compatible_surface: Some(&surface),
        })
        .await
        .expect("error finding adapter");

    let (device, queue) = adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                required_features: wgpu::Features::TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES,
                required_limits: Default::default(),
                memory_hints: Default::default(),
            },
            None,
        )
        .await
        .expect("error creating device");
    let mut size = window.inner_size();
    let swapchain_capabilities = surface.get_capabilities(&adapter);
    let format = swapchain_capabilities.formats[0];
    let mut sc = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format,
        width: size.width,
        present_mode: wgpu::PresentMode::AutoNoVsync,
        height: size.height,
        alpha_mode: swapchain_capabilities.alpha_modes[0],
        view_formats: vec![],
        desired_maximum_frame_latency: 2,
    };
    surface.configure(&device, &sc);

    let copy_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: None,
        source: wgpu::ShaderSource::Wgsl(
            std::fs::read_to_string("assets/copy.wgsl").unwrap().into(),
        ),
    });
    let copy_bind_group_layout =
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering),
                    count: None,
                },
            ],
        });
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: None,
        bind_group_layouts: &[&copy_bind_group_layout],
        push_constant_ranges: &[],
    });
    let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: None,
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &copy_shader,
            entry_point: Some("vs_main"),
            buffers: &[],
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &copy_shader,
            entry_point: Some("fs_main"),
            targets: &[Some(format.into())],
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        }),
        primitive: wgpu::PrimitiveState::default(),
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        multiview: None,
        cache: None,
    });

    let mut img = device.create_texture(&wgpu::TextureDescriptor {
        label: None,
        size: wgpu::Extent3d {
            width: size.width,
            height: size.height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    });
    let mut img_view = img.create_view(&Default::default());

    const CONFIG_SIZE: u64 = size_of::<IParams>() as u64;

    let config_dev = device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size: CONFIG_SIZE,
        usage: wgpu::BufferUsages::COPY_DST
            | wgpu::BufferUsages::STORAGE
            | wgpu::BufferUsages::UNIFORM,
        mapped_at_creation: false,
    });

    let (spheres, triangles) = if let Some(arg) = std::env::args().nth(1) {
        match arg.as_str() {
            "cornell" => scenes::cornell_box(),
            "spheres" => scenes::spheres(),
            _ => scenes::spheres(),
        }
    } else {
        scenes::spheres()
    };

    let triangle_vertices = triangles
        .iter()
        .flat_map(|mesh| {
            mesh.vertices
                .iter()
                .map(|x| x.extend(0.0))
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();

    let gpu_triangles = triangles
        .iter()
        .scan(0, |start_index, mesh| {
            let gpu_mesh = GPUTriangleMesh {
                start_index: *start_index,
                vertex_count: mesh.vertices.len() as u32,
                aabb: mesh.aabb,
                material: mesh.material,
                ..Default::default()
            };
            *start_index += gpu_mesh.vertex_count;
            Some(gpu_mesh)
        })
        .collect::<Vec<GPUTriangleMesh>>();

    println!(
        "{:?}",
        gpu_triangles
            .iter()
            .map(|x| (x.start_index, x.vertex_count))
            .collect::<Vec<_>>()
    );

    let sphere_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Sphere Buffer"),
        contents: bytemuck::cast_slice(&spheres),
        usage: wgpu::BufferUsages::STORAGE
            | wgpu::BufferUsages::COPY_DST
            | wgpu::BufferUsages::COPY_SRC,
    });

    let triangle_vertices_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Triangle Buffer"),
        contents: bytemuck::cast_slice(&triangle_vertices),
        usage: wgpu::BufferUsages::STORAGE
            | wgpu::BufferUsages::COPY_DST
            | wgpu::BufferUsages::COPY_SRC,
    });

    let gpu_triangles_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("GPU Triangle Mesh Buffer"),
        contents: bytemuck::cast_slice(&gpu_triangles),
        usage: wgpu::BufferUsages::STORAGE
            | wgpu::BufferUsages::COPY_DST
            | wgpu::BufferUsages::COPY_SRC,
    });

    let cs_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: None,
        source: wgpu::ShaderSource::Wgsl(
            std::fs::read_to_string("assets/compute.wgsl")
                .unwrap()
                .into(),
        ),
    });
    let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: None,
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::StorageTexture {
                    access: wgpu::StorageTextureAccess::ReadWrite,
                    format: wgpu::TextureFormat::Rgba8Unorm,
                    view_dimension: wgpu::TextureViewDimension::D2,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 2,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 3,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 4,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
        ],
    });
    let compute_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: None,
        bind_group_layouts: &[&bind_group_layout],
        push_constant_ranges: &[],
    });
    let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: None,
        layout: Some(&compute_pipeline_layout),
        module: &cs_module,
        entry_point: Some("main"),
        cache: None,
        compilation_options: wgpu::PipelineCompilationOptions::default(),
    });
    let mut bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: None,
        layout: &bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: config_dev.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::TextureView(&img_view),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: sphere_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 3,
                resource: triangle_vertices_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 4,
                resource: gpu_triangles_buffer.as_entire_binding(),
            },
        ],
    });
    let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        address_mode_w: wgpu::AddressMode::ClampToEdge,
        mag_filter: wgpu::FilterMode::Nearest,
        min_filter: wgpu::FilterMode::Nearest,
        mipmap_filter: wgpu::FilterMode::Nearest,
        ..Default::default()
    });
    let mut copy_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: None,
        layout: &copy_bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&img_view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(&sampler),
            },
        ],
    });
    // let start_time = std::time::Instant::now();
    let mut keys_pressed = HashSet::new();
    let mut mouse_grabbed = false;
    let mut light_dir = Vec3::new(0.2, 1.0, 0.05).normalize();
    let mut camera_dir = Quat::IDENTITY;
    let mut camera_pos = Vec3::new(0.0, 0.0, 5.0);
    let mut last_update = std::time::Instant::now();
    let mut mouse_delta = Vec2::ZERO;
    let mut accumulated_frames = 0;

    event_loop
        .run(move |event, target| {
            if let Event::WindowEvent {
                window_id: _,
                event,
            } = event
            {
                match event {
                    WindowEvent::RedrawRequested => {
                        accumulated_frames += 1;
                        let delta_time = last_update.elapsed().as_secs_f32();
                        last_update = std::time::Instant::now();
                        let (mut yaw, mut pitch, _) = camera_dir.to_euler(glam::EulerRot::YXZ);
                        let local_z = camera_dir * Vec3::Z;
                        let forward = -Vec3::new(local_z.x, 0.0, local_z.z).normalize_or_zero();
                        let right = Vec3::new(local_z.z, 0.0, -local_z.x).normalize_or_zero();
                        let mut move_dir = Vec3::ZERO;
                        for code in keys_pressed.iter() {
                            match code {
                                KeyCode::ArrowUp => {
                                    light_dir = (Quat::from_rotation_x(-2.5 * delta_time)
                                        * light_dir)
                                        .normalize();
                                }
                                KeyCode::ArrowDown => {
                                    light_dir = (Quat::from_rotation_x(2.5 * delta_time)
                                        * light_dir)
                                        .normalize();
                                }
                                KeyCode::ArrowLeft => {
                                    light_dir = (Quat::from_rotation_y(-2.5 * delta_time)
                                        * light_dir)
                                        .normalize();
                                }
                                KeyCode::ArrowRight => {
                                    light_dir = (Quat::from_rotation_y(2.5 * delta_time)
                                        * light_dir)
                                        .normalize();
                                }
                                KeyCode::KeyW => {
                                    move_dir += forward;
                                }
                                KeyCode::KeyS => {
                                    move_dir -= forward;
                                }
                                KeyCode::KeyA => {
                                    move_dir -= right;
                                }
                                KeyCode::KeyD => {
                                    move_dir += right;
                                }
                                KeyCode::Space => {
                                    move_dir += Vec3::Y;
                                }
                                KeyCode::ShiftLeft => {
                                    move_dir -= Vec3::Y;
                                }
                                _ => {}
                            }
                            accumulated_frames = 0;
                        }

                        let window_scale = size.height.max(size.width) as f32;
                        pitch -= (mouse_delta.y * window_scale * 0.00015).to_radians();
                        yaw -= (mouse_delta.x * window_scale * 0.00015).to_radians();
                        mouse_delta = Vec2::ZERO;
                        camera_pos += move_dir.normalize_or_zero()
                            * if keys_pressed.contains(&KeyCode::ControlLeft) {
                                30.0
                            } else {
                                10.0
                            }
                            * delta_time;
                        camera_dir = Quat::from_rotation_y(yaw)
                            * Quat::from_rotation_x(pitch.clamp(-1.54, 1.54));

                        let frame = surface
                            .get_current_texture()
                            .expect("error getting texture from swap chain");

                        // let i_time: f32 = start_time.elapsed().as_secs_f32();
                        let config_data = IParams {
                            camera_pos,
                            random_seed: rand::random(),
                            camera_dir: Mat3A::from_quat(camera_dir),
                            light_dir: light_dir.normalize_or_zero(),
                            accumulated_frames,
                            width: size.width,
                            height: size.height,
                            triangle_mesh_count: gpu_triangles.len() as u32,
                            sphere_count: spheres.len() as u32,
                        };
                        queue.write_buffer(&config_dev, 0, bytemuck::bytes_of(&config_data));
                        let mut encoder = device.create_command_encoder(&Default::default());
                        {
                            let mut cpass = encoder.begin_compute_pass(&Default::default());
                            cpass.set_pipeline(&pipeline);
                            cpass.set_bind_group(0, &bind_group, &[]);
                            cpass.dispatch_workgroups(size.width / 4, size.height / 4, 1);
                        }
                        {
                            let view = frame
                                .texture
                                .create_view(&wgpu::TextureViewDescriptor::default());
                            let mut rpass =
                                encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                                    label: None,
                                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                                        view: &view,
                                        resolve_target: None,
                                        ops: wgpu::Operations {
                                            load: wgpu::LoadOp::Clear(wgpu::Color::GREEN),
                                            store: wgpu::StoreOp::Store,
                                        },
                                    })],
                                    depth_stencil_attachment: None,
                                    timestamp_writes: None,
                                    occlusion_query_set: None,
                                });
                            rpass.set_pipeline(&render_pipeline);
                            rpass.set_bind_group(0, &copy_bind_group, &[]);
                            rpass.draw(0..3, 0..2);
                        }
                        queue.submit(Some(encoder.finish()));
                        frame.present();
                        window_clone.request_redraw();
                    }
                    WindowEvent::Resized(s) => {
                        if s.width > 0 && s.height > 0 {
                            size = s;
                            sc.width = s.width;
                            sc.height = s.height;
                            surface.configure(&device, &sc);

                            img = device.create_texture(&wgpu::TextureDescriptor {
                                label: None,
                                size: wgpu::Extent3d {
                                    width: s.width,
                                    height: s.height,
                                    depth_or_array_layers: 1,
                                },
                                mip_level_count: 1,
                                sample_count: 1,
                                dimension: wgpu::TextureDimension::D2,
                                format: wgpu::TextureFormat::Rgba8Unorm,
                                usage: wgpu::TextureUsages::STORAGE_BINDING
                                    | wgpu::TextureUsages::TEXTURE_BINDING,
                                view_formats: &[],
                            });
                            img_view = img.create_view(&Default::default());

                            bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                                label: None,
                                layout: &bind_group_layout,
                                entries: &[
                                    wgpu::BindGroupEntry {
                                        binding: 0,
                                        resource: config_dev.as_entire_binding(),
                                    },
                                    wgpu::BindGroupEntry {
                                        binding: 1,
                                        resource: wgpu::BindingResource::TextureView(&img_view),
                                    },
                                    wgpu::BindGroupEntry {
                                        binding: 2,
                                        resource: sphere_buffer.as_entire_binding(),
                                    },
                                    wgpu::BindGroupEntry {
                                        binding: 3,
                                        resource: triangle_vertices_buffer.as_entire_binding(),
                                    },
                                    wgpu::BindGroupEntry {
                                        binding: 4,
                                        resource: gpu_triangles_buffer.as_entire_binding(),
                                    },
                                ],
                            });

                            copy_bind_group =
                                device.create_bind_group(&wgpu::BindGroupDescriptor {
                                    label: None,
                                    layout: &copy_bind_group_layout,
                                    entries: &[
                                        wgpu::BindGroupEntry {
                                            binding: 0,
                                            resource: wgpu::BindingResource::TextureView(&img_view),
                                        },
                                        wgpu::BindGroupEntry {
                                            binding: 1,
                                            resource: wgpu::BindingResource::Sampler(&sampler),
                                        },
                                    ],
                                });
                        }
                    }
                    WindowEvent::KeyboardInput {
                        event:
                            KeyEvent {
                                physical_key: PhysicalKey::Code(code),
                                state: key_state,
                                ..
                            },
                        ..
                    } => {
                        if key_state.is_pressed() {
                            if code == KeyCode::Escape && mouse_grabbed {
                                window_clone.set_cursor_grab(CursorGrabMode::None).unwrap();
                                window_clone.set_cursor_visible(true);
                                mouse_grabbed = false;
                            }
                            keys_pressed.insert(code);
                        } else {
                            keys_pressed.remove(&code);
                        }
                    }
                    WindowEvent::MouseInput { state, button, .. } => {
                        if button == winit::event::MouseButton::Left
                            && state.is_pressed()
                            && !mouse_grabbed
                        {
                            window_clone
                                .set_cursor_grab(CursorGrabMode::Confined)
                                .unwrap();
                            window_clone.set_cursor_visible(false);
                            mouse_grabbed = true;
                        }
                    }
                    WindowEvent::CloseRequested => {
                        target.exit();
                    }
                    _ => (),
                }
            } else if let Event::DeviceEvent { event, .. } = event
                && let winit::event::DeviceEvent::MouseMotion { delta } = event
                && mouse_grabbed
            {
                mouse_delta += Vec2::new(delta.0 as f32, delta.1 as f32);
                accumulated_frames = 0;
            }
        })
        .unwrap();
}

fn main() {
    let event_loop = EventLoop::new().unwrap();
    let window = WindowBuilder::new()
        .with_title("Ray Tracing")
        .with_inner_size(PhysicalSize::new(1280, 720))
        .build(&event_loop)
        .unwrap();
    pollster::block_on(run(event_loop, window));
}
