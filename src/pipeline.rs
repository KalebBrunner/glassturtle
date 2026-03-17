// use vulkano::{
//     device::Device,
//     pipeline::{
//         self, GraphicsPipeline,
//         graphics::{
//             GraphicsPipelineCreateInfo,
//             color_blend::{ColorBlendAttachmentState, ColorBlendState},
//             input_assembly::InputAssemblyState,
//             multisample::MultisampleState,
//             rasterization::RasterizationState,
//             viewport::ViewportState,
//         },
//     },
// };

// pub fn init_pipeline(device: Device) {
//     let pipeline_info = GraphicsPipelineCreateInfo {
//         stages: &stages,
//         // How vertex data is read from the vertex buffers into the vertex shader.
//         vertex_input_state: Some(&vertex_input_state),
//         // How vertices are arranged into primitive shapes. The default primitive shape
//         // is a triangle.
//         input_assembly_state: Some(&InputAssemblyState::default()),
//         // How primitives are transformed and clipped to fit the framebuffer. We use a
//         // resizable viewport, set to draw over the entire window.
//         viewport_state: Some(&ViewportState::default()),
//         // How polygons are culled and converted into a raster of pixels. The default
//         // value does not perform any culling.
//         rasterization_state: Some(&RasterizationState::default()),
//         // How multiple fragment shader samples are converted to a single pixel value.
//         // The default value does not perform any multisampling.
//         multisample_state: Some(&MultisampleState::default()),
//         // How pixel values are combined with the values already present in the
//         // framebuffer. The default value overwrites the old value with the new one,
//         // without any blending.
//         color_blend_state: Some(&ColorBlendState {
//             attachments: &[ColorBlendAttachmentState::default()],
//             ..Default::default()
//         }),
//     };

//     // let pipeline = GraphicsPipeline
// }
