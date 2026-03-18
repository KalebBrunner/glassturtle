use std::sync::Arc;

use glfw::PWindow;
use vulkano::{
    device::Device,
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
    swapchain::Surface,
    sync::{self, GpuFuture},
};

use crate::{
    d_render_context::{
        render_context::RenderContext, swapchain::init_swapchain, window_size_dependent_setup,
    },
    struct_my_vertex::MyTriangleVertex,
};

pub fn init_render_context(
    window: Arc<PWindow>,
    surface: Arc<Surface>,
    device: Arc<Device>,
) -> RenderContext {
    let (swapchain, swapchain_images) = init_swapchain(&surface, device.clone());

    mod vs {
        vulkano_shaders::shader! {
            ty: "vertex",
            src: r"
                #version 450
                layout(location = 0) in vec2 position;
                layout(location = 1) in vec3 color;
                layout(location = 0) out vec3 v_color;
                
                void main() {
                    gl_Position = vec4(position, 0.0, 1.0);
                    v_color = color;
                }
            ",
        }
    }

    mod fs {
        vulkano_shaders::shader! {
            ty: "fragment",
            src: r"
                #version 450
    
                layout(location = 0) in vec3 v_color;
                layout(location = 0) out vec4 f_color;
                
                void main() {
                    f_color = vec4(v_color, 1.0);
                }
            ",
        }
    }

    // let vs = vs::load(device.clone()).unwrap();
    // let fs = fs::load(device.clone()).unwrap();

    let render_pass = vulkano::single_pass_renderpass!(
        device.clone(),
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
    .unwrap();

    // let framebuffers = window_size_dependent_setup(&swapchain_images, render_pass.clone());

    let pipeline = {
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

        GraphicsPipeline::new(
            device.clone(),
            None,
            GraphicsPipelineCreateInfo {
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
                    attachments: vec![ColorBlendAttachmentState::default()],
                    ..Default::default()
                }),
                // Dynamic states allows us to specify parts of the pipeline settings when
                // recording the command buffer, before we perform drawing. Here, we specify
                // that the viewport should be dynamic.
                dynamic_state: [DynamicState::Viewport].into_iter().collect(),
                subpass: Some(subpass.into()),
                ..GraphicsPipelineCreateInfo::layout(layout)
            },
        )
        .unwrap()
    };

    let framebuffers = window_size_dependent_setup(&swapchain_images, render_pass.clone());
    let recreate_swapchain = false;
    let previous_frame_end = Some(sync::now(device.clone()).boxed());

    let image_extent = swapchain.image_extent();

    let viewport = Viewport {
        offset: [0.0, 0.0],
        extent: [image_extent[0] as f32, image_extent[1] as f32],
        depth_range: 0.0..=1.0,
    };

    RenderContext {
        window,
        swapchain,
        render_pass,
        framebuffers,
        pipeline,
        viewport,
        recreate_swapchain,
        previous_frame_end,
    }
}
