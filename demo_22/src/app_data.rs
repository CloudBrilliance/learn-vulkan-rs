use crate::structs::Vertex;
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
    pub msaa_samples:    vk::SampleCountFlags,
    pub graphics_queue:  vk::Queue,
    pub present_queue:   vk::Queue,
    // Swapchain
    pub swapchain_format:      vk::Format,
    pub swapchain_extent:      vk::Extent2D,
    pub swapchain:             vk::SwapchainKHR,
    pub swapchain_images:      Vec<vk::Image>,
    pub swapchain_image_views: Vec<vk::ImageView>,
    // Pipeline
    pub render_pass:           vk::RenderPass,
    pub descriptor_set_layout: vk::DescriptorSetLayout,
    pub pipeline_layout:       vk::PipelineLayout,
    pub pipeline:              vk::Pipeline,
    // Framebuffers
    pub framebuffers: Vec<vk::Framebuffer>,
    // Command Pool
    pub command_pool: vk::CommandPool,
    // Color
    pub color_image:        vk::Image,
    pub color_image_memory: vk::DeviceMemory,
    pub color_image_view:   vk::ImageView,
    // Depth
    pub depth_image:        vk::Image,
    pub depth_image_memory: vk::DeviceMemory,
    pub depth_image_view:   vk::ImageView,
    // Texture
    pub mip_levels:           u32,
    pub texture_image:        vk::Image,
    pub texture_image_memory: vk::DeviceMemory,
    pub texture_image_view:   vk::ImageView,
    pub texture_sampler:      vk::Sampler,
    // Model
    pub vertices: Vec<Vertex>,
    pub indices:  Vec<u32>,
    // Buffers
    pub vertex_buffer:          vk::Buffer,
    pub vertex_buffer_memory:   vk::DeviceMemory,
    pub index_buffer:           vk::Buffer,
    pub index_buffer_memory:    vk::DeviceMemory,
    pub uniform_buffers:        Vec<vk::Buffer>,
    pub uniform_buffers_memory: Vec<vk::DeviceMemory>,
    // Descriptors
    pub descriptor_pool: vk::DescriptorPool,
    pub descriptor_sets: Vec<vk::DescriptorSet>,
    // Command Buffers
    pub command_buffers: Vec<vk::CommandBuffer>,
    // Sync Objects
    pub image_available_semaphores: Vec<vk::Semaphore>,
    pub render_finished_semaphores: Vec<vk::Semaphore>,
    pub in_flight_fences: Vec<vk::Fence>,
    pub images_in_flight: Vec<vk::Fence>,
}