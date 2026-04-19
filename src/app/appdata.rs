#![allow(unused)]

use anyhow::{Result, anyhow};
use log::{info, warn};
use vulkanalia::{
    Instance,
    vk::{self, InstanceV1_0, PhysicalDevice},
};

use crate::{error::suitability_error::SuitabilityError, render::vulkan::queue_family_indices::QueueFamilyIndices};

#[derive(Debug, Clone, Default)]
pub(crate) struct AppData {
    pub(crate) surface: vk::SurfaceKHR,
    pub(crate) messenger: vk::DebugUtilsMessengerEXT,
    pub(crate) physical_device: vk::PhysicalDevice,

    pub(crate) present_queue:vk::Queue,

    device_extensions: Vec<vk::ExtensionName>,
}

impl AppData {
    pub(crate) fn pick_physical_device(&mut self, instance: &Instance) -> Result<()> {
        for physical_device in unsafe { instance.enumerate_physical_devices() }? {
        let properties = unsafe { instance.get_physical_device_properties(physical_device) };

        self.physical_device = physical_device;
        if let Err(error) = self.check_physical_device(instance) {
            warn!("Skipping physical device (`{}`): {}", properties.device_name, error);
        } else {
            info!("Selected physical device (`{}`).", properties.device_name);
            self.physical_device = physical_device;
            return Ok(());
        }
    }

    Err(anyhow!("Failed to find suitable physical device."))
    }

    pub(crate) fn check_physical_device(&self, instance: &Instance) -> Result<()> {
        QueueFamilyIndices::get(instance, self)?;

        Ok(())
    }
}
