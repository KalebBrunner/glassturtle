use winit::{
    application::ApplicationHandler,
    dpi::LogicalSize,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop},
    window::{Window, WindowId},
};

struct App {
    window: Option<Window>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }

        let window_attributes = Window::default_attributes()
            .with_title("vk_playground")
            .with_inner_size(LogicalSize::new(960.0, 540.0));

        let window = event_loop
            .create_window(window_attributes)
            .expect("failed to create window");

        self.window = Some(window);

        // --- Vulkan instance setup will go here later ---
        //
        // use std::sync::Arc;
        // use vulkano::{
        //     instance::{Instance, InstanceCreateInfo},
        //     VulkanLibrary,
        // };
        //
        // let library = VulkanLibrary::new()
        //     .expect("failed to load Vulkan loader");
        //
        // let instance = Instance::new(
        //     library,
        //     InstanceCreateInfo::application_from_cargo_toml(),
        // )
        // .expect("failed to create Vulkan instance");
        //
        // println!("Vulkan instance created: {:?}", instance);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(new_size) => {
                println!("resized to {}x{}", new_size.width, new_size.height);
            }
            WindowEvent::RedrawRequested => {
                // Rendering will go here later.
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }
}

fn main() {
    // println!("Hello world")
    let event_loop = EventLoop::new().expect("failed to create event loop");

    let mut app = App { window: None };

    event_loop.run_app(&mut app).expect("event loop error");
}
