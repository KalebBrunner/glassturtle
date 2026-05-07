use crate::shaders::{fragment::fs, struct_triangle::MyTriangleVertex, vertex::vs};
use crate::summary::print_swapchain_support_summary;
use glfw::PWindow;
use std::{
    cmp::{max, min},
    sync::Arc,
};
use vulkano::device::physical::PhysicalDevice;
use vulkano::image::view::ImageView;
use vulkano::image::{ImageCreateInfo, ImageLayout, ImageType, SampleCount};
use vulkano::memory::allocator::{
    AllocationCreateInfo, FreeListAllocator, GenericMemoryAllocator, StandardMemoryAllocator,
};
use vulkano::render_pass::{
    AttachmentDescription, AttachmentLoadOp, AttachmentStoreOp, FramebufferCreateInfo,
};
use vulkano::swapchain::SurfaceInfo;
use vulkano::{
    device::Device,
    image::{Image, ImageUsage},
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
    render_pass::{Framebuffer, RenderPass, Subpass},
    swapchain::{
        CompositeAlpha, FullScreenExclusive, PresentMode, Surface, Swapchain, SwapchainCreateInfo,
    },
    sync::{self, GpuFuture},
};
pub struct RenderContext {
    pub memory_allocator: Arc<StandardMemoryAllocator>,
    pub swapchain: Arc<Swapchain>,
    pub render_pass: Arc<RenderPass>,
    pub framebuffers: Vec<Arc<Framebuffer>>,
    pub pipeline: Arc<GraphicsPipeline>,
    pub msaa_image_view: Arc<ImageView>,
    pub viewport: Viewport,
    pub recreate_swapchain: bool,
    pub previous_frame_end: Option<Box<dyn GpuFuture>>,
}

pub fn init_rcx(
    surface: Arc<Surface>,
    device: Arc<Device>,
    memory_allocator: Arc<GenericMemoryAllocator<FreeListAllocator>>,
) -> RenderContext {
    let (swapchain, swapchain_images) = init_swapchain(&surface, device.clone());

    let viewport = init_viewport(&swapchain);

    let render_pass = init_renderpass(device.clone(), &swapchain);

    let pipeline = init_pipeline(device.clone(), render_pass.clone());

    let msaa_image_view = create_msaa_image(memory_allocator.clone(), &swapchain);

    let framebuffers = build_msaa_framebuffers(
        &swapchain_images,
        render_pass.clone(),
        msaa_image_view.clone(),
    );

    let previous_frame_end = Some(sync::now(device.clone()).boxed());

    let recreate_swapchain = false;

    RenderContext {
        memory_allocator,
        swapchain,
        viewport,
        render_pass,
        pipeline,
        msaa_image_view,
        framebuffers,
        previous_frame_end,
        recreate_swapchain,
    }
}

fn init_viewport(swapchain: &Arc<Swapchain>) -> Viewport {
    let image_extent = swapchain.image_extent();

    Viewport {
        offset: [0.0, 0.0],
        extent: [image_extent[0] as f32, image_extent[1] as f32],
        depth_range: 0.0..=1.0,
    }
}

fn init_pipeline(device: Arc<Device>, render_pass: Arc<RenderPass>) -> Arc<GraphicsPipeline> {
    {
        let vs = vs::load(device.clone())
            .unwrap()
            .entry_point("main")
            .unwrap();
        let fs = fs::load(device.clone())
            .unwrap()
            .entry_point("main")
            .unwrap();

        let vertex_input_state = MyTriangleVertex::per_vertex().definition(&vs).unwrap();

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

        let msstate = MultisampleState {
            rasterization_samples: SampleCount::Sample16,
            ..Default::default()
        };

        let graphics_pipeline_info = GraphicsPipelineCreateInfo {
            stages: stages.into(),
            // How vertex data is read from the vertex buffers into the vertex shader.
            vertex_input_state: Some(vertex_input_state),
            // How vertices are arranged into primitive shapes. The default primitive shape
            // is a triangle.
            input_assembly_state: Some(InputAssemblyState::default()),
            // How primitives are transformed and clipped to fit the framebuffer. We use a
            // resizable viewport, set to draw over the entire window.
            viewport_state: Some(ViewportState::default()),
            // How polygons are culled and converted into a raster of pixels. The default
            // value does not perform any culling.
            rasterization_state: Some(RasterizationState::default()),
            // How multiple fragment shader samples are converted to a single pixel value.
            // The default value does not perform any multisampling.
            multisample_state: Some(msstate),
            // How pixel values are combined with the values already present in the
            // framebuffer. The default value overwrites the old value with the new one,
            // without any blending.
            color_blend_state: Some(ColorBlendState {
                attachments: vec![ColorBlendAttachmentState {
                    blend: Some(vulkano::pipeline::graphics::color_blend::AttachmentBlend::alpha()),
                    ..Default::default()
                }],
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

fn init_renderpass(
    device: Arc<Device>,
    swapchain: &Arc<Swapchain>,
    sample_count: SampleCount,
) -> Arc<RenderPass> {
    vulkano::single_pass_renderpass!(
        device,
        attachments: {
            // The multisample image we render into - note samples: 4
            // store_op: DontCare because we never need to read this back,
            // only the resolved result matters
            msaa_color: {
                format: swapchain.image_format(),
                samples: sample_count,
                load_op: Clear,
                store_op: DontCare,
            },
            // The resolve target - this IS the swapchain image
            // load_op: DontCare because we're going to overwrite every pixel via resolve
            color: {
                format: swapchain.image_format(),
                samples: 1,
                load_op: DontCare,
                store_op: Store,
                initial_layout: ImageLayout::Undefined,
                final_layout: ImageLayout::PresentSrc,
            },
        },
        pass: {
            color: [msaa_color],
            color_resolve: [color],
            depth_stencil: {},
        },
    )
    .unwrap()
}

pub fn init_swapchain(
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

    let composite_alpha = surface_capabilities
        .supported_composite_alpha
        .into_iter()
        .find(|mode| matches!(mode, CompositeAlpha::Inherit))
        .expect("This surface does not support transparent window composition");

    let swap_info = SwapchainCreateInfo {
        min_image_count,
        image_format,
        image_extent,
        image_usage: ImageUsage::COLOR_ATTACHMENT | ImageUsage::TRANSFER_DST,
        pre_transform,
        image_color_space: color_space,
        composite_alpha,
        present_mode,
        full_screen_exclusive,
        ..Default::default()
    };

    let (swapchain, images) =
        Swapchain::new(logical_device.clone(), surface.clone(), swap_info).unwrap();

    (swapchain, images)
}
pub fn create_msaa_image(
    memory_allocator: Arc<StandardMemoryAllocator>,
    swapchain: &Arc<Swapchain>,
) -> Arc<ImageView> {
    let image_extent = swapchain.image_extent();
    let msaa_image = Image::new(
        memory_allocator,
        ImageCreateInfo {
            image_type: ImageType::Dim2d,
            format: swapchain.image_format(),
            extent: [image_extent[0], image_extent[1], 1],
            samples: SampleCount::Sample16,
            usage: ImageUsage::COLOR_ATTACHMENT | ImageUsage::TRANSIENT_ATTACHMENT,
            ..Default::default()
        },
        AllocationCreateInfo::default(),
    )
    .unwrap();
    ImageView::new_default(msaa_image).unwrap()
}

pub fn build_msaa_framebuffers(
    images: &[Arc<Image>],
    render_pass: Arc<RenderPass>,
    msaa_image_view: Arc<ImageView>,
) -> Vec<Arc<Framebuffer>> {
    images
        .iter()
        .map(|image| {
            let swapchain_view = ImageView::new_default(image.clone()).unwrap();
            Framebuffer::new(
                render_pass.clone(),
                FramebufferCreateInfo {
                    attachments: vec![
                        msaa_image_view.clone(), // attachment 0: render into this
                        swapchain_view,          // attachment 1: resolve into this
                    ],
                    ..Default::default()
                },
            )
            .unwrap()
        })
        .collect()
}
