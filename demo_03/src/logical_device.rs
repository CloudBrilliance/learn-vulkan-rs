//================================================
// Logical Device
//================================================

use crate::app_data::AppData;
use crate::physical_device::QueueFamilyIndices;

use anyhow::Result;
use vulkanalia::prelude::v1_0::*;
use vulkanalia::Version;

/// Whether the validation layers should be enabled.
pub const VALIDATION_ENABLED: bool = cfg!(debug_assertions);
/// The name of the validation layers.
const VALIDATION_LAYER: vk::ExtensionName = vk::ExtensionName::from_bytes(b"VK_LAYER_KHRONOS_validation");

/// The Vulkan SDK version that started requiring the portability subset extension for macOS.
const PORTABILITY_MACOS_VERSION: Version = Version::new(1, 3, 216);

pub unsafe fn create_logical_device(entry: &Entry, instance: &Instance, data: &mut AppData) -> Result<Device> {
    // Queue Create Infos
    let indices = QueueFamilyIndices::get(instance, data, data.physical_device)?;

    let queue_priorities = &[1.0];
    let queue_info = vk::DeviceQueueCreateInfo::builder()
        .queue_family_index(indices.graphics)  // 指定队列家族索引为图形队列的索引
        .queue_priorities(queue_priorities);                                // 指定队列优先级

    // Layers（图层）
    let layers = if VALIDATION_ENABLED {
        vec![VALIDATION_LAYER.as_ptr()]  // 如果启用验证层，则添加验证层
    } else {
        vec![]
    };

    // Extensions
    let mut extensions = vec![];

    // Required by Vulkan SDK on macOS since 1.3.216.
    if cfg!(target_os = "macos") && entry.version()? >= PORTABILITY_MACOS_VERSION {
        extensions.push(vk::KHR_PORTABILITY_SUBSET_EXTENSION.name.as_ptr());
    }

    // Features
    let features = vk::PhysicalDeviceFeatures::builder();  // 使用默认特性

    // Create（创建设备信息）
    let queue_infos = &[queue_info];  // 将队列信息放入数组
    let info = vk::DeviceCreateInfo::builder()
        .queue_create_infos(queue_infos)       // 指定队列创建信息
        .enabled_layer_names(&layers)          // 指定启用的图层（验证层）
        .enabled_extension_names(&extensions)  // 指定启用的扩展
        .enabled_features(&features);                                  // 指定启用的特性

    let device = instance
        .create_device(data.physical_device, &info, None)?;  // 创建设备

    // Queues
    data.graphics_queue = device
        .get_device_queue(indices.graphics, 0);  // 获取图形队列

    Ok(device)
}