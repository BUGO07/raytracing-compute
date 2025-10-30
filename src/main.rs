use std::sync::Arc;

use glam::Vec3;
use wgpu::util::DeviceExt;

use winit::{
    event::{Event, WindowEvent},
    event_loop::EventLoop,
    window::Window,
};

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct IParams {
    camera_pos: Vec3,
    _pad1: u32,
    light_dir: Vec3,
    _pad2: u32,
    width: u32,
    height: u32,
    i_time: f32,
    sphere_count: u32,
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct Sphere {
    position: Vec3,
    radius: f32,
    material: Material,
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct Material {
    diffuse_color: Vec3,
    smoothness: f32,
    emission_color: Vec3,
    emission_strength: f32,
}

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
        .request_device(&Default::default(), None)
        .await
        .expect("error creating device");
    let size = window.inner_size();
    let swapchain_capabilities = surface.get_capabilities(&adapter);
    let format = swapchain_capabilities.formats[0];
    let mut sc = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format,
        width: size.width,
        present_mode: wgpu::PresentMode::AutoVsync,
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

    let img = device.create_texture(&wgpu::TextureDescriptor {
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
    let img_view = img.create_view(&Default::default());

    const CONFIG_SIZE: u64 = size_of::<IParams>() as u64;

    let config_dev = device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size: CONFIG_SIZE,
        usage: wgpu::BufferUsages::COPY_DST
            | wgpu::BufferUsages::STORAGE
            | wgpu::BufferUsages::UNIFORM,
        mapped_at_creation: false,
    });
    let config_resource = config_dev.as_entire_binding();

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
                smoothness: 0.95,
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

    let sphere_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Sphere Buffer"),
        contents: bytemuck::cast_slice(&spheres),
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
                    access: wgpu::StorageTextureAccess::WriteOnly,
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
    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: None,
        layout: &bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: config_resource,
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::TextureView(&img_view),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: sphere_buffer.as_entire_binding(),
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
    let copy_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
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
    let start_time = std::time::Instant::now();

    event_loop
        .run(move |event, target| {
            if let Event::WindowEvent {
                window_id: _,
                event,
            } = event
            {
                match event {
                    WindowEvent::RedrawRequested => {
                        let frame = surface
                            .get_current_texture()
                            .expect("error getting texture from swap chain");

                        let i_time: f32 = 0.5 + start_time.elapsed().as_micros() as f32 * 1e-6;
                        let config_data = IParams {
                            camera_pos: Vec3::new(0.0, 0.0, 5.0),
                            _pad1: 0,
                            light_dir: Vec3::new(0.2, 1.0, 0.05).normalize(),
                            _pad2: 0,
                            width: size.width,
                            height: size.height,
                            i_time,
                            sphere_count: spheres.len() as u32,
                        };
                        let config_host =
                            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                                label: None,
                                contents: bytemuck::cast_slice(&[config_data]),
                                usage: wgpu::BufferUsages::COPY_SRC,
                            });
                        let mut encoder = device.create_command_encoder(&Default::default());
                        encoder.copy_buffer_to_buffer(&config_host, 0, &config_dev, 0, CONFIG_SIZE);
                        {
                            let mut cpass = encoder.begin_compute_pass(&Default::default());
                            cpass.set_pipeline(&pipeline);
                            cpass.set_bind_group(0, &bind_group, &[]);
                            cpass.dispatch_workgroups(size.width / 16, size.height / 16, 1);
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
                    WindowEvent::Resized(size) => {
                        if size.width > 0 && size.height > 0 {
                            sc.width = size.width;
                            sc.height = size.height;
                            surface.configure(&device, &sc);
                        }
                    }
                    WindowEvent::CloseRequested => {
                        target.exit();
                    }
                    _ => (),
                }
            }
        })
        .unwrap();
}

fn main() {
    let event_loop = EventLoop::new().unwrap();
    let window = Window::new(&event_loop).unwrap();
    window.set_title("Ray Tracing");
    pollster::block_on(run(event_loop, window));
}
