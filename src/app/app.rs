#![allow(unused)]

use vulkanalia::{Entry, loader::{LIBRARY, LibloadingLoader}, vk::InstanceV1_0};
use vulkanalia::prelude::v1_0::Instance;
use winit::window::Window;
use anyhow::anyhow;

use anyhow::Result;

use crate::{app::data::Data, render::vulkan::vulkan::Vulkan};

#[derive(Debug, Clone)]
pub(crate) struct App {
    entry:Entry,
    instance: Instance,
    data:Data,
}

impl App{
    pub(crate) fn new(vulkan:&Vulkan, window: &Window) -> Result<Self> {
        let loader=unsafe { LibloadingLoader::new(LIBRARY)? };
        let entry=unsafe { 
            Entry::new(loader).map_err(|b| anyhow!("{}", b))? 
        };

        let mut data=Data::default();
        let instance=vulkan.create_instance(window, &entry, &mut data)?;

        data.pick_physical_device(&instance)?;
        Ok(Self {entry, instance, data})
    }

    pub(crate) fn render(&mut self, window: &Window) -> Result<()> {
        Ok(())
    }

    pub(crate) fn destroy(&mut self) {
        #[cfg(debug_assertions)]{
            use vulkanalia::vk::ExtDebugUtilsExtension;

            unsafe { self.instance.destroy_debug_utils_messenger_ext(self.data.messenger, None) };
        }
        unsafe { self.instance.destroy_instance(None) };
    }
}