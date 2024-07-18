//================================================
// Logical Device
//================================================
use crate::app_data::AppData;
use crate::instance::{VALIDATION_ENABLED, VALIDATION_LAYER, PORTABILITY_MACOS_VERSION};
use crate::structs::QueueFamilyIndices;
use crate::physical_device::DEVICE_EXTENSIONS;

use anyhow::Result;
use vulkanalia::prelude::v1_0::*;

pub unsafe fn create_logical_device(entry: &Entry, instance: &Instance, data: &mut AppData) -> Result<Device> {
    // Queue Create Infos
    let indices = QueueFamilyIndices::get(instance, data, data.physical_device)?;

    let queue_priorities = &[1.0];
    let queue_info = vk::DeviceQueueCreateInfo::builder()
        .queue_family_index(indices.graphics)
        .queue_priorities(queue_priorities);

    // Layers
    let layers = if VALIDATION_ENABLED {
        vec![VALIDATION_LAYER.as_ptr()]
    } else {
        vec![]
    };

    // Extensions
    let mut extensions = DEVICE_EXTENSIONS
        .iter()
        .map(|n| n.as_ptr())
        .collect::<Vec<_>>();

    // Required by Vulkan SDK on macOS since 1.3.216.
    if cfg!(target_os = "macos") && entry.version()? >= PORTABILITY_MACOS_VERSION {
        extensions.push(vk::KHR_PORTABILITY_SUBSET_EXTENSION.name.as_ptr());
    }

    // Features
    let features = vk::PhysicalDeviceFeatures::builder();

    // Create
    let queue_infos = &[queue_info];
    let info = vk::DeviceCreateInfo::builder()
        .queue_create_infos(queue_infos)
        .enabled_layer_names(&layers)
        .enabled_extension_names(&extensions)
        .enabled_features(&features);

    let device = instance
        .create_device(data.physical_device, &info, None)?;

    // Queues
    data.graphics_queue = device
        .get_device_queue(indices.graphics, 0);

    Ok(device)
}
