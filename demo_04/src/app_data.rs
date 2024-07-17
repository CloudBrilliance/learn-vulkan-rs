use vulkanalia::prelude::v1_0::*;

/// The Vulkan handles and associated properties used by our Vulkan app.
#[derive(Clone, Debug, Default)]
pub struct AppData {
    // Debug
    pub messenger:       vk::DebugUtilsMessengerEXT,
    // Surface
    pub surface:         vk::SurfaceKHR,
    // Physical Device / Logical Device
    pub physical_device: vk::PhysicalDevice,
    pub graphics_queue:  vk::Queue,
    pub present_queue:   vk::Queue,
    // Swapchain
    pub swapchain_format: vk::Format,
    pub swapchain_extent: vk::Extent2D,
    pub swapchain:        vk::SwapchainKHR,
    pub swapchain_images: Vec<vk::Image>,
}