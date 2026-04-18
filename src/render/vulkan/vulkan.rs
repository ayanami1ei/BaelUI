#![allow(unused)]

use std::collections::HashSet;

use anyhow::{Result, anyhow};
use log::info;
use vulkanalia::Version;
use vulkanalia::prelude::v1_0::*;
use vulkanalia::vk::ExtDebugUtilsExtension;
use vulkanalia::window as vk_window;
use winit::window::Window;

use crate::app;
use crate::app::data::Data;
use crate::render::vulkan::callback::callback;

pub(crate) struct Vulkan {
    portability_macos_version: Version,
    validation_enable: bool,
    validation_layer: Option<vk::ExtensionName>,
}

impl Vulkan {
    pub(crate) fn new() -> Self {
        let mut res = Self {
            portability_macos_version: Version::new(1, 3, 216),
            validation_enable: cfg!(debug_assertions),
            validation_layer: None,
        };

        if res.validation_enable {
            res.validation_layer = Some(vk::ExtensionName::from_bytes(
                b"VK_LAYER_KHRONOS_validation",
            ));
        }

        res
    }

    #[inline(always)]
    fn get_validation_layers(&self, entry: &Entry) -> Result<Vec<*const i8>> {
        let available_layers = unsafe { entry.enumerate_instance_layer_properties() }?
            .iter()
            .map(|l| l.layer_name)
            .collect::<HashSet<_>>();

        if self.validation_enable && !available_layers.contains(&self.validation_layer.unwrap()) {
            return Err(anyhow!("Validation layer requested but not supported."));
        }

        let layers = if self.validation_enable {
            vec![self.validation_layer.unwrap().as_ptr()]
        } else {
            Vec::new()
        };

        Ok(layers)
    }

    pub(crate) fn create_instance(
        &self,
        window: &Window,
        entry: &Entry,
        data: &mut Data,
    ) -> Result<Instance> {
        // 应用信息，可选
        let app_info = vk::ApplicationInfo::builder()
            .application_name(b"bael\0")
            .application_version(vk::make_version(0, 0, 0))
            .engine_name(b"No Engine\0")
            .engine_version(vk::make_version(0, 0, 0))
            .api_version(vk::make_version(0, 0, 0));

        let layers = self.get_validation_layers(entry)?;

        // 全局拓展和校验层
        let mut extensions = vk_window::get_required_instance_extensions(window)
            .iter()
            .map(|e| e.as_ptr())
            .collect::<Vec<_>>();

        if self.validation_enable {
            extensions.push(vk::EXT_DEBUG_UTILS_EXTENSION.name.as_ptr());
        }

        let flags =
            if cfg!(target_os = "macos") && entry.version()? > self.portability_macos_version {
                info!("Enabling extensions for macOS Vulkan Portability");
                extensions.push(
                    vk::KHR_GET_PHYSICAL_DEVICE_PROPERTIES2_EXTENSION
                        .name
                        .as_ptr(),
                );
                extensions.push(vk::KHR_PORTABILITY_ENUMERATION_EXTENSION.name.as_ptr());
                vk::InstanceCreateFlags::ENUMERATE_PORTABILITY_KHR
            } else {
                vk::InstanceCreateFlags::empty()
            };

        let mut info = vk::InstanceCreateInfo::builder()
            .application_info(&app_info)
            .enabled_layer_names(&layers)
            .enabled_extension_names(&extensions)
            .flags(flags);

        let mut debug_info = vk::DebugUtilsMessengerCreateInfoEXT::builder()
            .message_severity(vk::DebugUtilsMessageSeverityFlagsEXT::all())
            .message_type(
                vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
                    | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION
                    | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE,
            )
            .user_callback(Some(callback));

        if self.validation_enable {
            info = info.push_next(&mut debug_info);
        }

        let instance = unsafe { entry.create_instance(&info, None) }?;

        data.messenger = unsafe { instance.create_debug_utils_messenger_ext(&debug_info, None) }?;

        Ok(instance)
    }
}
