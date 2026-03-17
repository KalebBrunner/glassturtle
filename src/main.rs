// #![allow(unused_imports)]
// #![allow(unused_variables)]

use std::sync::Arc;

mod init_glfw;
mod presentation;
mod setup;
mod shaders;
mod vertex_buffer;

use vulkano::buffer::Subbuffer;
use vulkano::command_buffer::allocator::StandardCommandBufferAllocator;
use vulkano::command_buffer::{
    AutoCommandBufferBuilder, CommandBufferUsage, RenderPassBeginInfo, SubpassBeginInfo,
    SubpassContents, SubpassEndInfo,
};
use vulkano::device::{Device, Queue};
use vulkano::pipeline::GraphicsPipeline;
use vulkano::pipeline::graphics::viewport::Viewport;
use vulkano::render_pass::{Framebuffer, RenderPass};
use vulkano::swapchain::{self, Swapchain, SwapchainPresentInfo};
use vulkano::sync::{self, GpuFuture};

use crate::init_glfw::init_glfw;
use crate::presentation::{init_render_context, window_size_dependent_setup};
use crate::setup::init_vulkan;
use crate::shaders::MyTriangleVertex;
use crate::vertex_buffer::init_vertex_bufffer;

struct App {
    device: Arc<Device>,
    queue: Arc<Queue>,
    command_buffer_allocator: Arc<StandardCommandBufferAllocator>,
    vertex_buffer: Subbuffer<[MyTriangleVertex]>,
    render_context: Option<RenderContext>,
}

struct RenderContext {
    window: Arc<glfw::PWindow>,
    swapchain: Arc<Swapchain>,
    render_pass: Arc<RenderPass>,
    framebuffers: Vec<Arc<Framebuffer>>,
    pipeline: Arc<GraphicsPipeline>,
    viewport: Viewport,
    recreate_swapchain: bool,
    previous_frame_end: Option<Box<dyn GpuFuture>>,
}

impl App {
    fn draw_frame(&mut self) {
        let rcx = self.render_context.as_mut().unwrap();

        // 1) clean up old GPU work
        rcx.previous_frame_end.as_mut().unwrap().cleanup_finished();

        if rcx.recreate_swapchain {
            let image_extent = rcx.window.get_framebuffer_size();
            if image_extent.0 == 0 || image_extent.1 == 0 {
                return;
            }

            let (new_swapchain, new_images) = rcx
                .swapchain
                .recreate(vulkano::swapchain::SwapchainCreateInfo {
                    image_extent: [image_extent.0 as u32, image_extent.1 as u32],
                    ..rcx.swapchain.create_info()
                })
                .expect("failed to recreate swapchain");

            rcx.swapchain = new_swapchain;
            rcx.framebuffers = window_size_dependent_setup(&new_images, rcx.render_pass.clone());

            rcx.viewport = Viewport {
                offset: [0.0, 0.0],
                extent: [image_extent.0 as f32, image_extent.1 as f32],
                depth_range: 0.0..=1.0,
            };

            rcx.recreate_swapchain = false;
        }

        // 2) acquire next swapchain image
        let (image_index, suboptimal, acquire_future) =
            match swapchain::acquire_next_image(rcx.swapchain.clone(), None)
                .map_err(vulkano::Validated::unwrap)
            {
                Ok(r) => r,
                Err(vulkano::VulkanError::OutOfDate) => {
                    rcx.recreate_swapchain = true;
                    return;
                }
                Err(e) => panic!("failed to acquire image: {e}"),
            };

        if suboptimal {
            rcx.recreate_swapchain = true;
        }

        // 3) record commands for this frame
        let mut builder = AutoCommandBufferBuilder::primary(
            self.command_buffer_allocator.clone(),
            self.queue.queue_family_index(),
            CommandBufferUsage::OneTimeSubmit,
        )
        .unwrap();

        builder
            .begin_render_pass(
                RenderPassBeginInfo {
                    clear_values: vec![Some([0.0, 0.0, 0.0, 1.0].into())],
                    ..RenderPassBeginInfo::framebuffer(
                        rcx.framebuffers[image_index as usize].clone(),
                    )
                },
                SubpassBeginInfo {
                    contents: SubpassContents::Inline,
                    ..Default::default()
                },
            )
            .unwrap()
            .bind_pipeline_graphics(rcx.pipeline.clone())
            .unwrap()
            .set_viewport(0, [rcx.viewport.clone()].into_iter().collect())
            .unwrap()
            .bind_vertex_buffers(0, self.vertex_buffer.clone())
            .unwrap();

        unsafe { builder.draw(self.vertex_buffer.len() as u32, 1, 0, 0) }.unwrap();

        builder.end_render_pass(SubpassEndInfo::default()).unwrap();

        let command_buffer = builder.build().unwrap();

        // 4) submit + 5) present
        let future = rcx
            .previous_frame_end
            .take()
            .unwrap()
            .join(acquire_future)
            .then_execute(self.queue.clone(), command_buffer)
            .unwrap()
            .then_swapchain_present(
                self.queue.clone(),
                SwapchainPresentInfo::swapchain_image_index(rcx.swapchain.clone(), image_index),
            )
            .then_signal_fence_and_flush();

        // 6) store future for next frame
        rcx.previous_frame_end = Some(match future.map_err(vulkano::Validated::unwrap) {
            Ok(future) => future.boxed(),
            Err(vulkano::VulkanError::OutOfDate) => {
                rcx.recreate_swapchain = true;
                sync::now(self.device.clone()).boxed()
            }
            Err(e) => {
                eprintln!("failed to flush future: {e}");
                sync::now(self.device.clone()).boxed()
            }
        });
    }
}

fn main() {
    pollster::block_on(run());
}

async fn run() {
    // GLFW stands for Good Luck Fellow Witches and is the window manager
    let (mut glfw, glfw_window, events) = init_glfw();
    let window = Arc::new(glfw_window);
    let (_vulkan, surface, device, queue) = init_vulkan(window.clone());

    let (command_buffer_allocator, vertex_buffer) = init_vertex_bufffer(device.clone());

    let render_context = init_render_context(window.clone(), surface, device.clone());

    let mut myapp = App {
        device,
        queue,
        command_buffer_allocator,
        vertex_buffer,
        render_context: Some(render_context),
    };

    while !window.clone().should_close() {
        glfw.poll_events();

        for (_, event) in glfw::flush_messages(&events) {
            if let glfw::WindowEvent::FramebufferSize(_, _) = event
                && let Some(rcx) = myapp.render_context.as_mut()
            {
                rcx.recreate_swapchain = true;
            }
        }

        myapp.draw_frame();
    }
}
