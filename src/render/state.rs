pub struct State {
    surface: wgpu::Surface<'static>, // window for rendering onto
    surface_config: wgpu::SurfaceConfiguration, // describes a Surface
    device: wgpu::Device,            // connection to GPU
    queue: wgpu::Queue,              // executes recorded CommandBuffer objects

    sampler: wgpu::Sampler, // defines how a pipeline will sample from a TextureView (like define filters)

    compute_bind_group: wgpu::BindGroup, // set of resources that can be bound to ComputePass
    render_bind_group: wgpu::BindGroup,  // set of resources that can be bound to RenderPass

    compute_bind_group_layout: wgpu::BindGroupLayout, // used to create the bind group
    render_bind_group_layout: wgpu::BindGroupLayout,  // used to create the bind group

    compute_pipeline: wgpu::ComputePipeline, // compute pipeline, for all calculations
    render_pipeline: wgpu::RenderPipeline,   // render pipeline, just for full screen triangle

    pub window: std::sync::Arc<winit::window::Window>, // represents a window
}

// private helper function, not a method inside the impl because new() calls it
// called in both State's new() and resize()
// creates storage texture for storage texture view for compute and render bind groups
fn create_bind_groups(
    device: &wgpu::Device,
    sampler: &wgpu::Sampler,
    compute_bind_group_layout: &wgpu::BindGroupLayout,
    render_bind_group_layout: &wgpu::BindGroupLayout,
    texture_size: &(u32, u32),
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
            format: wgpu::TextureFormat::Rgba16Float, // linear gamma
            usage: wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[], // linear gamma
        })
        .create_view(&wgpu::TextureViewDescriptor::default()); // linear gamma

    (
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Compute bind group"),
            layout: compute_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0, // matches with shader.wgsl @binding(0)
                resource: wgpu::BindingResource::TextureView(&storage_texture_view),
            }],
        }),
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Render bind group"),
            layout: render_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0, // matches with shader.wgsl @binding(0)
                    resource: wgpu::BindingResource::TextureView(&storage_texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1, // matches with shader.wgsl @binding(1)
                    resource: wgpu::BindingResource::Sampler(sampler),
                },
            ],
        }),
    )
}

impl State {
    pub async fn new(
        window: std::sync::Arc<winit::window::Window>,
        display_handle: winit::event_loop::OwnedDisplayHandle,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        log::info!("Called: new");

        // create instance, the context for all other wgpu objects
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            #[cfg(not(target_arch = "wasm32"))]
            backends: wgpu::Backends::PRIMARY, // Vulkan, Metal, DX12, WebGPU (no WebGL)
            #[cfg(target_arch = "wasm32")]
            backends: wgpu::Backends::BROWSER_WEBGPU, // just WebGPU
            ..wgpu::InstanceDescriptor::new_with_display_handle(Box::new(display_handle))
        });

        // create surface, which targets the given winit window
        let surface = instance.create_surface(std::sync::Arc::clone(&window))?;

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await?;
        log::info!("Using adapter: {}", adapter.get_info().name); // doesn't log the name on web for some reason, but logs fine on native
        let supported_limits = adapter.limits(); // get the maximum limits the physical hardware supports
        log::info!(
            "Largest possible storage buffer binding size: {} MiB",
            supported_limits.max_storage_buffer_binding_size as f32 / 1024.0 / 1024.0
        );
        let required_limits = wgpu::Limits {
            max_storage_buffer_binding_size: supported_limits.max_storage_buffer_binding_size, // the default is 128 MiB, which is too small for millions of triangles
            max_buffer_size: supported_limits.max_buffer_size,
            ..wgpu::Limits::default()
        };
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: None,
                required_features: wgpu::Features::empty(),
                required_limits,
                experimental_features: unsafe { wgpu::ExperimentalFeatures::enabled() },
                memory_hints: wgpu::MemoryHints::Performance,
                trace: wgpu::Trace::Off,
            })
            .await?;

        // see which TextureFormat's are supported
        // Bgra8Unorm and Bgra8UnormSrgb should be guaranteed, but on web, Bgra8UnormSrgb isn't supported it seems
        log::info!(
            "Surface formats: {:?}",
            surface.get_capabilities(&adapter).formats
        );

        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: wgpu::TextureFormat::Bgra8Unorm, // linear gamma. Bgra8Unorm should be a guaranteed supported format
            // sometimes on web initial page load, the canvas can have window.inner_size().width of 0
            // 0 length or width causes surface.configure() to panic
            // so, .max(1) makes sure that width and height are never less than 1
            width: window.inner_size().width.max(1),
            height: window.inner_size().height.max(1),
            present_mode: wgpu::PresentMode::AutoVsync,
            desired_maximum_frame_latency: 2,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            view_formats: vec![], // linear gamma
        };

        // render() only works when the surface is configured
        // render() is often called right after new(), and resize() isn't called unless a resize happens, so configuring surface here is necessary
        surface.configure(&device, &surface_config);

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            ..Default::default()
        });

        // define compute and render bind group layouts
        // these are only defined once here and do not change, but are used in many places
        let compute_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Compute bind group layout"),
                entries: &[
                    // storage texture
                    wgpu::BindGroupLayoutEntry {
                        binding: 0, // matches with shader.wgsl @binding(0)
                        // which stages can see this binding
                        // even though both render and compute bind group layouts have entries with binding 0 (and group 0), this visibility distinguishes them
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::StorageTexture {
                            access: wgpu::StorageTextureAccess::WriteOnly,
                            format: wgpu::TextureFormat::Rgba16Float, // linear gamma
                            view_dimension: wgpu::TextureViewDimension::D2,
                        },
                        count: None,
                    },
                ],
            });
        let render_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Render bind group layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,                               // matches with shader.wgsl @binding(0)
                        visibility: wgpu::ShaderStages::FRAGMENT, // which stages can see this binding
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,                               // matches with shader.wgsl @binding(1)
                        visibility: wgpu::ShaderStages::FRAGMENT, // which stages can see this binding
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
            });

        // create a shader module from shader.wgsl
        // used for everything: compute, vertex, and fragment
        let shader = device.create_shader_module(wgpu::include_wgsl!("shader.wgsl"));

        // configure compute and render pipelines with the bind group layouts and the shader module
        let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Compute pipeline"),
            layout: Some(
                &device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Compute pipeline layout"),
                    bind_group_layouts: &[Some(&compute_bind_group_layout)],
                    immediate_size: 0,
                }),
            ),
            module: &shader,
            entry_point: Some("compute_main"), // function name in shader.wgsl
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
                entry_point: Some("vs_main"), // function name in shader.wgsl
                compilation_options: Default::default(),
                buffers: &[],
            },
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList, // for ex: Vertices 0 1 2 3 4 5 create two triangles 0 1 2 and 3 4 5
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw, // ccw are front-face; right-handed coordinate system
                cull_mode: Some(wgpu::Face::Back),
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"), // function name in shader.wgsl
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_config.format, // linear gamma
                    blend: Some(wgpu::BlendState {
                        color: wgpu::BlendComponent::REPLACE,
                        alpha: wgpu::BlendComponent::REPLACE,
                    }),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview_mask: None,
            cache: None,
        });

        // use helper function to create bind groups
        let (compute_bind_group, render_bind_group) = create_bind_groups(
            &device,
            &sampler,
            &compute_bind_group_layout,
            &render_bind_group_layout,
            &(surface_config.width, surface_config.height),
        );

        Ok(Self {
            surface,
            surface_config,
            device,
            queue,

            sampler,

            compute_bind_group,
            render_bind_group,

            compute_bind_group_layout,
            render_bind_group_layout,

            compute_pipeline,
            render_pipeline,

            window,
        })
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        log::info!("Called: resize {width}x{height}");

        if width > 0 && height > 0 {
            self.surface_config.width = width;
            self.surface_config.height = height;
            self.surface.configure(&self.device, &self.surface_config);

            // recreate bind groups with size of new storage texture matching new surface size
            // recreating bind groups is necessary for a resize since storage texture size must match the surface size
            (self.compute_bind_group, self.render_bind_group) = create_bind_groups(
                &self.device,
                &self.sampler,
                &self.compute_bind_group_layout,
                &self.render_bind_group_layout,
                &(width, height),
            );

            // on initial window creation on MacOS, and sometimes on initial web page load, even though resize is called, render isn't called afterwards
            // so, force a render call here
            self.window.request_redraw();
        }
    }

    pub fn update(&mut self) {
        match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(frame) => {
                let view = frame
                    .texture
                    .create_view(&wgpu::TextureViewDescriptor::default()); // linear gamma

                // encoder can record RenderPasses, ComputePasses, and transfer operations between driver-managed resources like Buffers and Textures
                // when finished recording, CommandEncoder::finish is called to obtain a CommandBuffer which is submitted for execution
                let mut encoder =
                    self.device
                        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                            label: Some("Encoder"),
                        });

                // compute and render passes
                {
                    // this is in a code block because begin_compute_pass() takes a &mut to encoder
                    let mut compute_pass =
                        encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                            label: Some("Compute pass"),
                            timestamp_writes: None,
                        });

                    compute_pass.set_pipeline(&self.compute_pipeline);
                    compute_pass.set_bind_group(0, &self.compute_bind_group, &[]); // the u32 passed here, which is 0, matches with @group(0) in shader.wgsl

                    let workgroup_size = 8; // matches with @compute @workgroup_size(8, 8, 1) in shader.wgsl
                    let workgroup_count_x = self.surface_config.width.div_ceil(workgroup_size); // make sure that the entire texture is covered by 8x8 workgroups, since texture size should always equal surface_config size
                    let workgroup_count_y = self.surface_config.height.div_ceil(workgroup_size);
                    compute_pass.dispatch_workgroups(workgroup_count_x, workgroup_count_y, 1);
                }
                {
                    // this is in a code block because begin_compute_pass() takes a &mut to encoder
                    let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: Some("Render pass"),
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: &view,
                            depth_slice: None, // only useful for 3D textures
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Load,
                                store: wgpu::StoreOp::Store,
                            },
                        })],
                        depth_stencil_attachment: None,
                        timestamp_writes: None,
                        occlusion_query_set: None,
                        multiview_mask: None,
                    });

                    render_pass.set_pipeline(&self.render_pipeline);
                    render_pass.set_bind_group(0, &self.render_bind_group, &[]); // the u32 passed here, which is 0, matches with the @group(0) in shader.wgsl
                    render_pass.draw(0..3, 0..1); // draw a triangle
                }

                self.queue.submit([encoder.finish()]); // CommandEncoder::finish and executed here
                // although everything seems to work without pre_present_notify(), this is encouraged by winit docs
                // might only matter on Wayland
                self.window.pre_present_notify();
                frame.present();
            }

            wgpu::CurrentSurfaceTexture::Timeout | wgpu::CurrentSurfaceTexture::Occluded => {}
            wgpu::CurrentSurfaceTexture::Outdated
            | wgpu::CurrentSurfaceTexture::Suboptimal(_)
            | wgpu::CurrentSurfaceTexture::Lost => {
                // On Windows, fast resizes can cause Outdated error
                let size = self.window.inner_size();
                self.resize(size.width, size.height);
            }
            wgpu::CurrentSurfaceTexture::Validation => {
                log::error!("Validation error in get_current_texture");
            }
        }

        // this will trigger RedrawRequested event, which is a call to self.update() again, which creates a loop at the vsync rate of the monitor
        // self.window.request_redraw();
    }

    pub fn key_event(&mut self, key_event: &winit::event::KeyEvent) {
        ()
    }

    pub fn mouse_move_event(&mut self, delta: (f64, f64)) {
        ()
    }

    pub fn mouse_button_event(
        &mut self,
        state: winit::event::ElementState,
        button: winit::event::MouseButton,
    ) {
        ()
    }
}
