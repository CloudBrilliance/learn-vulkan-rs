// SPDX-License-Identifier: Apache-2.0
// 使用 winit 库来创建一个窗口，并在窗口上运行一个 Vulkan 应用

#![allow(
    dead_code,                              // 允许未使用的代码
    unused_variables,                       // 允许未使用的变量
    clippy::manual_slice_size_calculation,  // 允许手动计算切片的大小
    clippy::too_many_arguments,             // 允许函数具有过多的参数
    clippy::unnecessary_wraps,              // 允许不必要的包装
)]  // 告诉编译器忽略一些特定的警告

use anyhow::Result;                              // Result 通用的错误处理类型
use winit::dpi::LogicalSize;                     // 用于表示窗口的逻辑大小
use winit::event::{Event, WindowEvent};          // 用于处理窗口事件
use winit::event_loop::{ControlFlow, EventLoop}; // 用于控制事件循环
use winit::window::{Window, WindowBuilder};      // 用于创建和管理窗口
use mylib::app::App;

#[rustfmt::skip]
fn main() -> Result<()> {       // 返回 Result 类型，代表函数可能出错
    pretty_env_logger::init();  // 初始化环境变量驱动的日志记录器（会把日志打印到控制台）

    // ----- Window -----
    let event_loop: EventLoop<()> = EventLoop::new();  // 创建一个新的事件循环
    let window: Window = WindowBuilder::new()
        .with_title("Vulkan Tutorial (Rust)")
        .with_inner_size(LogicalSize::new(1024, 768))
        .build(&event_loop)?;           // 创建一个新的窗口

    // ----- App -----
    let mut app: App = unsafe { App::create(&window)? };  // 创建一个新的 Vulkan App
    let mut destroying: bool = false;                     // 用于记录 App 是否将要销毁
    
    // 启动事件循环，并提供一个闭包来处理每个事件
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;
        match event {
            // Vulkan App 没有正在被销毁
            Event::MainEventsCleared if !destroying => unsafe { 
                app.render(&window)  // 渲染一帧
            }.unwrap(),
            
            // 窗口关闭请求事件
            Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => {
                destroying = true;
                *control_flow = ControlFlow::Exit;
                unsafe { app.destroy(); }
            }
            _ => {}
        }
    });
}
