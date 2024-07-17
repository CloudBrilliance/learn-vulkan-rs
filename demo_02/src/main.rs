// SPDX-License-Identifier: Apache-2.0
// 2. 物理设备

use mylib::app::App;

use anyhow::Result;
use winit::dpi::LogicalSize;
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::{Window, WindowBuilder};

#[rustfmt::skip]
fn main() -> Result<()> {
    pretty_env_logger::init();

    // --- Window ---
    let event_loop: EventLoop<()> = EventLoop::new();
    let window: Window = WindowBuilder::new()
        .with_title("Vulkan Tutorial (Rust)")
        .with_inner_size(LogicalSize::new(1024, 768))
        .build(&event_loop)?;

    // --- App ---
    let mut app: App = unsafe { App::create(&window)? };
    let mut destroying: bool = false;
    
    // --- Event ---
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;
        match event {
            Event::MainEventsCleared if !destroying => unsafe { 
                app.render(&window)
            }.unwrap(),
            
            Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => {
                destroying = true;
                *control_flow = ControlFlow::Exit;
                unsafe { app.destroy(); }
            }
            _ => {}
        }
    });
}
