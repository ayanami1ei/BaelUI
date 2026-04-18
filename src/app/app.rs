#![allow(unused)]

use vulkanalia::{Entry, Instance, loader::{LIBRARY, LibloadingLoader}, vk::InstanceV1_0};
use winit::window::Window;
use anyhow::anyhow;

use anyhow::Result;

use crate::render::vulkan::Vulkan;

#[derive(Debug, Clone)]
pub(crate) struct App {
    entry:Entry,
    instance: Instance,
}

impl App{
    pub(crate) fn new(vulkan:&Vulkan, window: &Window) -> Result<Self> {
        let loader=unsafe { LibloadingLoader::new(LIBRARY)? };
        let entry=unsafe { 
            Entry::new(loader).map_err(|b| anyhow!("{}", b))? 
        };
        let instance=vulkan.create_instance(window, &entry)?;
        Ok(Self {entry, instance})
    }

    pub(crate) fn render(&mut self, window: &Window) -> Result<()> {
        Ok(())
    }

    pub(crate) fn destroy(&mut self) {
        unsafe { self.instance.destroy_instance(None) };
    }
}