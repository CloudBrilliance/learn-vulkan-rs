use vulkanalia::prelude::v1_0::*;

/// The Vulkan handles and associated properties used by our Vulkan app.
#[derive(Clone, Debug, Default)]
pub struct AppData {
    // Debug
    pub messenger: vk::DebugUtilsMessengerEXT,
    // Physical Device
    pub physical_device: vk::PhysicalDevice,
}