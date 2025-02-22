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
use std::ptr::{copy_nonoverlapping as memcpy, slice_from_raw_parts};

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
    pub models: usize,
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
        
        command_pool::create_command_pools(&instance, &device, &mut data)?;
        
        color_objects::create_color_objects(&instance, &device, &mut data)?;
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
            models: 1,
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

        self.update_command_buffer(image_index)?;
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

    
    /// Updates a command buffer for our Vulkan app.
    #[rustfmt::skip]
    unsafe fn update_command_buffer(&mut self, image_index: usize) -> Result<()> {
        // Reset
        let command_pool = self.data.command_pools[image_index];
        self.device.reset_command_pool(command_pool, vk::CommandPoolResetFlags::empty())?;

        let command_buffer = self.data.command_buffers[image_index];

        // Commands
        let info = vk::CommandBufferBeginInfo::builder().flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);

        self.device.begin_command_buffer(command_buffer, &info)?;

        let render_area = vk::Rect2D::builder()
            .offset(vk::Offset2D::default())
            .extent(self.data.swapchain_extent);

        let color_clear_value = vk::ClearValue {
            color: vk::ClearColorValue {
                float32: [0.0, 0.0, 0.0, 1.0],
            },
        };

        let depth_clear_value = vk::ClearValue {
            depth_stencil: vk::ClearDepthStencilValue { depth: 1.0, stencil: 0 },
        };

        let clear_values = &[color_clear_value, depth_clear_value];
        let info = vk::RenderPassBeginInfo::builder()
            .render_pass(self.data.render_pass)
            .framebuffer(self.data.framebuffers[image_index])
            .render_area(render_area)
            .clear_values(clear_values);

        self.device.cmd_begin_render_pass(command_buffer, &info, vk::SubpassContents::SECONDARY_COMMAND_BUFFERS);

        let secondary_command_buffers = (0..self.models)
            .map(|i| 
                self.update_secondary_command_buffer(image_index, i))
            .collect::<Result<Vec<_>, _>>()?;
        self.device.cmd_execute_commands(command_buffer, &secondary_command_buffers[..]);

        self.device.cmd_end_render_pass(command_buffer);

        self.device.end_command_buffer(command_buffer)?;

        Ok(())
    }

    /// Updates a secondary command buffer for our Vulkan app.
    #[rustfmt::skip]
    unsafe fn update_secondary_command_buffer(
        &mut self,
        image_index: usize,
        model_index: usize,
    ) -> Result<vk::CommandBuffer> {
        // Allocate

        let command_buffers = &mut self.data.secondary_command_buffers[image_index];
        while model_index >= command_buffers.len() {
            let allocate_info = vk::CommandBufferAllocateInfo::builder()
                .command_pool(self.data.command_pools[image_index])
                .level(vk::CommandBufferLevel::SECONDARY)
                .command_buffer_count(1);

            let command_buffer = self.device.allocate_command_buffers(&allocate_info)?[0];
            command_buffers.push(command_buffer);
        }

        let command_buffer = command_buffers[model_index];

        // Model
        let y = (((model_index % 2) as f32) * 2.5) - 1.25;
        let z = (((model_index / 2) as f32) * -2.0) + 1.0;

        let time = self.start.elapsed().as_secs_f32();

        let model = Mat4::from_translation(vec3(0.0, y, z)) * Mat4::from_axis_angle(
            vec3(0.0, 0.0, 1.0),
            Deg(90.0) * time
        );

        let model_bytes = &*slice_from_raw_parts(
            &model as *const Mat4 as *const u8,
            size_of::<Mat4>()
        );

        let opacity = (model_index + 1) as f32 * 0.25;
        let opacity_bytes = &opacity.to_ne_bytes()[..];

        // Commands
        let inheritance_info = vk::CommandBufferInheritanceInfo::builder()
            .render_pass(self.data.render_pass)
            .subpass(0)
            .framebuffer(self.data.framebuffers[image_index]);

        let info = vk::CommandBufferBeginInfo::builder()
            .flags(vk::CommandBufferUsageFlags::RENDER_PASS_CONTINUE)
            .inheritance_info(&inheritance_info);

        self.device.begin_command_buffer(command_buffer, &info)?;
        self.device.cmd_bind_pipeline(command_buffer, vk::PipelineBindPoint::GRAPHICS, self.data.pipeline);
        self.device.cmd_bind_vertex_buffers(command_buffer, 0, &[self.data.vertex_buffer], &[0]);
        self.device.cmd_bind_index_buffer(command_buffer, self.data.index_buffer, 0, vk::IndexType::UINT32);
        self.device.cmd_bind_descriptor_sets(
            command_buffer,
            vk::PipelineBindPoint::GRAPHICS,
            self.data.pipeline_layout,
            0,
            &[self.data.descriptor_sets[image_index]],
            &[],
        );
        self.device.cmd_push_constants(
            command_buffer,
            self.data.pipeline_layout,
            vk::ShaderStageFlags::VERTEX,
            0,
            model_bytes,
        );
        self.device.cmd_push_constants(
            command_buffer,
            self.data.pipeline_layout,
            vk::ShaderStageFlags::FRAGMENT,
            64,
            opacity_bytes,
        );
        self.device.cmd_draw_indexed(command_buffer, self.data.indices.len() as u32, 1, 0, 0, 0);

        self.device.end_command_buffer(command_buffer)?;

        Ok(command_buffer)
    }

    /// Updates the uniform buffer object for our Vulkan app.
    unsafe fn update_uniform_buffer(&self, image_index: usize) -> Result<()> {
        // Create MVP matrix

        // let time = self.start.elapsed().as_secs_f32();
        // let model = Mat4::from_axis_angle(
        //     vec3(0.0, 0.0, 1.0), 
        //     Deg(90.0) * time,
        // );

        let view = Mat4::look_at_rh(
            point3::<f32>(6.0, 0.0, 2.0),
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

        let ubo = UniformBufferObject { view, proj };

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
        
        color_objects::create_color_objects(&self.instance, &self.device, &mut self.data)?;
        
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
        self.data.command_pools
            .iter()
            .for_each(|p| 
                self.device.destroy_command_pool(*p, None));
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

        self.device.destroy_image_view(self.data.color_image_view, None);
        self.device.free_memory(self.data.color_image_memory, None);
        self.device.destroy_image(self.data.color_image, None);

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