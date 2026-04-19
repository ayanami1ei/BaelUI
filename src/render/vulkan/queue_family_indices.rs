#![allow(unused)]

use vulkanalia::{Instance, vk};
use vulkanalia::vk::{InstanceV1_0, KhrSurfaceExtension};

use anyhow::{Result, anyhow};

use crate::app::appdata::AppData;
use crate::error::suitability_error::SuitabilityError;

pub(crate) struct QueueFamilyIndices {
    pub(crate) graphics: u32,
    pub(crate) present: u32,
}

impl QueueFamilyIndices {
    pub(crate) fn get(instance: &Instance, data: &AppData) -> Result<Self> {
        let properties =
            unsafe { instance.get_physical_device_queue_family_properties(data.physical_device) };

        let graphics = properties
            .iter()
            .position(|p| p.queue_flags.contains(vk::QueueFlags::GRAPHICS))
            .map(|(index)| index as u32);

        let mut present = None;
        for (index, properties) in properties.iter().enumerate(){
            if unsafe { instance.get_physical_device_surface_support_khr(
                data.physical_device, 
                index as u32, 
                data.surface) }?
            {
                present = Some(index as u32);
                break;
            }
        }

        if let (Some(graphics), Some(present)) = (graphics, present) {
            Ok(Self { graphics, present})
        } else {
            Err(anyhow!(SuitabilityError("Missing required queue families.")))
        }
    }
}
