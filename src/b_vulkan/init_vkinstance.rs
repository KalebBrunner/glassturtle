use std::sync::Arc;

use vulkano::{
    Version, VulkanLibrary,
    instance::{Instance, InstanceCreateInfo, InstanceExtensions},
};

const USE_VALIDATION_LAYERS: bool = true;
const VALIDATION_LAYERS: [&str; 1] = ["VK_LAYER_KHRONOS_validation"];

pub fn init_vkinstance(extensions: InstanceExtensions) -> Arc<Instance> {
    let library = VulkanLibrary::new().expect("failed to load Vulkan loader");
    println!("Highest Vulkan ver: {:?}", library.api_version());

    let mut layers = vec![];
    if USE_VALIDATION_LAYERS {
        layers = create_validation_layers(&library);
    }

    println!("Active layers: {:?}", layers);
    println!("Active extensions: {:?}", extensions);

    let create_info = InstanceCreateInfo {
        engine_name: Some("Glass Turtle Graphics".into()),
        engine_version: Version::V1_5,
        max_api_version: Some(Version::V1_5),
        enabled_layers: layers,
        enabled_extensions: extensions,
        ..InstanceCreateInfo::application_from_cargo_toml()
    };

    Instance::new(library, create_info).expect("failed to create Vulkan instance")
}

fn create_validation_layers(library: &Arc<VulkanLibrary>) -> Vec<String> {
    let available_layer_names: Vec<String> = library
        .layer_properties()
        .unwrap()
        .map(|layer| layer.name().to_owned())
        .collect();

    let required_layer_names = VALIDATION_LAYERS.map(|layer| layer.to_owned()).to_vec();

    let missing_layer_names: Vec<_> = required_layer_names
        .iter()
        .filter(|req| !available_layer_names.iter().any(|avail| avail == *req))
        .collect();

    assert!(
        missing_layer_names.is_empty(),
        "Required Vulkan validation layer(s) not found: {:?}",
        missing_layer_names,
    );

    required_layer_names
}
