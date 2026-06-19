use std::sync::Arc;

use glfw::{PWindow, log_errors};
use vulkano::{
    Validated, Version,
    VulkanError::{self},
    VulkanLibrary,
    buffer::{Buffer, BufferContents, BufferCreateInfo, BufferUsage, Subbuffer},
    command_buffer::{
        AutoCommandBufferBuilder, CommandBufferUsage, RenderingAttachmentInfo, RenderingInfo,
        allocator::StandardCommandBufferAllocator,
    },
    device::{
        Device, DeviceCreateInfo, DeviceExtensions, DeviceFeatures, Queue, QueueCreateInfo,
        QueueFlags, physical::PhysicalDeviceType,
    },
    format::Format,
    image::{Image, ImageUsage, view::ImageView},
    instance::{Instance, InstanceCreateInfo},
    memory::allocator::{AllocationCreateInfo, MemoryTypeFilter, StandardMemoryAllocator},
    pipeline::{
        DynamicState, GraphicsPipeline, PipelineLayout, PipelineShaderStageCreateInfo,
        graphics::{
            GraphicsPipelineCreateInfo,
            color_blend::{ColorBlendAttachmentState, ColorBlendState},
            input_assembly::InputAssemblyState,
            multisample::MultisampleState,
            rasterization::RasterizationState,
            subpass::PipelineRenderingCreateInfo,
            vertex_input::{Vertex, VertexDefinition},
            viewport::{Viewport, ViewportState},
        },
        layout::PipelineDescriptorSetLayoutCreateInfo,
    },
    render_pass::{AttachmentLoadOp, AttachmentStoreOp},
    swapchain::{
        Surface, Swapchain, SwapchainCreateInfo, SwapchainPresentInfo, acquire_next_image,
    },
    sync::{self, GpuFuture},
};

struct App {
    instance: Arc<Instance>,
    device: Arc<Device>,
    queue: Arc<Queue>,
    command_buffer_allocator: Arc<StandardCommandBufferAllocator>,
    vertex_buffer: Subbuffer<[MyVertex]>,
    rcx: Option<RenderContext>,
}

struct RenderContext {
    window: Arc<PWindow>,
    swapchain: Arc<Swapchain>,
    attachment_image_views: Vec<Arc<ImageView>>,
    pipeline: Arc<GraphicsPipeline>,
    viewport: Viewport,
    recreate_swapchain: bool,
    previous_frame_end: Option<Box<dyn GpuFuture>>,
}

fn main() {
    let mut glfw = glfw::init(glfw::log_errors!()).unwrap();
    glfw.window_hint(glfw::WindowHint::ClientApi(glfw::ClientApiHint::NoApi));
    let (mut pwindow, events) = glfw
        .create_window(640, 480, "Glass Turtle", glfw::WindowMode::Windowed)
        .unwrap();

    pwindow.set_framebuffer_size_polling(true);
    pwindow.set_close_polling(true);

    let window = Arc::new(pwindow);

    let mut app = App::new(&window);
    app.init_render_context(window.clone());

    let mut close_requested = false;

    while !window.should_close() && !close_requested {
        glfw.wait_events_timeout(1.0 / 60.0);

        for (_, events) in glfw::flush_messages(&events) {
            match events {
                glfw::WindowEvent::FramebufferSize(_, _) => {
                    if let Some(rcx) = app.rcx.as_mut() {
                        rcx.recreate_swapchain = true;
                    }
                }
                glfw::WindowEvent::Close => {
                    close_requested = true;
                }
                _ => {}
            }
        }

        app.redraw();
    }
}

impl App {
    fn new(window: &Arc<PWindow>) -> Self {
        let window_extensions = Surface::required_extensions(window.as_ref())
            .expect("Failed to get required extensions");

        let library = VulkanLibrary::new()
            .unwrap_or_else(|err| panic!("Couldn't load Vulkan library: {:?}", err));

        let instance_create_info = InstanceCreateInfo {
            engine_name: Some("Glass Turtle Graphics".into()),
            engine_version: Version::V1_0,
            max_api_version: Some(Version::V1_4),
            enabled_layers: vec!["VK_LAYER_KHRONOS_validation".to_owned()],
            enabled_extensions: window_extensions,
            ..InstanceCreateInfo::application_from_cargo_toml()
        };

        let instance = Instance::new(library, instance_create_info)
            .unwrap_or_else(|err| panic!("Couldn't create instance: {:?}", err));

        let mut device_extensions = DeviceExtensions {
            khr_swapchain: true,
            ..DeviceExtensions::empty()
        };

        let (physical_device, queue_family_index) = instance
            .enumerate_physical_devices()
            .unwrap()
            .filter(|p| {
                p.api_version() >= Version::V1_3 || p.supported_extensions().khr_dynamic_rendering
            })
            .filter(|p| p.supported_extensions().contains(&device_extensions))
            .filter_map(|p| {
                p.queue_family_properties()
                    .iter()
                    .enumerate()
                    .position(|(i, q)| {
                        q.queue_flags.intersects(QueueFlags::GRAPHICS)
                            && p.presentation_support(i as u32, window.as_ref()).unwrap()
                    })
                    .map(|i| (p, i as u32))
            })
            .min_by_key(|(p, _)| match p.properties().device_type {
                PhysicalDeviceType::DiscreteGpu => 0,
                PhysicalDeviceType::IntegratedGpu => 1,
                PhysicalDeviceType::VirtualGpu => 2,
                PhysicalDeviceType::Cpu => 3,
                PhysicalDeviceType::Other => 4,
                _ => 5,
            })
            .expect("no suitable physical device found");
        println!(
            "Using device: {} (type: {:?})",
            physical_device.properties().device_name,
            physical_device.properties().device_type,
        );

        if physical_device.api_version() < Version::V1_3 {
            device_extensions.khr_dynamic_rendering = true;
        }

        let queue_create_info = QueueCreateInfo {
            queue_family_index,
            ..Default::default()
        };

        let (device, mut queues) = match Device::new(
            physical_device,
            DeviceCreateInfo {
                queue_create_infos: vec![queue_create_info],
                enabled_extensions: device_extensions,
                enabled_features: DeviceFeatures {
                    dynamic_rendering: true,
                    ..DeviceFeatures::empty()
                },
                ..Default::default()
            },
        ) {
            Ok(d) => d,
            Err(err) => panic!("Could not build device: {:?}", err),
        };

        let queue = queues.next().unwrap();

        let memory_allocator = Arc::new(StandardMemoryAllocator::new_default(device.clone()));

        let command_buffer_allocator = Arc::new(StandardCommandBufferAllocator::new(
            device.clone(),
            Default::default(),
        ));

        // We now create a buffer that will store the shape of our triangle.
        let vertices = [
            MyVertex {
                position: [-0.5, -0.25],
            },
            MyVertex {
                position: [0.0, 0.5],
            },
            MyVertex {
                position: [0.25, -0.1],
            },
        ];
        let vertex_buffer = Buffer::from_iter(
            memory_allocator,
            BufferCreateInfo {
                usage: BufferUsage::VERTEX_BUFFER,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                    | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
            vertices,
        )
        .unwrap();

        App {
            instance,
            device,
            queue,
            command_buffer_allocator,
            vertex_buffer,
            rcx: None,
        }
    }

    fn init_render_context(&mut self, window: Arc<PWindow>) {
        let surface = Surface::from_window(self.instance.clone(), window.clone()).unwrap();
        let (width, height) = window.get_framebuffer_size();
        let image_extent = [width as u32, height as u32];
        let view_extent = [width as f32, height as f32];

        let surface_capabilities = self
            .device
            .physical_device()
            .surface_capabilities(&surface, Default::default())
            .unwrap();

        let (image_format, _) = self
            .device
            .physical_device()
            .surface_formats(&surface, Default::default())
            .unwrap()[0];

        let desired_image_count = surface_capabilities.min_image_count.max(2);
        let min_image_count = surface_capabilities
            .max_image_count
            .map_or(desired_image_count, |maximum| {
                desired_image_count.min(maximum)
            });
        let swapchain_create_info = SwapchainCreateInfo {
            image_format,
            min_image_count,
            image_extent: image_extent,
            image_usage: ImageUsage::COLOR_ATTACHMENT,
            composite_alpha: surface_capabilities
                .supported_composite_alpha
                .into_iter()
                .next()
                .unwrap(),
            ..Default::default()
        };

        let (swapchain, images) =
            Swapchain::new(self.device.clone(), surface, swapchain_create_info).unwrap();

        let viewport = Viewport {
            offset: [0.0, 0.0],
            extent: view_extent,
            depth_range: 0.0..=1.0,
        };

        let attachment_image_views = window_size_dependent_setup(&images);
        let pipeline = self.create_pipeline(image_format);
        let recreate_swapchain = false;
        let previous_frame_end = Some(sync::now(self.device.clone()).boxed());

        self.rcx = Some(RenderContext {
            window,
            swapchain,
            attachment_image_views,
            pipeline,
            viewport,
            recreate_swapchain,
            previous_frame_end,
        });
    }

    fn redraw(&mut self) {
        let Some(mut rcx) = self.rcx.take() else {
            return;
        };
        let (width, height) = rcx.window.get_framebuffer_size();
        let image_extent = [width as u32, height as u32];
        let view_extent = [width as f32, height as f32];

        if width == 0 || height == 0 {
            self.rcx = Some(rcx);
            return;
        }

        rcx.previous_frame_end.as_mut().unwrap().cleanup_finished();

        if rcx.recreate_swapchain {
            let (new_swapchain, new_images) = rcx
                .swapchain
                .recreate(SwapchainCreateInfo {
                    image_extent,
                    ..rcx.swapchain.create_info()
                })
                .expect("failed to recreate swapchain");

            let format_changed = new_swapchain.image_format() != rcx.swapchain.image_format();

            rcx.swapchain = new_swapchain;

            rcx.attachment_image_views = window_size_dependent_setup(&new_images);

            if format_changed {
                rcx.pipeline = self.create_pipeline(rcx.swapchain.image_format());
            }
            rcx.viewport.extent = view_extent;

            rcx.recreate_swapchain = false;
        }

        let (image_index, suboptimal, acquire_future) =
            match acquire_next_image(rcx.swapchain.clone(), None).map_err(Validated::unwrap) {
                Ok(r) => r,
                Err(VulkanError::OutOfDate) => {
                    rcx.recreate_swapchain = true;
                    self.rcx = Some(rcx);
                    return;
                }
                Err(e) => panic!("failed to acquire next image: {e}"),
            };

        if suboptimal {
            rcx.recreate_swapchain = true;
        }

        let mut builder = AutoCommandBufferBuilder::primary(
            self.command_buffer_allocator.clone(),
            self.queue.queue_family_index(),
            CommandBufferUsage::OneTimeSubmit,
        )
        .unwrap();

        builder
            // Before we can draw, we have to *enter a render pass*. We specify which
            // attachments we are going to use for rendering here, which needs to match
            // what was previously specified when creating the pipeline.
            .begin_rendering(RenderingInfo {
                color_attachments: vec![Some(RenderingAttachmentInfo {
                    load_op: AttachmentLoadOp::Clear,
                    store_op: AttachmentStoreOp::Store,
                    clear_value: Some([0.0, 0.0, 1.0, 1.0].into()),
                    ..RenderingAttachmentInfo::image_view(
                        // We specify image view corresponding to the currently acquired
                        // swapchain image, to use for this attachment.
                        rcx.attachment_image_views[image_index as usize].clone(),
                    )
                })],
                ..Default::default()
            })
            .unwrap()
            // We are now inside the first subpass of the render pass.
            //
            // TODO: Document state setting and how it affects subsequent draw commands.
            .set_viewport(0, [rcx.viewport.clone()].into_iter().collect())
            .unwrap()
            .bind_pipeline_graphics(rcx.pipeline.clone())
            .unwrap()
            .bind_vertex_buffers(0, self.vertex_buffer.clone())
            .unwrap();

        unsafe { builder.draw(self.vertex_buffer.len() as u32, 1, 0, 0) }.unwrap();

        builder
            // We leave the render pass.
            .end_rendering()
            .unwrap();

        // Finish recording the command buffer by calling `end`.
        let command_buffer = builder.build().unwrap();
        let future = rcx
            .previous_frame_end
            .take()
            .unwrap()
            .join(acquire_future)
            .then_execute(self.queue.clone(), command_buffer)
            .unwrap()
            // The color output is now expected to contain our triangle. But in order to
            // show it on the screen, we have to *present* the image by calling
            // `then_swapchain_present`.
            //
            // This function does not actually present the image immediately. Instead it
            // submits a present command at the end of the queue. This means that it will
            // only be presented once the GPU has finished executing the command buffer
            // that draws the triangle.
            .then_swapchain_present(
                self.queue.clone(),
                SwapchainPresentInfo::swapchain_image_index(rcx.swapchain.clone(), image_index),
            )
            .then_signal_fence_and_flush();

        match future.map_err(Validated::unwrap) {
            Ok(future) => {
                rcx.previous_frame_end = Some(future.boxed());
            }
            Err(VulkanError::OutOfDate) => {
                rcx.recreate_swapchain = true;
                rcx.previous_frame_end = Some(sync::now(self.device.clone()).boxed());
            }
            Err(e) => {
                println!("failed to flush future: {e}");
                rcx.previous_frame_end = Some(sync::now(self.device.clone()).boxed());
            }
        }
        self.rcx = Some(rcx);
    }

    fn create_pipeline(&self, image_format: Format) -> Arc<GraphicsPipeline> {
        mod vs {
            vulkano_shaders::shader! {
                ty: "vertex",
                src: r"
                                #version 450

                                layout(location = 0) in vec2 position;

                                void main() {
                                    gl_Position = vec4(position, 0.0, 1.0);
                                }
                            ",
            }
        }

        mod fs {
            vulkano_shaders::shader! {
                ty: "fragment",
                src: r"
                                #version 450

                                layout(location = 0) out vec4 f_color;

                                void main() {
                                    f_color = vec4(251.0 / 255.0, 113.0 / 255.0, 133.0 / 255.0, 1.0);
                                }
                            ",
            }
        }

        {
            let vs = vs::load(self.device.clone())
                .unwrap()
                .entry_point("main")
                .unwrap();
            let fs = fs::load(self.device.clone())
                .unwrap()
                .entry_point("main")
                .unwrap();

            let vertex_input_state = MyVertex::per_vertex().definition(&vs).unwrap();

            let stages = [
                PipelineShaderStageCreateInfo::new(vs),
                PipelineShaderStageCreateInfo::new(fs),
            ];

            let layout = PipelineLayout::new(
                self.device.clone(),
                PipelineDescriptorSetLayoutCreateInfo::from_stages(stages.iter())
                    .into_pipeline_layout_create_info(self.device.clone())
                    .unwrap(),
            )
            .unwrap();

            let subpass = PipelineRenderingCreateInfo {
                color_attachment_formats: vec![Some(image_format)],
                ..Default::default()
            };

            GraphicsPipeline::new(
                self.device.clone(),
                None,
                GraphicsPipelineCreateInfo {
                    stages: stages.into_iter().collect(),
                    vertex_input_state: Some(vertex_input_state),
                    input_assembly_state: Some(InputAssemblyState::default()),
                    viewport_state: Some(ViewportState::default()),
                    rasterization_state: Some(RasterizationState::default()),
                    multisample_state: Some(MultisampleState::default()),
                    color_blend_state: Some(ColorBlendState {
                        attachments: vec![ColorBlendAttachmentState::default()],
                        ..Default::default()
                    }),
                    dynamic_state: [DynamicState::Viewport].into_iter().collect(),

                    subpass: Some(subpass.clone().into()),
                    ..GraphicsPipelineCreateInfo::layout(layout)
                },
            )
            .unwrap()
        }
    }
}

// We use `#[repr(C)]` here to force rustc to use a defined layout for our data as the default
// representation has *no guarantees*.
#[derive(Clone, Copy, BufferContents, Vertex)]
#[repr(C)]
struct MyVertex {
    // We need to set a GPU compatible format for each vertex attribute.
    #[format(R32G32_SFLOAT)]
    position: [f32; 2],
}

// This function is called once during initialization, then again whenever the window is resized.
fn window_size_dependent_setup(images: &[Arc<Image>]) -> Vec<Arc<ImageView>> {
    images
        .iter()
        .map(|image| ImageView::new_default(image.to_owned()).unwrap())
        .collect::<Vec<_>>()
}
