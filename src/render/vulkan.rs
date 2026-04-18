#![allow(unused)]

use anyhow::Result;
use log::info;
use vulkanalia::prelude::v1_0::*;
use vulkanalia::window as vk_window;
use winit::window::Window;
use vulkanalia::Version;

pub(crate) struct Vulkan {
    portability_macos_version: Version,
}

impl Vulkan {
    pub(crate) fn new() -> Self {
        Self {
            portability_macos_version: Version::new(1, 3, 216),
        }
    }

    pub(crate) fn create_instance(&self, window: &Window, entry: &Entry) -> Result<Instance> {
        // 应用信息，可选
        let app_info = vk::ApplicationInfo::builder()
            .application_name(b"bael\0")
            .application_version(vk::make_version(0, 0, 0))
            .engine_name(b"No Engine\0")
            .engine_version(vk::make_version(0, 0, 0))
            .api_version(vk::make_version(0, 0, 0));

        // 全局拓展和校验层
        let mut extensions = vk_window::get_required_instance_extensions(window)
            .iter()
            .map(|e| e.as_ptr())
            .collect::<Vec<_>>();

        let flags=if 
            cfg!(target_os="macos") &&
            entry.version()? > self.portability_macos_version {
            info!("Enabling extensions for macOS Vulkan Portability");
            extensions.push(vk::KHR_GET_PHYSICAL_DEVICE_PROPERTIES2_EXTENSION.name.as_ptr());
            extensions.push(vk::KHR_PORTABILITY_ENUMERATION_EXTENSION.name.as_ptr());
            vk::InstanceCreateFlags::ENUMERATE_PORTABILITY_KHR
        } else {
            vk::InstanceCreateFlags::empty()
        };

        let info = vk::InstanceCreateInfo::builder()
            .application_info(&app_info)
            .enabled_extension_names(&extensions)
            .flags(flags);

        Ok(unsafe { entry.create_instance(&info, None) }?)
    }
}
