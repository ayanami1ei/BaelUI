#![allow(unused)]

use std::collections::HashSet;

use anyhow::anyhow;
use vulkanalia::prelude::v1_0::Instance;
use vulkanalia::vk::{DeviceV1_0, KhrSurfaceExtension};
use vulkanalia::window as vk_window;
use vulkanalia::{
    Device, Entry,
    loader::{LIBRARY, LibloadingLoader},
    vk::{self, HasBuilder, InstanceV1_0},
};
use winit::window::Window;

use anyhow::Result;

use crate::{
    app::appdata::AppData,
    render::vulkan::{queue_family_indices::QueueFamilyIndices, vulkan::Vulkan},
};

#[derive(Debug, Clone)]
pub(crate) struct App<'a> {
    entry: Entry,
    instance: Instance,
    data: AppData<'a>,
    device: Device,
}

impl<'a> App<'a> {
    fn create_logic_device(
        vulkan: &Vulkan,
        entry: &Entry,
        instance: &Instance,
        data: &mut AppData<'a>,
    ) -> Result<Device> {
        let indices = QueueFamilyIndices::get(instance, data)?;

        let mut unique_indices = HashSet::new();
        unique_indices.insert(indices.graphics);
        unique_indices.insert(indices.present);

        let queue_priorities = &[1.0];
        let queue_infos = unique_indices
            .iter()
            .map(|i| {
                vk::DeviceQueueCreateInfo::builder()
                    .queue_family_index(*i)
                    .queue_priorities(queue_priorities)
            })
            .collect::<Vec<_>>();

        let queue_properties = &[1.0];
        let queue_info = vk::DeviceQueueCreateInfo::builder()
            .queue_family_index(indices.graphics)
            .queue_priorities(queue_properties);

        let layers = if cfg!(debug_assertions) {
            vec![vulkan.validation_layer.unwrap().as_ptr()]
        } else {
            vec![]
        };

        let mut extensions = vec![];

        // Required by Vulkan SDK on macOS since 1.3.216.
        if cfg!(target_os = "macos") && entry.version()? >= vulkan.portability_macos_version {
            extensions.push(vk::KHR_PORTABILITY_SUBSET_EXTENSION.name.as_ptr());
        }

        let features = vk::PhysicalDeviceFeatures::builder();

        let info = vk::DeviceCreateInfo::builder()
            .queue_create_infos(&queue_infos)
            .enabled_layer_names(&layers)
            .enabled_extension_names(&extensions)
            .enabled_features(&features);

        let device = unsafe { instance.create_device(data.physical_device, &info, None) }?;
        data.present_queue = unsafe { device.get_device_queue(indices.present, 0) };
        Ok(device)
    }

    pub(crate) fn new(vulkan: &Vulkan, window: &Window) -> Result<Self> {
        let loader = unsafe { LibloadingLoader::new(LIBRARY)? };
        let entry = unsafe { Entry::new(loader).map_err(|b| anyhow!("{}", b))? };

        let mut data = AppData::default();
        let instance = vulkan.create_instance(window, &entry, &mut data)?;

        data.surface = unsafe { vk_window::create_surface(&instance, window, window)? };

        data.pick_physical_device(&instance)?;

        let device = Self::create_logic_device(&vulkan, &entry, &instance, &mut data)?;
        data.pick_physical_device(&instance)?;

        Ok(Self {
            entry,
            instance,
            data,
            device,
        })
    }

    pub(crate) fn render(&mut self, window: &Window) -> Result<()> {
        Ok(())
    }

    pub(crate) fn destroy(&mut self) {
        unsafe { self.device.destroy_device(None) };

        #[cfg(debug_assertions)]
        {
            use vulkanalia::vk::ExtDebugUtilsExtension;

            unsafe {
                self.instance
                    .destroy_debug_utils_messenger_ext(self.data.messenger, None)
            };
        }

        unsafe {
            self.instance.destroy_surface_khr(self.data.surface, None);
            self.instance.destroy_instance(None)
        };
    }
}
