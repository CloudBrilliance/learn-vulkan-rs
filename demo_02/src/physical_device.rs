//================================================
// Logical Device
//================================================
#![allow(
    dead_code,         // 允许未使用的代码
    unused_variables,  // 允许未使用的变量
)]

use crate::app_data::AppData;

use anyhow::{anyhow, Result};
use vulkanalia::prelude::v1_0::*;
use log::*;
use thiserror::Error;

// 定义一个错误类型 SuitabilityError，用于表示缺少所需的队列家族
#[derive(Debug, Error)]
#[error("Missing {0}.")]
pub struct SuitabilityError(pub &'static str);

// 用于检查物理设备是否适合
unsafe fn check_physical_device(
    instance: &Instance,
    data: &AppData,
    physical_device: vk::PhysicalDevice,
) -> Result<()> {
    QueueFamilyIndices::get(instance, data, physical_device)?;  // 获取队列家族索引
    Ok(())
}

pub unsafe fn pick_physical_device(instance: &Instance, data: &mut AppData) -> Result<()> {
    // 枚举所有可用的物理设备
    for physical_device in instance.enumerate_physical_devices()? {
        let properties = instance
            .get_physical_device_properties(physical_device);  // 获取设备的属性

        // 检查设备是否适合
        if let Err(error) = check_physical_device(instance, data, physical_device) {
            // 如果不适合，记录警告日志并跳过该设备
            warn!("Skipping physical device (`{}`): {}", properties.device_name, error);
        } else {
            // 如果适合，记录信息日志，保存设备到 AppData，并返回Ok(())
            info!("Selected physical device (`{}`).", properties.device_name);
            data.physical_device = physical_device;
            return Ok(());
        }
    }

    // 如果没有找到适合的设备，返回错误
    Err(anyhow!("Failed to find suitable physical device."))
}


//================================================
// Structs
//================================================
#[derive(Copy, Clone, Debug)]
struct QueueFamilyIndices {
    graphics: u32,  // 用于图形操作的队列索引
}

impl QueueFamilyIndices {
    // 用于获取指定物理设备的队列家族属性
    unsafe fn get(
        instance: &Instance,                  // Vulkan 实例
        data: &AppData,                       // 应用数据
        physical_device: vk::PhysicalDevice,  // 物理设备
    ) -> Result<Self> {
        // 获取物理设备的队列家族属性
        let properties = instance
            .get_physical_device_queue_family_properties(physical_device);

        // 查找支持图形队列的索引
        let graphics = properties
            .iter()
            .position(|p| p.queue_flags.contains(vk::QueueFlags::GRAPHICS))
            .map(|i| i as u32);

        if let Some(graphics) = graphics {
            // 如果找到支持图形队列的索引，返回包含该索引的 QueueFamilyIndices 实例
            Ok(Self { graphics })
        } else {
            // 如果没有找到，返回错误
            Err(anyhow!(SuitabilityError("Missing required queue families.")))
        }
    }
}