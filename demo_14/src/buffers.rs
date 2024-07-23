//================================================
// Buffers
//================================================

use crate::app_data::AppData;
use crate::structs::Vertex;
use crate::shared;

use std::mem::size_of;
use std::ptr::copy_nonoverlapping as memcpy;

use anyhow::Result;
use cgmath::{vec2, vec3};
use vulkanalia::prelude::v1_0::*;

#[rustfmt::skip]
static VERTICES: [Vertex; 3] = [
    Vertex::new(vec2( 0.0, -0.5), vec3(1.0, 1.0, 1.0)),
    Vertex::new(vec2( 0.5,  0.5), vec3(0.0, 1.0, 0.0)),
    Vertex::new(vec2(-0.5,  0.5), vec3(0.0, 0.0, 1.0)),
];

pub unsafe fn create_vertex_buffer(instance: &Instance, device: &Device, data: &mut AppData) -> Result<()> {
    // 1. Create Buffer
    let buffer_info = vk::BufferCreateInfo::builder()
        .size((size_of::<Vertex>() * VERTICES.len()) as u64)
        .usage(vk::BufferUsageFlags::VERTEX_BUFFER)
        .sharing_mode(vk::SharingMode::EXCLUSIVE);

    data.vertex_buffer = device.create_buffer(&buffer_info, None)?;

    // 2. Allocate Memory
    let requirements = device.get_buffer_memory_requirements(data.vertex_buffer);

    let memory_info = vk::MemoryAllocateInfo::builder()
        .allocation_size(requirements.size)
        .memory_type_index(shared::get_memory_type_index(
            instance,
            data,
            vk::MemoryPropertyFlags::HOST_COHERENT | vk::MemoryPropertyFlags::HOST_VISIBLE,
            requirements,
        )?);

    data.vertex_buffer_memory = device.allocate_memory(&memory_info, None)?;

    device.bind_buffer_memory(data.vertex_buffer, data.vertex_buffer_memory, 0)?;

    // 3. Copy Data
    let memory = device.map_memory(
        data.vertex_buffer_memory,
        0,
        buffer_info.size,
        vk::MemoryMapFlags::empty(),
    )?;

    memcpy(VERTICES.as_ptr(), memory.cast(), VERTICES.len());

    device.unmap_memory(data.vertex_buffer_memory);

    Ok(())
}