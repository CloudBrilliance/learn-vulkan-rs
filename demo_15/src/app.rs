#![allow(
    dead_code,
    unused_variables,
)]

use crate::*;
use app_data::AppData;
use instance::VALIDATION_ENABLED;
use sync_objects::MAX_FRAMES_IN_FLIGHT;

use anyhow::{anyhow, Result};
use winit::window::Window;
use vulkanalia::loader::{LibloadingLoader, LIBRARY};
use vulkanalia::prelude::v1_0::*;
use vulkanalia::window as vk_window;
use vulkanalia::vk::ExtDebugUtilsExtension;
use vulkanalia::vk::KhrSurfaceExtension;
use vulkanalia::vk::KhrSwapchainExtension;

/// Our Vulkan app.
#[derive(Clone, Debug)]
pub struct App {
    entry: Entry,
    instance: Instance,
    data: AppData,
    device: Device,
    frame: usize,
    pub resized: bool,
}

impl App {
    /// Creates our Vulkan app.
    pub unsafe fn create(window: &Window) -> Result<Self> {
        let loader: LibloadingLoader = LibloadingLoader::new(LIBRARY)?;
        let entry: Entry = Entry::new(loader).map_err(
            |b| anyhow!("{}", b))?;
        let mut data: AppData = AppData::default();
        
        let instance: Instance = instance::create_instance(window, &entry, &mut data)?;
        data.surface = vk_window::create_surface(&instance, &window, &window)?;
        physical_device::pick_physical_device(&instance, &mut data)?;
        let device: Device = logical_device::create_logical_device(&entry, &instance, &mut data)?;
        swapchain::create_swapchain(window, &instance, &device, &mut data)?;
        swapchain::create_swapchain_image_views(&device, &mut data)?;
        pipeline::create_render_pass(&instance, &device, &mut data)?;
        pipeline::create_pipeline(&device, &mut data)?;
        framebuffers::create_framebuffers(&device, &mut data)?;
        command_pool::create_command_pool(&instance, &device, &mut data)?;
        
        // vertex buffer
        buffers::create_vertex_buffer(&instance, &device, &mut data)?;
        // bind vertex buffers
        command_buffers::create_command_buffers(&device, &mut data)?;
        
        sync_objects::create_sync_objects(&device, &mut data)?;

        Ok(Self { 
            entry, 
            instance, 
            data, 
            device, 
            frame: 0,
            resized: false,
        })
    }

    /// Renders a frame for our Vulkan app.
    pub unsafe fn render(&mut self, window: &Window) -> Result<()> {
        // Get semaphore and wait
        let in_flight_fence = self.data.in_flight_fences[self.frame];

        self.device.wait_for_fences(
            &[in_flight_fence], 
            true, 
            u64::max_value()
        )?;

        // Get image from swapchain
        let image_index = self.device
            .acquire_next_image_khr(
                self.data.swapchain,
                u64::max_value(),
                self.data.image_available_semaphores[self.frame],
                vk::Fence::null(),
            )?
            .0 as usize;

        let image_in_flight = self.data.images_in_flight[image_index];
        if !image_in_flight.is_null() {
            self.device.wait_for_fences(
                &[image_in_flight], 
                true, 
                u64::max_value()
            )?;
        }

        self.data.images_in_flight[image_index] = in_flight_fence;

        // Commit command buffer
        let wait_semaphores = &[self.data.image_available_semaphores[self.frame]];
        let wait_stages = &[vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
        let command_buffers = &[self.data.command_buffers[image_index]];
        let signal_semaphores = &[self.data.render_finished_semaphores[self.frame]];
        let submit_info = vk::SubmitInfo::builder()
            .wait_semaphores(wait_semaphores)
            .wait_dst_stage_mask(wait_stages)
            .command_buffers(command_buffers)
            .signal_semaphores(signal_semaphores);

        self.device.reset_fences(&[in_flight_fence])?;

        // Submit drawing commands to the graphics queue
        self.device.queue_submit(
            self.data.graphics_queue, 
            &[submit_info], 
            in_flight_fence
        )?;

        // Present
        let swapchains = &[self.data.swapchain];
        let image_indices = &[image_index as u32];
        let present_info = vk::PresentInfoKHR::builder()
            .wait_semaphores(signal_semaphores)
            .swapchains(swapchains)
            .image_indices(image_indices);

        let result = self.device
            .queue_present_khr(self.data.present_queue, &present_info);
        
        // Recreates the swapchain
        let changed = result == Ok(vk::SuccessCode::SUBOPTIMAL_KHR) || result == Err(vk::ErrorCode::OUT_OF_DATE_KHR);
        if self.resized || changed {
            self.resized = false;
            self.recreate_swapchain(window)?;
        } else if let Err(e) = result {
            return Err(anyhow!(e));
        }

        // Update current frame index
        self.frame = (self.frame + 1) % MAX_FRAMES_IN_FLIGHT;

        Ok(())
    }

    /// Recreates the swapchain for our Vulkan app.
    #[rustfmt::skip]
    unsafe fn recreate_swapchain(&mut self, window: &Window) -> Result<()> {
        self.device.device_wait_idle()?;
        self.destroy_swapchain();  // destory

        // recreate
        swapchain::create_swapchain(window, &self.instance, &self.device, &mut self.data)?;
        swapchain::create_swapchain_image_views(&self.device, &mut self.data)?;
        pipeline::create_render_pass(&self.instance, &self.device, &mut self.data)?;
        pipeline::create_pipeline(&self.device, &mut self.data)?;
        framebuffers::create_framebuffers(&self.device, &mut self.data)?;
        command_buffers::create_command_buffers(&self.device, &mut self.data)?;
        
        self.data.images_in_flight
            .resize(self.data.swapchain_images.len(), vk::Fence::null());
        
        Ok(())
    }

    /// Destroys our Vulkan app.
    #[rustfmt::skip]
    pub unsafe fn destroy(&mut self) {
        self.device.device_wait_idle().unwrap();

        self.destroy_swapchain();

        self.data.in_flight_fences
            .iter()
            .for_each(|f| 
                self.device.destroy_fence(*f, None));
        self.data.render_finished_semaphores
            .iter()
            .for_each(|s| 
                self.device.destroy_semaphore(*s, None));
        self.data.image_available_semaphores
            .iter().
            for_each(|s| 
                self.device.destroy_semaphore(*s, None));
        
        // destory buffers
        self.device.free_memory(self.data.vertex_buffer_memory, None);
        self.device.destroy_buffer(self.data.vertex_buffer, None);

        self.device.destroy_command_pool(self.data.command_pool, None);
        self.device.destroy_device(None);
        self.instance.destroy_surface_khr(self.data.surface, None);

        if VALIDATION_ENABLED {
            self.instance.destroy_debug_utils_messenger_ext(self.data.messenger, None);
        }

        self.instance.destroy_instance(None);
    }

    /// Destroys the parts of our Vulkan app related to the swapchain.
    #[rustfmt::skip]
    unsafe fn destroy_swapchain(&mut self) {
        // clean command buffers
        self.device.free_command_buffers(self.data.command_pool, &self.data.command_buffers);
        self.data.framebuffers
            .iter()
            .for_each(|f| 
                self.device.destroy_framebuffer(*f, None));
        self.device.destroy_pipeline(self.data.pipeline, None);
        self.device.destroy_pipeline_layout(self.data.pipeline_layout, None);
        self.device.destroy_render_pass(self.data.render_pass, None);
        self.data.swapchain_image_views
            .iter()
            .for_each(|v| 
                self.device.destroy_image_view(*v, None));
        self.device.destroy_swapchain_khr(self.data.swapchain, None);
    }
}