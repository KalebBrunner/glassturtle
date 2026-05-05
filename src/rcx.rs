use crate::shaders::{fragment::fs, struct_triangle::MyTriangleVertex, vertex::vs};
use crate::summary::print_swapchain_support_summary;
use crate::window_size_dependent_setup;
use glfw::PWindow;
use std::{
    cmp::{max, min},
    sync::Arc,
};
use vulkano::device::physical::PhysicalDevice;
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
pub struct MyRenderContext {
    pub window: Arc<glfw::PWindow>,
    pub swapchain: Arc<Swapchain>,
    pub render_pass: Arc<RenderPass>,
    pub framebuffers: Vec<Arc<Framebuffer>>,
    pub pipeline: Arc<GraphicsPipeline>,
    pub viewport: Viewport,
    pub recreate_swapchain: bool,
    pub previous_frame_end: Option<Box<dyn GpuFuture>>,
}

pub fn init_rcx(
    window: Arc<PWindow>,
    surface: Arc<Surface>,
    device: Arc<Device>,
) -> MyRenderContext {
    let (swapchain, swapchain_images) = init_swapchain(&surface, device.clone());
    let render_pass = init_renderpass(device.clone(), &swapchain);
    let pipeline = init_pipeline(device.clone(), render_pass.clone());
    let framebuffers = window_size_dependent_setup(&swapchain_images, render_pass.clone());
    let previous_frame_end = Some(sync::now(device.clone()).boxed());
    let viewport = init_viewport(&swapchain);
    let recreate_swapchain = false;

    MyRenderContext {
        window,
        swapchain,
        render_pass,
        pipeline,
        framebuffers,
        previous_frame_end,
        viewport,
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
            multisample_state: Some(MultisampleState::default()),
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

fn init_renderpass(device: Arc<Device>, swapchain: &Arc<Swapchain>) -> Arc<RenderPass> {
    vulkano::single_pass_renderpass!(
        device,
        attachments: {
            // `color` is a custom name we give to the first and only attachment.
            color: {
                // `format: <ty>` indicates the type of the format of the image. This has to be
                // one of the types of the `vulkano::format` module (or alternatively one of
                // your structs that implements the `FormatDesc` trait). Here we use the same
                // format as the swapchain.
                format: swapchain.image_format(),
                // `samples: 1` means that we ask the GPU to use one sample to determine the
                // value of each pixel in the color attachment. We could use a larger value
                // (multisampling) for antialiasing. An example of this can be found in
                // msaa-renderpass.rs.
                samples: 1,
                // `load_op: Clear` means that we ask the GPU to clear the content of this
                // attachment at the start of the drawing.
                load_op: Clear,
                // `store_op: Store` means that we ask the GPU to store the output of the draw
                // in the actual image. We could also ask it to discard the result.
                store_op: Store,
            },
        },
        pass: {
            // We use the attachment named `color` as the one and only color attachment.
            color: [color],
            // No depth-stencil attachment is indicated with empty brackets.
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
        image_usage: ImageUsage::COLOR_ATTACHMENT,
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
