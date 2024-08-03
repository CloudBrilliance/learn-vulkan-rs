//================================================
// Command Buffers
//================================================
use crate::app_data::AppData;
use crate::structs::Mat4;

use std::mem::size_of;
use std::ptr::slice_from_raw_parts;

use anyhow::Result;
use cgmath::{vec3, Deg};
use vulkanalia::prelude::v1_0::*;

pub unsafe fn create_command_buffers(device: &Device, data: &mut AppData) -> Result<()> {
    // Allocate
    let allocate_info = vk::CommandBufferAllocateInfo::builder()
        .command_pool(data.command_pool)
        .level(vk::CommandBufferLevel::PRIMARY)
        .command_buffer_count(data.framebuffers.len() as u32);

    data.command_buffers = device.allocate_command_buffers(&allocate_info)?;

    // Push Constants
    let model = Mat4::from_axis_angle(vec3(0.0, 0.0, 1.0), Deg(0.0));

    let model_bytes = &*slice_from_raw_parts(&model as *const Mat4 as *const u8, size_of::<Mat4>());

    let opacity = 0.25f32;
    let opacity_bytes = &opacity.to_ne_bytes()[..];

    // Commands
    for (i, command_buffer) in data.command_buffers.iter().enumerate() {
        let info = vk::CommandBufferBeginInfo::builder();

        device.begin_command_buffer(*command_buffer, &info)?;

        let render_area = vk::Rect2D::builder()
            .offset(vk::Offset2D::default())
            .extent(data.swapchain_extent);

        let color_clear_value = vk::ClearValue {
            color: vk::ClearColorValue {
                float32: [0.0, 0.0, 0.0, 1.0],
            },
        };

        let depth_clear_value = vk::ClearValue {
            depth_stencil: vk::ClearDepthStencilValue { 
                depth: 1.0, 
                stencil: 0 
            },
        };

        let clear_values = &[color_clear_value, depth_clear_value];
        let info = vk::RenderPassBeginInfo::builder()
            .render_pass(data.render_pass)
            .framebuffer(data.framebuffers[i])
            .render_area(render_area)
            .clear_values(clear_values);

        device.cmd_begin_render_pass(*command_buffer, &info, vk::SubpassContents::INLINE);
        
        // bind
        device.cmd_bind_pipeline(*command_buffer, vk::PipelineBindPoint::GRAPHICS, data.pipeline);
        device.cmd_bind_vertex_buffers(*command_buffer, 0, &[data.vertex_buffer], &[0]);
        device.cmd_bind_index_buffer(*command_buffer, data.index_buffer, 0, vk::IndexType::UINT32);

        device.cmd_bind_descriptor_sets(
            *command_buffer,
            vk::PipelineBindPoint::GRAPHICS,
            data.pipeline_layout,
            0,
            &[data.descriptor_sets[i]],
            &[],
        );
        
        device.cmd_push_constants(
            *command_buffer,
            data.pipeline_layout,
            vk::ShaderStageFlags::VERTEX,
            0,
            model_bytes,
        );
        device.cmd_push_constants(
            *command_buffer,
            data.pipeline_layout,
            vk::ShaderStageFlags::FRAGMENT,
            64,
            opacity_bytes,
        );

        // draw (use index buffer)
        device.cmd_draw_indexed(*command_buffer, data.indices.len() as u32, 1, 0, 0, 0);
        
        device.cmd_end_render_pass(*command_buffer);

        device.end_command_buffer(*command_buffer)?;
    }

    Ok(())
}