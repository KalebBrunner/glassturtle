use std::sync::Arc;

use glfw::{PWindow, Window, log_errors};
use vulkano::{
    Version, VulkanLibrary,
    device::{
        Device, DeviceCreateInfo, DeviceExtensions, DeviceFeatures, Queue, QueueCreateInfo,
        QueueFlags, physical::PhysicalDeviceType,
    },
    instance::{Instance, InstanceCreateInfo},
    pipeline::graphics::viewport::Viewport,
    swapchain::{Surface, Swapchain},
};
use vulkano_taskgraph::{
    Id,
    graph::ExecutableTaskGraph,
    resource::{Flight, Resources},
};

const MAX_FRAMES_IN_FLIGHT: u32 = 2;
const MIN_SWAPCHAIN_IMAGES: u32 = MAX_FRAMES_IN_FLIGHT + 1;

struct App {
    instance: Arc<Instance>,
    device: Arc<Device>,
    queue: Arc<Queue>,
    resources: Arc<Resources>,
    flight_id: Id<Flight>,
    rcx: Option<RenderContext>,
}

struct RenderContext {
    window: Arc<Window>,
    swapchain_id: Id<Swapchain>,
    viewport: Viewport,
    recreate_swapchain: bool,
    task_graph: ExecutableTaskGraph<Self>,
    virtual_swapchain_id: Id<Swapchain>,
}

fn main() {
    pollster::block_on(run());
}

async fn run() {
    let mut glfw = glfw::init(glfw::log_errors!()).unwrap();
    let (pwindow, _events) = glfw
        .create_window(640, 480, "Glass Turtle", glfw::WindowMode::Windowed)
        .unwrap();
    let window = Arc::new(pwindow);
    // window.set_framebuffer_size_polling(true);
    // window.set_key_polling(true);
    // window.set_mouse_button_polling(true);
    // window.set_pos_polling(true);

    let app = App::new(window.clone());

    while !window.should_close() {
        glfw.poll_events();
    }
}

impl App {
    fn new(window: Arc<PWindow>) -> Self {
        let window_extensions =
            Surface::required_extensions(&window).expect("Failed to get required extensions");

        let library = VulkanLibrary::new()
            .unwrap_or_else(|err| panic!("Couldn't load Vulkan library: {:?}", err));

        let instance_create_info = InstanceCreateInfo {
            engine_name: Some("Glass Turtle Graphics".into()),
            engine_version: Version::V1_0,
            max_api_version: Some(Version::V1_4),
            enabled_layers: vec!["VK_LAYER_KHRONOS_validation".to_owned()],
            enabled_extensions: window_extensions,
            ..InstanceCreateInfo::application_from_cargo_toml()
        };

        let instance = Instance::new(library, instance_create_info)
            .unwrap_or_else(|err| panic!("Couldn't create instance: {:?}", err));

        let surface = Surface::from_window(instance.clone(), window.clone())
            .expect("failed to create surface");

        let device_extensions = DeviceExtensions {
            khr_swapchain: true,
            ..DeviceExtensions::empty()
        };

        let (physical_device, queue_family_index) = instance
            .enumerate_physical_devices()
            .unwrap()
            .filter(|p| p.supported_extensions().contains(&device_extensions))
            .filter_map(|p| {
                p.queue_family_properties()
                    .iter()
                    .enumerate()
                    .position(|(i, q)| {
                        q.queue_flags.intersects(QueueFlags::GRAPHICS)
                            && p.presentation_support(i as u32, &window).unwrap()
                    })
                    .map(|i| (p, i as u32))
            })
            .min_by_key(|(p, _)| match p.properties().device_type {
                PhysicalDeviceType::DiscreteGpu => 0,
                PhysicalDeviceType::IntegratedGpu => 1,
                PhysicalDeviceType::VirtualGpu => 2,
                PhysicalDeviceType::Cpu => 3,
                PhysicalDeviceType::Other => 4,
                _ => 5,
            })
            .expect("no suitable physical device found");
        println!(
            "Using device: {} (type: {:?})",
            physical_device.properties().device_name,
            physical_device.properties().device_type,
        );

        // let physical_device = instance
        //     .enumerate_physical_devices()
        //     .unwrap()
        //     .nth(0)
        //     .unwrap();

        // println!(
        //     "Using device: {} (type: {:?})",
        //     physical_device.properties().device_name,
        //     physical_device.properties().device_type
        // );

        // let queue_family_index = physical_device
        //     .queue_family_properties()
        //     .iter()
        //     .enumerate()
        //     .find_map(|(index, properties)| {
        //         properties
        //             .queue_flags
        //             .contains(QueueFlags::GRAPHICS)
        //             .then_some(index as u32)
        //     })
        //     .expect("could not find a graphics family");

        // let device_extensions = DeviceExtensions {
        //     khr_swapchain: true,
        //     ..DeviceExtensions::default()
        // };

        let queue_create_info = QueueCreateInfo {
            queue_family_index,
            ..Default::default()
        };

        let (device, mut queues) = match Device::new(
            physical_device,
            DeviceCreateInfo {
                queue_create_infos: vec![queue_create_info],
                enabled_extensions: device_extensions,
                enabled_features: DeviceFeatures::empty(),
                ..Default::default()
            },
        ) {
            Ok(d) => d,
            Err(err) => panic!("Could not build device: {:?}", err),
        };

        let queue = queues.next().unwrap();

        let resources = Resources::new(&device, &Default::default());

        let flight_id = resources.create_flight(MAX_FRAMES_IN_FLIGHT).unwrap();

        let rcx = None;

        App {
            instance,
            device,
            queue,
            resources,
            flight_id,
            rcx,
        }
    }
}
