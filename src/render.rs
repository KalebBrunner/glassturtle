use crate::camera::Camera;
use crate::shaders::mesh_vertex::CameraUniform;
use crate::shaders::{fragment::fs, mesh_vertex::MeshVertex, vertex::vs};
use std::{
    cmp::{max, min},
    sync::Arc,
};
use vulkano::buffer::{Buffer, BufferCreateInfo, BufferUsage};
use vulkano::descriptor_set::allocator::StandardDescriptorSetAllocator;
use vulkano::descriptor_set::{DescriptorSet, WriteDescriptorSet};
use vulkano::device::Device;
use vulkano::format::Format;
use vulkano::image::view::ImageView;
use vulkano::image::{ImageCreateInfo, ImageType};
use vulkano::memory::allocator::{AllocationCreateInfo, MemoryTypeFilter, StandardMemoryAllocator};
use vulkano::pipeline::Pipeline;
use vulkano::render_pass::FramebufferCreateInfo;
use vulkano::{
    image::{Image, ImageUsage},
    pipeline::{
        DynamicState, GraphicsPipeline, PipelineLayout, PipelineShaderStageCreateInfo,
        graphics::{
            GraphicsPipelineCreateInfo,
            color_blend::{ColorBlendAttachmentState, ColorBlendState},
            depth_stencil::{CompareOp, DepthState, DepthStencilState},
            input_assembly::InputAssemblyState,
            multisample::MultisampleState,
            rasterization::RasterizationState,
            vertex_input::{Vertex, VertexDefinition},
            viewport::{Viewport, ViewportState},
        },
        layout::PipelineDescriptorSetLayoutCreateInfo,
    },
    render_pass::{Framebuffer, RenderPass, Subpass},
    swapchain::{FullScreenExclusive, PresentMode, Surface, Swapchain, SwapchainCreateInfo},
    sync::{self, GpuFuture},
};

pub struct RenderContextKB {
    pub swapchain: Arc<Swapchain>,
    pub render_pass: Arc<RenderPass>,
    pub framebuffers: Vec<Arc<Framebuffer>>,
    pub pipeline: Arc<GraphicsPipeline>,
    pub viewport: Viewport,
    pub recreate_swapchain: bool,
    pub previous_frame_end: Option<Box<dyn GpuFuture>>,
}

impl RenderContextKB {
    pub fn new(surface: Arc<Surface>, device: Arc<Device>) -> Self {
        let (swapchain, swapchain_images) = create_swapchain(&surface, device.clone());

        let image_extent = swapchain.image_extent();

        let viewport = Viewport {
            offset: [0.0, 0.0],
            extent: [image_extent[0] as f32, image_extent[1] as f32],
            depth_range: 0.0..=1.0,
        };

        let render_pass = create_renderpass(device.clone(), swapchain.image_format());

        let framebuffers =
            create_framebuffers(device.clone(), &swapchain_images, render_pass.clone());

        let pipeline = create_pipeline(device.clone(), render_pass.clone());

        let previous_frame_end = Some(sync::now(device.clone()).boxed());

        let recreate_swapchain = false;

        Self {
            swapchain,
            viewport,
            render_pass,
            pipeline,
            framebuffers,
            previous_frame_end,
            recreate_swapchain,
        }
    }

    pub fn create_camera_descriptor_set(
        &self,
        descriptor_set_allocator: Arc<StandardDescriptorSetAllocator>,
        device: Arc<Device>,
        camera: &Camera,
    ) -> Arc<DescriptorSet> {
        let aspect = self.viewport.extent[0] / self.viewport.extent[1];

        let camera_uniform = CameraUniform {
            world_to_clip: camera.world_to_clip(aspect).to_cols_array_2d(),
        };

        let memory_allocator = Arc::new(StandardMemoryAllocator::new_default(device.clone()));

        let uniform_buffer = Buffer::from_data(
            memory_allocator,
            BufferCreateInfo {
                usage: BufferUsage::UNIFORM_BUFFER,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_HOST
                    | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
            camera_uniform,
        )
        .unwrap();

        let layout = self.pipeline.layout().set_layouts().get(0).unwrap().clone();

        let descriptor_set = DescriptorSet::new(
            descriptor_set_allocator.clone(),
            layout,
            [WriteDescriptorSet::buffer(0, uniform_buffer)],
            [],
        )
        .unwrap();

        return descriptor_set;
    }
}

fn create_pipeline(device: Arc<Device>, render_pass: Arc<RenderPass>) -> Arc<GraphicsPipeline> {
    {
        let vs = vs::load(device.clone())
            .unwrap()
            .entry_point("main")
            .unwrap();
        let fs = fs::load(device.clone())
            .unwrap()
            .entry_point("main")
            .unwrap();

        let vertex_input_state = MeshVertex::per_vertex().definition(&vs).unwrap();

        let stages = vec![
            PipelineShaderStageCreateInfo::new(vs),
            PipelineShaderStageCreateInfo::new(fs),
        ];

        let layout = PipelineLayout::new(
            device.clone(),
            PipelineDescriptorSetLayoutCreateInfo::from_stages(&stages)
                .into_pipeline_layout_create_info(device.clone())
                .unwrap(),
        )
        .unwrap();
        let subpass = Subpass::from(render_pass.clone(), 0).unwrap();

        let graphics_pipeline_info = GraphicsPipelineCreateInfo {
            stages: stages.into(),
            vertex_input_state: Some(vertex_input_state),
            input_assembly_state: Some(InputAssemblyState::default()),
            viewport_state: Some(ViewportState::default()),
            rasterization_state: Some(RasterizationState::default()),
            multisample_state: Some(MultisampleState::default()),
            color_blend_state: Some(ColorBlendState {
                attachments: vec![ColorBlendAttachmentState {
                    blend: Some(vulkano::pipeline::graphics::color_blend::AttachmentBlend::alpha()),
                    ..Default::default()
                }],
                ..Default::default()
            }),
            depth_stencil_state: Some(DepthStencilState {
                depth: Some(DepthState {
                    write_enable: true,
                    compare_op: CompareOp::Less,
                }),
                ..Default::default()
            }),
            // Dynamic states allows us to specify parts of the pipeline settings when
            // recording the command buffer, before we perform drawing. Here, we specify
            // that the viewport should be dynamic.
            dynamic_state: [DynamicState::Viewport].into_iter().collect(),
            subpass: Some(subpass.into()),
            ..GraphicsPipelineCreateInfo::layout(layout)
        };

        GraphicsPipeline::new(device.clone(), None, graphics_pipeline_info).unwrap()
    }
}

fn create_renderpass(device: Arc<Device>, format: Format) -> Arc<RenderPass> {
    vulkano::single_pass_renderpass!(
        device,
        attachments: {
            // `color` is a custom name we give to the first and only attachment.
            color: {
                format: format,
                samples: 1,
                load_op: Clear,
                store_op: Store,
            },
            depth: {
                format: Format::D32_SFLOAT,
                samples: 1,
                load_op: Clear,
                store_op: DontCare,
            }
        },
        pass: {
            color: [color],
            depth_stencil: {depth},
        },
    )
    .unwrap()
}

fn create_swapchain(
    surface: &Arc<Surface>,
    logical_device: Arc<Device>,
) -> (Arc<Swapchain>, Vec<Arc<Image>>) {
    let surface_capabilities = logical_device
        .physical_device()
        .surface_capabilities(surface, Default::default())
        .unwrap();

    let image_extent = surface_capabilities.current_extent.unwrap_or([1920, 1080]);

    let min_image_count = match surface_capabilities.max_image_count {
        None => max(2, surface_capabilities.min_image_count),
        Some(limit) => min(max(2, surface_capabilities.min_image_count), limit),
    };

    let pre_transform = surface_capabilities.current_transform;

    let (image_format, color_space) = logical_device
        .physical_device()
        .surface_formats(surface, Default::default())
        .unwrap()
        .into_iter()
        .find(|(format, _)| {
            matches!(
                format,
                vulkano::format::Format::B8G8R8A8_SRGB
                    | vulkano::format::Format::B8G8R8A8_UNORM
                    | vulkano::format::Format::R8G8B8A8_SRGB
                    | vulkano::format::Format::R8G8B8A8_UNORM
            )
        })
        .expect("No alpha-capable surface format found");
    let present_mode = PresentMode::Fifo;
    let full_screen_exclusive = FullScreenExclusive::Default;

    // let composite_alpha = surface_capabilities
    //     .supported_composite_alpha
    //     .into_iter()
    //     .find(|mode| matches!(mode, CompositeAlpha::Inherit))
    //     .expect("This surface does not support transparent window composition");

    let swap_info = SwapchainCreateInfo {
        min_image_count,
        image_format,
        image_extent,
        image_usage: ImageUsage::COLOR_ATTACHMENT,
        pre_transform,
        image_color_space: color_space,
        // composite_alpha,
        present_mode,
        full_screen_exclusive,
        ..Default::default()
    };

    let (swapchain, images) =
        Swapchain::new(logical_device.clone(), surface.clone(), swap_info).unwrap();

    (swapchain, images)
}

pub fn create_framebuffers(
    device: Arc<Device>,
    images: &[Arc<Image>],
    render_pass: Arc<RenderPass>,
) -> Vec<Arc<Framebuffer>> {
    let memory_allocator = Arc::new(StandardMemoryAllocator::new_default(device));

    images
        .iter()
        .map(|image| {
            let color_view = ImageView::new_default(image.clone()).unwrap();

            let depth_image = Image::new(
                memory_allocator.clone(),
                ImageCreateInfo {
                    image_type: ImageType::Dim2d,
                    format: Format::D32_SFLOAT,
                    extent: [image.extent()[0], image.extent()[1], 1],
                    usage: ImageUsage::DEPTH_STENCIL_ATTACHMENT,
                    ..Default::default()
                },
                AllocationCreateInfo::default(),
            )
            .unwrap();

            let depth_view = ImageView::new_default(depth_image).unwrap();

            Framebuffer::new(
                render_pass.clone(),
                FramebufferCreateInfo {
                    attachments: vec![color_view, depth_view],
                    ..Default::default()
                },
            )
            .unwrap()
        })
        .collect()
}
