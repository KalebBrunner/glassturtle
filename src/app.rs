use std::sync::Arc;
use vulkano::{
    buffer::Subbuffer,
    command_buffer::{
        AutoCommandBufferBuilder, CommandBufferUsage, RenderPassBeginInfo, SubpassBeginInfo,
        SubpassContents, SubpassEndInfo, allocator::StandardCommandBufferAllocator,
    },
    device::{Device, Queue},
    image::SampleCount,
    pipeline::graphics::viewport::Viewport,
    swapchain::{self, SwapchainPresentInfo},
    sync::{self, GpuFuture},
};

use crate::rcx::{RenderContext, build_msaa_framebuffers, create_msaa_image};
use crate::shaders::struct_triangle::MyTriangleVertex;

pub struct App {
    pub window: Arc<glfw::PWindow>,
    pub device: Arc<Device>,
    pub queue: Arc<Queue>,
    pub command_buffer_allocator: Arc<StandardCommandBufferAllocator>,
    pub vertex_buffer: Subbuffer<[MyTriangleVertex]>,
    pub render_context: Option<RenderContext>,
    pub sample_count_index: usize,
}

impl App {
    pub fn draw_frame(&mut self) {
        let rcx = self.render_context.as_mut().unwrap();

        rcx.previous_frame_end.as_mut().unwrap().cleanup_finished();

        if rcx.recreate_swapchain {
            let image_extent = self.window.get_framebuffer_size();
            recreate_swapchain(rcx, image_extent);
        }

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

        let mut builder = AutoCommandBufferBuilder::primary(
            self.command_buffer_allocator.clone(),
            self.queue.queue_family_index(),
            CommandBufferUsage::OneTimeSubmit,
        )
        .unwrap();

        builder
            .begin_render_pass(
                RenderPassBeginInfo {
                    clear_values: vec![
                        Some([0.0, 0.0, 0.0, 0.0].into()), // msaa attachment: gets cleared
                        None, // swapchain resolve target: load_op is DontCare
                    ],
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

    const SAMPLE_COUNTS: [SampleCount; 5] = [
        SampleCount::Sample1,
        SampleCount::Sample2,
        SampleCount::Sample4,
        SampleCount::Sample8,
        SampleCount::Sample16,
    ];

    pub fn cycle_sample_count(&mut self, direction: i32) {
        let len = Self::SAMPLE_COUNTS.len() as i32;
        self.sample_count_index =
            ((self.sample_count_index as i32 + direction).rem_euclid(len)) as usize;
        let new_count = Self::SAMPLE_COUNTS[self.sample_count_index];
        println!("Sample count: {:?}", new_count);
        // rebuild rcx here next
    }
}

pub fn recreate_swapchain(rcx: &mut RenderContext, image_extent: (i32, i32)) {
    if image_extent.0 == 0 || image_extent.1 == 0 {
        println!("RCX: Window is minimized");
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
    rcx.msaa_image_view = create_msaa_image(rcx.memory_allocator.clone(), &rcx.swapchain);
    rcx.framebuffers = build_msaa_framebuffers(
        &new_images,
        rcx.render_pass.clone(),
        rcx.msaa_image_view.clone(),
    );

    rcx.viewport = Viewport {
        offset: [0.0, 0.0],
        extent: [image_extent.0 as f32, image_extent.1 as f32],
        depth_range: 0.0..=1.0,
    };

    rcx.recreate_swapchain = false;
}
