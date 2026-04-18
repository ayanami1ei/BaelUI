#![allow(unused)]

use anyhow::{Result, anyhow};
use vulkanalia::{
    Instance,
    vk::{self, InstanceV1_0},
};

use crate::error::suitability_error::SuitabilityError;

#[derive(Debug, Clone, Default)]
pub(crate) struct Data {
    pub(crate) messenger: vk::DebugUtilsMessengerEXT,
    physical_device: vk::PhysicalDevice,
}

impl Data {
    pub(crate) fn pick_physical_device(&mut self, instance: &Instance) -> Result<()> {
        Ok(())
    }

    pub(crate) fn check_device_suitability(&self, instance: &Instance) -> Result<()> {
        let properties = unsafe { instance.get_physical_device_properties(self.physical_device) };
        if properties.device_type != vk::PhysicalDeviceType::DISCRETE_GPU {
            return Err(anyhow!(SuitabilityError(
                "Only discrete GPUs are supported."
            )));
        }

        let features = unsafe { instance.get_physical_device_features(self.physical_device) };
        if features.geometry_shader != vk::TRUE {
            return Err(anyhow!(SuitabilityError(
                "Missing geometry shader support."
            )));
        }

        Ok(())
    }
}
