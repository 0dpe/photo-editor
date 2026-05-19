use eframe::egui_wgpu::{self, wgpu};

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Config {
    pub pan: glam::Vec2,
    pub zoom: f32,
    pub _pad: u32,
    pub image_size: glam::Vec2,
    pub _pad2: glam::Vec2,
}

impl Config {
    pub fn default_view(image_size: glam::Vec2) -> Self {
        Self {
            pan: glam::Vec2::ZERO,
            zoom: 1.0,
            _pad: 0,
            image_size,
            _pad2: glam::Vec2::ZERO,
        }
    }

    /// Scale and center so the full image is visible in the viewport.
    pub fn fit_to_viewport(&mut self, viewport: glam::Vec2) {
        if viewport.x < 1.0 || viewport.y < 1.0 {
            return;
        }
        if self.image_size.x < 1.0 || self.image_size.y < 1.0 {
            return;
        }
        self.zoom = (viewport.x / self.image_size.x).min(viewport.y / self.image_size.y);
        self.pan = glam::Vec2::ZERO;
    }
}

pub struct ImageRenderResources {
    compute_bind_group: wgpu::BindGroup,
    render_bind_group: wgpu::BindGroup,

    compute_bind_group_layout: wgpu::BindGroupLayout,
    render_bind_group_layout: wgpu::BindGroupLayout,

    compute_pipeline: wgpu::ComputePipeline,
    render_pipeline: wgpu::RenderPipeline,

    image_bind_group: wgpu::BindGroup,
    config_buffer: wgpu::Buffer,

    viewport_size: (u32, u32),
    image_size: glam::Vec2,
}

fn create_storage_bind_groups(
    device: &wgpu::Device,
    compute_bind_group_layout: &wgpu::BindGroupLayout,
    render_bind_group_layout: &wgpu::BindGroupLayout,
    texture_size: (u32, u32),
) -> (wgpu::BindGroup, wgpu::BindGroup) {
    let storage_texture_view = device
        .create_texture(&wgpu::TextureDescriptor {
            label: Some("Storage texture"),
            size: wgpu::Extent3d {
                width: texture_size.0,
                height: texture_size.1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba16Float,
            usage: wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[wgpu::TextureFormat::Rgba16Float],
        })
        .create_view(&wgpu::TextureViewDescriptor {
            label: Some("Storage texture view"),
            format: Some(wgpu::TextureFormat::Rgba16Float),
            dimension: Some(wgpu::TextureViewDimension::D2),
            aspect: wgpu::TextureAspect::All,
            base_mip_level: 0,
            mip_level_count: Some(1),
            ..Default::default()
        });

    (
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Compute bind group"),
            layout: compute_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&storage_texture_view),
            }],
        }),
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Render bind group"),
            layout: render_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&storage_texture_view),
            }],
        }),
    )
}

fn load_image() -> image::RgbaImage {
    #[cfg(target_arch = "wasm32")]
    {
        image::load_from_memory(include_bytes!("../../photo_small.jpg"))
            .expect("Failed to decode embedded photo.jpg")
            .into_rgba8()
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        image::open("photo_small.jpg")
            .expect("Failed to load photo.jpg from project root")
            .into_rgba8()
    }
}

impl ImageRenderResources {
    pub fn new(wgpu_render_state: &egui_wgpu::RenderState) -> Self {
        let device = &wgpu_render_state.device;
        let queue = &wgpu_render_state.queue;
        let target_format = wgpu_render_state.target_format;

        let img = load_image();
        let dimensions = img.dimensions();
        let image_size = glam::Vec2::new(dimensions.0 as f32, dimensions.1 as f32);

        let image_texture_size = wgpu::Extent3d {
            width: dimensions.0,
            height: dimensions.1,
            depth_or_array_layers: 1,
        };

        let image_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Image texture"),
            size: image_texture_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &image_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &img,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * dimensions.0),
                rows_per_image: Some(dimensions.1),
            },
            image_texture_size,
        );

        let config = Config::default_view(image_size);

        let config_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Config buffer"),
            size: std::mem::size_of::<Config>() as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        queue.write_buffer(&config_buffer, 0, bytemuck::cast_slice(&[config]));

        let image_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Image bind group layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
            });

        let image_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Image Sampler"),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Linear,
            ..Default::default()
        });

        let image_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Image bind group"),
            layout: &image_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(
                        &image_texture.create_view(&wgpu::TextureViewDescriptor::default()),
                    ),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&image_sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: config_buffer.as_entire_binding(),
                },
            ],
        });

        let compute_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Compute bind group layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::StorageTexture {
                        access: wgpu::StorageTextureAccess::WriteOnly,
                        format: wgpu::TextureFormat::Rgba16Float,
                        view_dimension: wgpu::TextureViewDimension::D2,
                    },
                    count: None,
                }],
            });

        let render_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Render bind group layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                }],
            });

        let shader = device.create_shader_module(wgpu::include_wgsl!("shader.wgsl"));

        let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Compute pipeline"),
            layout: Some(
                &device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Compute pipeline layout"),
                    bind_group_layouts: &[
                        Some(&compute_bind_group_layout),
                        Some(&image_bind_group_layout),
                    ],
                    immediate_size: 0,
                }),
            ),
            module: &shader,
            entry_point: Some("compute_main"),
            compilation_options: Default::default(),
            cache: None,
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render pipeline"),
            layout: Some(
                &device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Render pipeline layout"),
                    bind_group_layouts: &[Some(&render_bind_group_layout)],
                    immediate_size: 0,
                }),
            ),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                compilation_options: Default::default(),
                buffers: &[],
            },
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: target_format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview_mask: None,
            cache: None,
        });

        let viewport_size = (1, 1);
        let (compute_bind_group, render_bind_group) = create_storage_bind_groups(
            device,
            &compute_bind_group_layout,
            &render_bind_group_layout,
            viewport_size,
        );

        Self {
            compute_bind_group,
            render_bind_group,
            compute_bind_group_layout,
            render_bind_group_layout,
            compute_pipeline,
            render_pipeline,
            image_bind_group,
            config_buffer,
            viewport_size,
            image_size,
        }
    }

    pub fn image_size(&self) -> glam::Vec2 {
        self.image_size
    }

    fn ensure_viewport_size(&mut self, device: &wgpu::Device, size: (u32, u32)) {
        let size = (size.0.max(1), size.1.max(1));
        if size == self.viewport_size {
            return;
        }
        self.viewport_size = size;
        (self.compute_bind_group, self.render_bind_group) = create_storage_bind_groups(
            device,
            &self.compute_bind_group_layout,
            &self.render_bind_group_layout,
            size,
        );
    }

    pub fn prepare(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        config: Config,
        viewport_size: (u32, u32),
    ) {
        self.ensure_viewport_size(device, viewport_size);
        queue.write_buffer(&self.config_buffer, 0, bytemuck::cast_slice(&[config]));

        let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("Image compute pass"),
            timestamp_writes: None,
        });
        compute_pass.set_pipeline(&self.compute_pipeline);
        compute_pass.set_bind_group(0, &self.compute_bind_group, &[]);
        compute_pass.set_bind_group(1, &self.image_bind_group, &[]);

        let workgroup_size = 8;
        let workgroup_count_x = self.viewport_size.0.div_ceil(workgroup_size);
        let workgroup_count_y = self.viewport_size.1.div_ceil(workgroup_size);
        compute_pass.dispatch_workgroups(workgroup_count_x, workgroup_count_y, 1);
    }

    pub fn paint(&self, render_pass: &mut wgpu::RenderPass<'_>) {
        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_bind_group(0, &self.render_bind_group, &[]);
        render_pass.draw(0..3, 0..1);
    }
}
