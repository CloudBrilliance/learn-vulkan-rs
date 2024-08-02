//================================================
// Buffers
//================================================

use crate::app_data::AppData;
use crate::structs::{Vertex, UniformBufferObject};
use crate::shared;

use std::mem::size_of;
use std::ptr::copy_nonoverlapping as memcpy;

use anyhow::Result;
use cgmath::{vec2, vec3};
use vulkanalia::prelude::v1_0::*;

#[rustfmt::skip]
static VERTICES: [Vertex; 8] = [
    Vertex::new(
        vec3(-0.5, -0.5, 0.0),
        vec3(1.0, 0.0, 0.0),
        vec2(1.0, 0.0)),
    Vertex::new(
        vec3(0.5, -0.5, 0.0), 
        vec3(0.0, 1.0, 0.0), 
        vec2(0.0, 0.0)),
    Vertex::new(
        vec3(0.5, 0.5, 0.0), 
        vec3(0.0, 0.0, 1.0), 
        vec2(0.0, 1.0)),
    Vertex::new(
        vec3(-0.5, 0.5, 0.0),
        vec3(1.0, 1.0, 1.0),
        vec2(1.0, 1.0)),
    //
    Vertex::new(
        vec3(-0.5, -0.5, -0.5),
        vec3(1.0, 0.0, 0.0), 
        vec2(1.0, 0.0)),
    Vertex::new(
        vec3(0.5, -0.5, -0.5), 
        vec3(0.0, 1.0, 0.0), 
        vec2(0.0, 0.0)),
    Vertex::new(
        vec3(0.5, 0.5, -0.5), 
        vec3(0.0, 0.0, 1.0), 
        vec2(0.0, 1.0)),
    Vertex::new(
        vec3(-0.5, 0.5, -0.5), 
        vec3(1.0, 1.0, 1.0), 
        vec2(1.0, 1.0)),
];

#[rustfmt::skip]
pub const INDICES: &[u16] = &[
    0, 1, 2, 2, 3, 0,  //
    4, 5, 6, 6, 7, 4
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


pub unsafe fn create_index_buffer(instance: &Instance, device: &Device, data: &mut AppData) -> Result<()> {
    //================================================
    // Stage Buffer: CPU accessible
    //================================================
    // 1. Create buffer
    let size = (size_of::<u16>() * INDICES.len()) as u64;

    let (staging_buffer, staging_buffer_memory) = shared::create_buffer(
        instance,
        device,
        data,
        size,
        vk::BufferUsageFlags::TRANSFER_SRC,
        vk::MemoryPropertyFlags::HOST_COHERENT | vk::MemoryPropertyFlags::HOST_VISIBLE,
    )?;

    // Copy data
    let memory = device.map_memory(staging_buffer_memory, 0, size, vk::MemoryMapFlags::empty())?;

    memcpy(INDICES.as_ptr(), memory.cast(), INDICES.len());

    device.unmap_memory(staging_buffer_memory);

    //================================================
    // Index Buffer: CPU is not accessible
    //================================================
    // 1. Create buffer 
    let (index_buffer, index_buffer_memory) = shared::create_buffer(
        instance,
        device,
        data,
        size,
        vk::BufferUsageFlags::TRANSFER_DST | vk::BufferUsageFlags::INDEX_BUFFER,
        vk::MemoryPropertyFlags::DEVICE_LOCAL,
    )?;

    data.index_buffer = index_buffer;
    data.index_buffer_memory = index_buffer_memory;

    // Copy data
    shared::copy_buffer(device, data, staging_buffer, index_buffer, size)?;

    //================================================
    // Cleanup
    //================================================
    device.destroy_buffer(staging_buffer, None);
    device.free_memory(staging_buffer_memory, None);

    Ok(())
}


pub unsafe fn create_uniform_buffers(instance: &Instance, device: &Device, data: &mut AppData) -> Result<()> {
    data.uniform_buffers.clear();
    data.uniform_buffers_memory.clear();

    for _ in 0..data.swapchain_images.len() {
        let (uniform_buffer, uniform_buffer_memory) = shared::create_buffer(
            instance,
            device,
            data,
            size_of::<UniformBufferObject>() as u64,
            vk::BufferUsageFlags::UNIFORM_BUFFER,
            vk::MemoryPropertyFlags::HOST_COHERENT | vk::MemoryPropertyFlags::HOST_VISIBLE,
        )?;

        data.uniform_buffers.push(uniform_buffer);
        data.uniform_buffers_memory.push(uniform_buffer_memory);
    }

    Ok(())
}
