#![allow(
    dead_code,
    unused_variables,
)]

use crate::*;
use app_data::AppData;
use instance::VALIDATION_ENABLED;
use sync_objects::MAX_FRAMES_IN_FLIGHT;
use structs::{Mat4, UniformBufferObject};

use std::time::Instant;
use std::mem::size_of;
use std::ptr::copy_nonoverlapping as memcpy;

use anyhow::{anyhow, Result};
use cgmath::{point3, vec3, Deg};
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
    start: Instant,
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
        descriptor::create_descriptor_set_layout(&device, &mut data)?;
        pipeline::create_pipeline(&device, &mut data)?;
        command_pool::create_command_pool(&instance, &device, &mut data)?;
        depth_objects::create_depth_objects(&instance, &device, &mut data)?;
        framebuffers::create_framebuffers(&device, &mut data)?;
        texture::create_texture_image(&instance, &device, &mut data)?;
        texture::create_texture_image_view(&device, &mut data)?;
        texture::create_texture_sampler(&device, &mut data)?;
        model::load_model(&mut data)?;
        buffers::create_vertex_buffer(&instance, &device, &mut data)?;
        buffers::create_index_buffer(&instance, &device, &mut data)?;
        buffers::create_uniform_buffers(&instance, &device, &mut data)?;
        descriptor::create_descriptor_pool(&device, &mut data)?;
        descriptor::create_descriptor_sets(&device, &mut data)?;
        command_buffers::create_command_buffers(&device, &mut data)?;
        sync_objects::create_sync_objects(&device, &mut data)?;

        Ok(Self { 
            entry, 
            instance, 
            data, 
            device, 
            frame: 0,
            resized: false,
            start: Instant::now(),
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

        // Update uniform buffer with new transformation matrix
        self.update_uniform_buffer(image_index)?;

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

    
    /// Updates the uniform buffer object for our Vulkan app.
    unsafe fn update_uniform_buffer(&self, image_index: usize) -> Result<()> {
        // Create MVP matrix by time
        let time = self.start.elapsed().as_secs_f32();

        let model = Mat4::from_axis_angle(
            vec3(0.0, 0.0, 1.0), 
            Deg(90.0) * time,
        );

        let view = Mat4::look_at_rh(
            point3::<f32>(2.0, 2.0, 2.0),
            point3::<f32>(0.0, 0.0, 0.0),
            vec3(0.0, 0.0, 1.0),
        );

        #[rustfmt::skip]
        let correction = Mat4::new(
            1.0,  0.0,       0.0, 0.0,
            0.0, -1.0,       0.0, 0.0,
            0.0,  0.0, 1.0 / 2.0, 0.0,
            0.0,  0.0, 1.0 / 2.0, 1.0,
        );

        let proj = correction * cgmath::perspective(
            Deg(45.0),
            self.data.swapchain_extent.width as f32 / self.data.swapchain_extent.height as f32,
            0.1,
            10.0,
        );

        let ubo = UniformBufferObject { model, view, proj };

        // Update uniform buffer with MVP matrix
        let memory = self.device.map_memory(
            self.data.uniform_buffers_memory[image_index],
            0,
            size_of::<UniformBufferObject>() as u64,
            vk::MemoryMapFlags::empty(),
        )?;
        memcpy(&ubo, memory.cast(), 1);

        self.device.unmap_memory(self.data.uniform_buffers_memory[image_index]);

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
        depth_objects::create_depth_objects(&self.instance, &self.device, &mut self.data)?;
        framebuffers::create_framebuffers(&self.device, &mut self.data)?;
        buffers::create_uniform_buffers(&self.instance, &self.device, &mut self.data)?;
        descriptor::create_descriptor_pool(&self.device, &mut self.data)?;
        descriptor::create_descriptor_sets(&self.device, &mut self.data)?;
        
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
        self.device.free_memory(self.data.index_buffer_memory, None);
        self.device.destroy_buffer(self.data.index_buffer, None);
        self.device.free_memory(self.data.vertex_buffer_memory, None);
        self.device.destroy_buffer(self.data.vertex_buffer, None);
        self.device.destroy_sampler(self.data.texture_sampler, None);
        self.device.destroy_image_view(self.data.texture_image_view, None);
        self.device.free_memory(self.data.texture_image_memory, None);
        self.device.destroy_image(self.data.texture_image, None);

        self.device.destroy_command_pool(self.data.command_pool, None);
        self.device.destroy_descriptor_set_layout(self.data.descriptor_set_layout, None);
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
        self.device.free_command_buffers(self.data.command_pool, &self.data.command_buffers);
        
        // destory descriptor pool
        self.device.destroy_descriptor_pool(self.data.descriptor_pool, None);
        // destory uniform buffers
        self.data.uniform_buffers_memory
            .iter()
            .for_each(|m| 
                self.device.free_memory(*m, None));
        self.data.uniform_buffers
            .iter()
            .for_each(|b| 
                self.device.destroy_buffer(*b, None));
        
        // destory depth image
        self.device.destroy_image_view(self.data.depth_image_view, None);
        self.device.free_memory(self.data.depth_image_memory, None);
        self.device.destroy_image(self.data.depth_image, None);

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