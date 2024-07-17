#![allow(
    dead_code,
    unused_variables,
)]

use crate::*;
use app_data::AppData;
use instance::VALIDATION_ENABLED;

use anyhow::{anyhow, Result};
use winit::window::Window;
use vulkanalia::loader::{LibloadingLoader, LIBRARY};
use vulkanalia::prelude::v1_0::*;
use vulkanalia::vk::ExtDebugUtilsExtension;

/// Our Vulkan app.
#[derive(Clone, Debug)]
pub struct App {
    entry: Entry,
    instance: Instance,
    data: AppData,
    device: Device,
}

impl App {
    /// Creates our Vulkan app.
    pub unsafe fn create(window: &Window) -> Result<Self> {
        let loader: LibloadingLoader = LibloadingLoader::new(LIBRARY)?;
        let entry: Entry = Entry::new(loader).map_err(
            |b| anyhow!("{}", b))?;
        let mut data: AppData = AppData::default();
        let instance: Instance = instance::create_instance(window, &entry, &mut data)?;
        physical_device::pick_physical_device(&instance, &mut data)?;

        let device: Device = logical_device::create_logical_device(&entry, &instance, &mut data)?;

        Ok(Self { entry, instance, data, device })
    }

    /// Renders a frame for our Vulkan app.
    pub unsafe fn render(&mut self, window: &Window) -> Result<()> {
        Ok(())
    }

    /// Destroys our Vulkan app.
    #[rustfmt::skip]
    pub unsafe fn destroy(&mut self) {
        self.device.destroy_device(None);

        if VALIDATION_ENABLED {
            self.instance.destroy_debug_utils_messenger_ext(self.data.messenger, None);
        }

        self.instance.destroy_instance(None);
    }
}