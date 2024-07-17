//================================================
// Instance
//================================================
use crate::app_data::AppData;

use std::collections::HashSet;
use std::ffi::CStr;
use std::os::raw::c_void;

use anyhow::{anyhow, Result};
use winit::window::Window;
use vulkanalia::prelude::v1_0::*;
use vulkanalia::window as vk_window;
use vulkanalia::Version;
use vulkanalia::vk::ExtDebugUtilsExtension;
use log::*;

/// Whether the validation layers should be enabled.
pub const VALIDATION_ENABLED: bool = cfg!(debug_assertions);
/// The name of the validation layers.
const VALIDATION_LAYER: vk::ExtensionName = vk::ExtensionName::from_bytes(b"VK_LAYER_KHRONOS_validation");

/// The Vulkan SDK version that started requiring the portability subset extension for macOS.
const PORTABILITY_MACOS_VERSION: Version = Version::new(1, 3, 216);

pub unsafe fn create_instance(window: &Window, entry: &Entry, data: &mut AppData) -> Result<Instance> {
    // Application Info
    let application_info = vk::ApplicationInfo::builder()
        .application_name(b"Vulkan Tutorial (Rust)\0")  // 应用程序名称
        .application_version(vk::make_version(1, 0, 0))  // 应用程序版本
        .engine_name(b"No Engine\0")  // 引擎名称
        .engine_version(vk::make_version(1, 0, 0))  // 引擎版本
        .api_version(vk::make_version(1, 0, 0));  // Vulkan API版本

    // Layers
    let available_layers = entry
        .enumerate_instance_layer_properties()?  // 枚举实例层属性
        .iter()
        .map(|l| l.layer_name)  // 获取层的名称
        .collect::<HashSet<_>>();  // 转换为 HashSet
    
    // 如果启用了验证层，但可用的层中没有找到验证层，则返回错误
    if VALIDATION_ENABLED && !available_layers.contains(&VALIDATION_LAYER) {
        return Err(anyhow!("Validation layer requested but not supported."));
    }

    // 根据是否启用验证层设置层名称
    let layers = if VALIDATION_ENABLED {
        vec![VALIDATION_LAYER.as_ptr()]  // 验证层名称指针
    } else {
        Vec::new()  // 空向量
    };

    // Extensions（扩展）
    let mut extensions = vk_window::get_required_instance_extensions(window)
        .iter()
        .map(|e| e.as_ptr())  // 获取扩展名称指针
        .collect::<Vec<_>>();  // 转换为向量

    // Required by Vulkan SDK on macOS since 1.3.216.
    let flags = if 
        cfg!(target_os = "macos") && 
        entry.version()? >= PORTABILITY_MACOS_VERSION
    {
        info!("Enabling extensions for macOS portability.");  // 日志信息
        extensions.push(vk::KHR_GET_PHYSICAL_DEVICE_PROPERTIES2_EXTENSION.name.as_ptr());
        extensions.push(vk::KHR_PORTABILITY_ENUMERATION_EXTENSION.name.as_ptr());
        vk::InstanceCreateFlags::ENUMERATE_PORTABILITY_KHR
    } else {
        vk::InstanceCreateFlags::empty()
    };

    if VALIDATION_ENABLED {  // 如果启用了验证层，添加调试工具扩展
        extensions.push(vk::EXT_DEBUG_UTILS_EXTENSION.name.as_ptr());
    }

    // Create
    let mut info = vk::InstanceCreateInfo::builder()
        .application_info(&application_info)   // 应用程序信息
        .enabled_layer_names(&layers)          // 启用的层名称
        .enabled_extension_names(&extensions)  // 启用的扩展名称
        .flags(flags);                                                   // 实例创建标志

    // 调试信息
    let mut debug_info = vk::DebugUtilsMessengerCreateInfoEXT::builder()
        .message_severity(vk::DebugUtilsMessageSeverityFlagsEXT::all())  // 消息严重性
        .message_type(vk::DebugUtilsMessageTypeFlagsEXT::all())  // 消息类型
        .user_callback(Some(debug_callback));    // 用户回调函数

    if VALIDATION_ENABLED {
        // 如果启用了验证层，添加调试信息到实例信息中
        info = info.push_next(&mut debug_info);
    }

    let instance = entry.create_instance(&info, None)?;  // 创建实例

    // Messenger
    if VALIDATION_ENABLED {
        // 如果启用了验证层，创建调试信使
        data.messenger = instance.create_debug_utils_messenger_ext(&debug_info, None)?;
    }

    Ok(instance)
}

extern "system" fn debug_callback(
    severity: vk::DebugUtilsMessageSeverityFlagsEXT,      // 消息严重性
    type_: vk::DebugUtilsMessageTypeFlagsEXT,             // 消息类型
    data: *const vk::DebugUtilsMessengerCallbackDataEXT,  // 调试回调数据
    _: *mut c_void,                                       // 用户数据指针
) -> vk::Bool32 {
    let data = unsafe { *data };  // 解引用数据指针
    // 获取消息字符串
    let message = unsafe { CStr::from_ptr(data.message) }.to_string_lossy();

    // 根据严重性输出日志信息
    if severity >= vk::DebugUtilsMessageSeverityFlagsEXT::ERROR {
        error!("({:?}) {}", type_, message);
    } else if severity >= vk::DebugUtilsMessageSeverityFlagsEXT::WARNING {
        warn!("({:?}) {}", type_, message);
    } else if severity >= vk::DebugUtilsMessageSeverityFlagsEXT::INFO {
        debug!("({:?}) {}", type_, message);
    } else {
        trace!("({:?}) {}", type_, message);
    }

    vk::FALSE  // 返回 false 表示继续执行
}