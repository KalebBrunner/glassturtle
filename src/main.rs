#![allow(unused_imports)]
#![allow(unused_variables)]
use std::sync::Arc;
use vulkano::command_buffer::allocator::StandardCommandBufferAllocator;
use vulkano::device::{Device, Queue};
use vulkano::image::{Image, view::ImageView};
use vulkano::instance::debug::DebugUtilsMessenger;
use vulkano::instance::{Instance, InstanceExtensions};
use vulkano::render_pass::{Framebuffer, FramebufferCreateInfo, RenderPass};
use vulkano::swapchain::Surface;

mod app;
mod camera;
mod diagnostics_print;
mod keybinds;
mod mesh;
mod render;
mod shaders;
mod vulkan;
mod windower;

use crate::app::App;
use crate::camera::Camera;
use crate::diagnostics_print::print_diagnostics;

use crate::mesh::create_mesh;
use crate::render::create_render_context;
use crate::windower::create_window;

pub struct RendererKB {
    vulkan: VulkanKB,
    device: VulkanDeviceKB,
}

impl RendererKB {
    pub fn new(window_extensions: InstanceExtensions) -> Self {
        let vulkan = VulkanKB::new(window_extensions);
        let device = VulkanDeviceKB::new(vulkan.instance.clone());
        Self { vulkan, device }
    }

    pub fn print(&self, surface: &Arc<Surface>) {
        print_diagnostics(
            &self.vulkan.instance,
            &surface,
            &self.device.device,
            &self.device.queue,
        );
    }
}

pub struct VulkanKB {
    pub instance: Arc<Instance>,
    pub _debug_messenger: DebugUtilsMessenger,
}

pub struct VulkanDeviceKB {
    pub device: Arc<Device>,
    pub queue: Arc<Queue>,
}

fn main() {
    pollster::block_on(run());
}

async fn run() {
    // GLFW is the window manager api. It stands for Good Luck Fellow Witches
    let (mut glfw, window, events) = create_window();

    let window_extensions =
        Surface::required_extensions(&window).expect("Failed to get required extensions");

    let my_renderer = RendererKB::new(window_extensions);

    let surface = Surface::from_window(my_renderer.vulkan.instance.clone(), window.clone())
        .expect("failed to create surface");

    my_renderer.print(&surface);

    let render_context = create_render_context(surface.clone(), my_renderer.device.device.clone());

    let command_buffer_allocator = Arc::new(StandardCommandBufferAllocator::new(
        my_renderer.device.device.clone(),
        Default::default(),
    ));

    let mesh = create_mesh(my_renderer.device.device.clone());

    let camera = Camera::new();

    let mut myapp = App {
        window,
        device: my_renderer.device.device,
        queue: my_renderer.device.queue,
        command_buffer_allocator,
        mesh,
        render_context: render_context,
        camera,
    };

    while !myapp.window.clone().should_close() {
        glfw.poll_events();

        for (_, event) in glfw::flush_messages(&events) {
            if let glfw::WindowEvent::FramebufferSize(_, _) = event {
                myapp.render_context.recreate_swapchain = true;
            }

            keybinds::handle_keybinds(&event, &mut myapp.camera);
        }

        myapp.draw_frame();
    }
}
