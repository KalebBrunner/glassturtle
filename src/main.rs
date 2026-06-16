use std::sync::Arc;

use glfw::{PWindow, log_errors};
use vulkano::{
    Validated, Version,
    VulkanError::{self},
    VulkanLibrary,
    buffer::{Buffer, BufferContents, BufferCreateInfo, BufferUsage},
    device::{
        Device, DeviceCreateInfo, DeviceExtensions, DeviceFeatures, Queue, QueueCreateInfo,
        QueueFlags, physical::PhysicalDeviceType,
    },
    format::Format,
    image::ImageUsage,
    instance::{Instance, InstanceCreateInfo},
    memory::allocator::{AllocationCreateInfo, DeviceLayout, MemoryTypeFilter},
    pipeline::{
        DynamicState, GraphicsPipeline, PipelineLayout, PipelineShaderStageCreateInfo,
        graphics::{
            GraphicsPipelineCreateInfo,
            color_blend::{ColorBlendAttachmentState, ColorBlendState},
            input_assembly::InputAssemblyState,
            multisample::MultisampleState,
            rasterization::RasterizationState,
            vertex_input::{Vertex, VertexDefinition},
            viewport::{Viewport, ViewportState},
        },
        layout::PipelineDescriptorSetLayoutCreateInfo,
    },
    render_pass::Subpass,
    swapchain::{Surface, Swapchain, SwapchainCreateInfo},
};
use vulkano_taskgraph::{
    ClearValues, Id, QueueFamilyType, Task, TaskContext,
    command_buffer::RecordingCommandBuffer,
    graph::{AttachmentInfo, CompileInfo, ExecutableTaskGraph, ExecuteError, TaskGraph},
    resource::{AccessTypes, Flight, HostAccessType, ImageLayoutType, Resources},
    resource_map,
};

const MAX_FRAMES_IN_FLIGHT: u32 = 2;
const MIN_SWAPCHAIN_IMAGES: u32 = MAX_FRAMES_IN_FLIGHT + 1;
struct App {
    instance: Arc<Instance>,
    device: Arc<Device>,
    queue: Arc<Queue>,
    resources: Arc<Resources>,
    flight_id: Id<Flight>,
    rcx: Option<RenderContext>,
}

struct RenderContext {
    window: Arc<PWindow>,
    swapchain_id: Id<Swapchain>,
    viewport: Viewport,
    recreate_swapchain: bool,
    task_graph: ExecutableTaskGraph<Self>,
    virtual_swapchain_id: Id<Swapchain>,
}

fn main() {
    pollster::block_on(run());
}

async fn run() {
    let mut glfw = glfw::init(glfw::log_errors!()).unwrap();
    glfw.window_hint(glfw::WindowHint::ClientApi(glfw::ClientApiHint::NoApi));
    let (mut pwindow, events) = glfw
        .create_window(640, 480, "Glass Turtle", glfw::WindowMode::Windowed)
        .unwrap();

    pwindow.set_framebuffer_size_polling(true);
    pwindow.set_close_polling(true);

    let window = Arc::new(pwindow);

    let mut app = App::new(&window);
    app.resumed(app.instance.clone(), window.clone());

    let mut close_requested = false;

    while !window.should_close() && !close_requested {
        glfw.wait_events();

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
        let window_extensions =
            Surface::required_extensions(&window).expect("Failed to get required extensions");

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

        let device_extensions = DeviceExtensions {
            khr_swapchain: true,
            ..DeviceExtensions::empty()
        };

        let (physical_device, queue_family_index) = instance
            .enumerate_physical_devices()
            .unwrap()
            .filter(|p| p.supported_extensions().contains(&device_extensions))
            .filter_map(|p| {
                p.queue_family_properties()
                    .iter()
                    .enumerate()
                    .position(|(i, q)| {
                        q.queue_flags.intersects(QueueFlags::GRAPHICS)
                            && p.presentation_support(i as u32, &window).unwrap()
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

        // let physical_device = instance
        //     .enumerate_physical_devices()
        //     .unwrap()
        //     .nth(0)
        //     .unwrap();

        // println!(
        //     "Using device: {} (type: {:?})",
        //     physical_device.properties().device_name,
        //     physical_device.properties().device_type
        // );

        // let queue_family_index = physical_device
        //     .queue_family_properties()
        //     .iter()
        //     .enumerate()
        //     .find_map(|(index, properties)| {
        //         properties
        //             .queue_flags
        //             .contains(QueueFlags::GRAPHICS)
        //             .then_some(index as u32)
        //     })
        //     .expect("could not find a graphics family");

        // let device_extensions = DeviceExtensions {
        //     khr_swapchain: true,
        //     ..DeviceExtensions::default()
        // };

        let queue_create_info = QueueCreateInfo {
            queue_family_index,
            ..Default::default()
        };

        let (device, mut queues) = match Device::new(
            physical_device,
            DeviceCreateInfo {
                queue_create_infos: vec![queue_create_info],
                enabled_extensions: device_extensions,
                enabled_features: DeviceFeatures::empty(),
                ..Default::default()
            },
        ) {
            Ok(d) => d,
            Err(err) => panic!("Could not build device: {:?}", err),
        };

        let queue = queues.next().unwrap();

        let resources = Resources::new(&device, &Default::default());

        let flight_id = resources.create_flight(MAX_FRAMES_IN_FLIGHT).unwrap();

        let rcx = None;

        App {
            instance,
            device,
            queue,
            resources,
            flight_id,
            rcx,
        }
    }

    fn resumed(&mut self, vulkan: Arc<Instance>, window: Arc<PWindow>) {
        let surface = Surface::from_window(vulkan, window.clone()).unwrap();
        let (width, height) = window.get_framebuffer_size();
        let image_extent = [width as u32, height as u32];
        let view_extent = [width as f32, height as f32];

        let (swapchain_id, swapchain_format) =
            get_swapchain_id(&self, surface.clone(), image_extent);

        let viewport = Viewport {
            offset: [0.0, 0.0],
            extent: view_extent,
            depth_range: 0.0..=1.0,
        };

        let mut task_graph: TaskGraph<RenderContext> = TaskGraph::new(&self.resources, 1, 1);
        let virtual_swapchain_id = task_graph.add_swapchain(&SwapchainCreateInfo {
            image_format: swapchain_format,
            ..Default::default()
        });

        let virtual_framebuffer_id = task_graph.add_framebuffer();

        let triangle_node_id = task_graph
            .create_task_node(
                "Triangle",
                QueueFamilyType::Graphics,
                TriangleTask::new(self, virtual_swapchain_id),
            )
            .framebuffer(virtual_framebuffer_id)
            .color_attachment(
                virtual_swapchain_id.current_image_id(),
                AccessTypes::COLOR_ATTACHMENT_WRITE,
                ImageLayoutType::Optimal,
                &AttachmentInfo {
                    clear: true,
                    ..Default::default()
                },
            )
            .build();

        let mut task_graph = unsafe {
            task_graph.compile(&CompileInfo {
                // We need to provide all queues that we want to use for executing the graph. The
                // queue family types that were specified in the task nodes must be compatible with
                // these queues.
                //
                // In this example, we only have a single graphics queue.
                queues: &[&self.queue],
                // We use the same queue for presentation. You must specify a present queue if your
                // task graph uses any swapchains.
                present_queue: Some(&self.queue),
                // The flight that we use to track each execution of this task graph.
                flight_id: self.flight_id,
                ..Default::default()
            })
        }
        .unwrap();

        let triangle_node = task_graph.task_node_mut(triangle_node_id).unwrap();
        let subpass = triangle_node.subpass().unwrap().clone();
        triangle_node
            .task_mut()
            .downcast_mut::<TriangleTask>()
            .unwrap()
            .create_pipeline(&self, &subpass);

        let recreate_swapchain = false;

        self.rcx = Some(RenderContext {
            window,
            swapchain_id,
            viewport,
            recreate_swapchain,
            task_graph,
            virtual_swapchain_id,
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

        if rcx.recreate_swapchain {
            let flight = self.resources.flight(self.flight_id).unwrap();
            flight.wait(None).unwrap();

            rcx.swapchain_id = self
                .resources
                .recreate_swapchain(rcx.swapchain_id, |create_info: SwapchainCreateInfo| {
                    SwapchainCreateInfo {
                        image_extent,
                        ..create_info
                    }
                })
                .expect("failed to recreate swapchain");

            rcx.viewport.extent = view_extent;
            rcx.recreate_swapchain = false;
        }

        let flight = self.resources.flight(self.flight_id);
        flight.unwrap().wait(None).unwrap();

        let resource_map =
            resource_map!(&rcx.task_graph, rcx.virtual_swapchain_id => rcx.swapchain_id).unwrap();

        // Finally, it is time to execute the graph.
        match unsafe { rcx.task_graph.execute(resource_map, &rcx, || {}) } {
            Ok(()) => {}
            // Since the task graph also handles presenting to the swapchain, it may return
            // a swapchain error. When the swapchain is "out of date", we set a flag to
            // recreate it during the next frame.
            Err(ExecuteError::Swapchain {
                error: Validated::Error(VulkanError::OutOfDate),
                ..
            }) => {
                rcx.recreate_swapchain = true;
            }
            Err(e) => {
                panic!("failed to execute next frame: {e:?}");
            }
        }

        self.rcx = Some(rcx);
    }
}

fn get_swapchain_id(
    app: &App,
    surface: Arc<Surface>,
    image_extent: [u32; 2],
) -> (Id<Swapchain>, Format) {
    let surface_capabilities = app
        .device
        .physical_device()
        .surface_capabilities(&surface, Default::default())
        .unwrap();

    let (swapchain_format, _) = app
        .device
        .physical_device()
        .surface_formats(&surface, Default::default())
        .unwrap()[0];

    let swapchain_create_info = SwapchainCreateInfo {
        min_image_count: surface_capabilities
            .min_image_count
            .max(MIN_SWAPCHAIN_IMAGES),
        image_format: swapchain_format,
        image_extent: image_extent,
        image_usage: ImageUsage::COLOR_ATTACHMENT,
        composite_alpha: surface_capabilities
            .supported_composite_alpha
            .into_iter()
            .next()
            .unwrap(),
        ..Default::default()
    };

    let swapchain_id = app
        .resources
        .create_swapchain(app.flight_id, surface, swapchain_create_info)
        .unwrap();

    (swapchain_id, swapchain_format)
}

struct TriangleTask {
    pipeline: Option<Arc<GraphicsPipeline>>,
    vertex_buffer_id: Id<Buffer>,
    swapchain_id: Id<Swapchain>,
}

impl TriangleTask {
    fn new(app: &mut App, swapchain_id: Id<Swapchain>) -> Self {
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

        // Allocate the Vulkan buffer that will hold the vertices.
        //
        // Since we are using vulkano's task graph, the buffer is created using the `Resources`
        // collection.
        let vertex_buffer_id = app
            .resources
            .create_buffer(
                BufferCreateInfo {
                    // We are going to bind this buffer as a vertex buffer.
                    usage: BufferUsage::VERTEX_BUFFER,
                    ..Default::default()
                },
                AllocationCreateInfo {
                    // We want the buffer to be located on the device (GPU) so it is fast to access
                    // from shaders. It must also be writable from the host side (CPU) to initially
                    // upload the data.
                    memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                        | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                    ..Default::default()
                },
                // The device layout determines the size and alignment of the buffer.
                DeviceLayout::for_value(vertices.as_slice()).unwrap(),
            )
            .unwrap();

        unsafe {
            vulkano_taskgraph::execute(
                &app.queue,
                &app.resources,
                app.flight_id,
                |_cbf, tcx| {
                    tcx.write_buffer::<[MyVertex]>(vertex_buffer_id, ..)?
                        .copy_from_slice(&vertices);

                    Ok(())
                },
                [(vertex_buffer_id, HostAccessType::Write)],
                [],
                [],
            )
        }
        .unwrap();

        let pipeline = None;

        Self {
            pipeline,
            vertex_buffer_id,
            swapchain_id,
        }
    }

    pub fn create_pipeline(&mut self, app: &App, subpass: &Subpass) {
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
        let pipeline = {
            let vs = vs::load(app.device.clone())
                .unwrap()
                .entry_point("main")
                .unwrap();
            let fs = fs::load(app.device.clone())
                .unwrap()
                .entry_point("main")
                .unwrap();

            let vertex_input_state = MyVertex::per_vertex().definition(&vs).unwrap();

            let stages = [
                PipelineShaderStageCreateInfo::new(vs),
                PipelineShaderStageCreateInfo::new(fs),
            ];

            let layout = PipelineLayout::new(
                app.device.clone(),
                PipelineDescriptorSetLayoutCreateInfo::from_stages(stages.iter())
                    .into_pipeline_layout_create_info(app.device.clone())
                    .unwrap(),
            )
            .unwrap();

            GraphicsPipeline::new(
                app.device.clone(),
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
        };

        self.pipeline = Some(pipeline);
    }
}

// The `Task` trait defines the logic of a task in the task graph.
impl Task for TriangleTask {
    type World = RenderContext;

    fn clear_values(&self, clear_values: &mut ClearValues<'_>) {
        // Earlier, we requested that the color attachment of the task node should be cleared. This
        // method is where we specify the clear values that should be used.

        clear_values.set(
            self.swapchain_id.current_image_id(),
            [2.0 / 255.0, 6.0 / 255.0, 24.0 / 255.0, 1.0],
        );
    }

    unsafe fn execute(
        &self,
        cbf: &mut RecordingCommandBuffer<'_>,
        _tcx: &mut TaskContext<'_>,
        rcx: &Self::World,
    ) -> vulkano_taskgraph::TaskResult {
        // This method is called when the task graph executes the task node. Here, we record all
        // GPU commands to execute as part of this task.

        // Update the dynamic viewport, which is set to the current window and swapchain size.

        unsafe {
            cbf.set_viewport(0, std::slice::from_ref(&rcx.viewport))?;
            cbf.bind_pipeline_graphics(self.pipeline.as_ref().unwrap())?;
            cbf.bind_vertex_buffers(0, &[self.vertex_buffer_id], &[0], &[], &[])?;
            cbf.draw(3, 1, 0, 0)?;
        }

        // If you are familiar with Vulkan, you will notice that we have performed no manual
        // synchronization here. This is handled entirely by the task graph as long as we have
        // specified all resources that we want to access when creating the task node.

        Ok(())
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
