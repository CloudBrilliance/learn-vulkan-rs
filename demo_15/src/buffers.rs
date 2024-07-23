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
    //================================================
    // Stage Buffer: CPU accessible
    //================================================
    // 1. Create Buffer
    let size = (size_of::<Vertex>() * VERTICES.len()) as u64;
    let (staging_buffer, staging_buffer_memory) = shared::create_buffer(
        instance,
        device,
        data,
        size,
        vk::BufferUsageFlags::TRANSFER_SRC,
        vk::MemoryPropertyFlags::HOST_COHERENT | vk::MemoryPropertyFlags::HOST_VISIBLE,
    )?;  // 1. create buffer, 2. allocate memory, 3. bind

    // 2. Copy Data: VERTICES -> staging_buffer_memory
    let memory = device.map_memory(
        staging_buffer_memory, 
        0, 
        size, 
        vk::MemoryMapFlags::empty()
    )?;
    memcpy(VERTICES.as_ptr(), memory.cast(), VERTICES.len());

    device.unmap_memory(staging_buffer_memory);

    //================================================
    // Vertex Buffer: CPU is not accessible
    //================================================
    // 1. Create Buffer
    let (vertex_buffer, vertex_buffer_memory) = shared::create_buffer(
        instance,
        device,
        data,
        size,
        vk::BufferUsageFlags::TRANSFER_DST | vk::BufferUsageFlags::VERTEX_BUFFER,
        vk::MemoryPropertyFlags::DEVICE_LOCAL,
    )?;

    data.vertex_buffer = vertex_buffer;
    data.vertex_buffer_memory = vertex_buffer_memory;

    // 2. Copy Data (staging_buffer -> vertex_buffer)
    shared::copy_buffer(device, data, staging_buffer, vertex_buffer, size)?;

    //================================================
    // Cleanup (Stage Buffer)
    //================================================
    device.destroy_buffer(staging_buffer, None);
    device.free_memory(staging_buffer_memory, None);

    Ok(())
}