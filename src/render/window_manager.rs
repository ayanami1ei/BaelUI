#![allow(unused)]

use winit::{
    event_loop::EventLoop,
    window::{Window, WindowBuilder},
};

use anyhow::Result;

pub(crate) struct WindowManager {
    window: Window,
}

impl WindowManager {
    pub(crate) fn new(
        event_loop: &EventLoop<()>,
        title: &String,
        width: i32,
        height: i32,
    ) -> Result<Self> {
        Ok(Self {
            window: WindowBuilder::new()
                .with_title(title)
                .with_inner_size(winit::dpi::LogicalSize::new(width, height))
                .build(event_loop)?,
        })
    }

    pub(crate) fn get_window(&self) -> &Window {
        &self.window
    }
}
