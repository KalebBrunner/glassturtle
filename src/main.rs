// #![allow(unused_imports)]
// #![allow(unused_variables)]

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
use crate::render::RenderContextKB;
use crate::vulkan::{WindowContextKB, create_instance, setup_debug_messenger};
use crate::windower::create_window;

use vulkano::swapchain::Surface;

fn main() {
    pollster::block_on(run());
}

async fn run() {
    // GLFW is the window manager api. It stands for Good Luck Fellow Witches.
    let (mut glfw, window, events) = create_window();

    // Functionallity requested by the window manager and thus also the OS.
    // Most notably is the VkSurfaceKHR extension.
    let window_extensions =
        Surface::required_extensions(&window).expect("Failed to get required extensions");

    // Vulkan itself
    let vulkan = create_instance(window_extensions);
    let _debug_messenger = setup_debug_messenger(vulkan.clone());

    // Native platform surface or window objects are abstracted by surface objects,
    // which are represented by VkSurfaceKHR handles.
    let surface =
        Surface::from_window(vulkan.clone(), window.clone()).expect("failed to create surface");

    let window_context = WindowContextKB::new(vulkan.clone(), surface.clone());

    print_diagnostics(
        &vulkan,
        &surface,
        &window_context.device,
        &window_context.queue,
    );

    // An encapsulation for the rendering engine
    let render_context = RenderContextKB::new(
        window_context.surface.clone(),
        window_context.device.clone(),
    );

    let mesh = create_mesh(window_context.device.clone());

    let camera = Camera::new();

    let mut myapp = App {
        window,
        window_context,
        render_context,
        mesh,
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
