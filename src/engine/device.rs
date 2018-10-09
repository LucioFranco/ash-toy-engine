use ash;
use ash::extensions::Swapchain;
use ash::version::{InstanceV1_0, V1_0};
use ash::vk;
use std::ptr;

pub struct Device {
    pub device: ash::Device<V1_0>,
}

impl Device {
    pub fn new(
        instance: &ash::Instance<V1_0>,
        layer_names_raw: Vec<*const i8>,
        device_extension_names_raw: Vec<*const i8>,
        queue_index: u32,
        pdevice: vk::PhysicalDevice,
    ) -> Result<Device, ()> {
        let priorities = [1.0];
        let queue_info = vk::DeviceQueueCreateInfo {
            s_type: vk::StructureType::DeviceQueueCreateInfo,
            p_next: ptr::null(),
            flags: Default::default(),
            queue_family_index: queue_index,
            p_queue_priorities: priorities.as_ptr(),
            queue_count: priorities.len() as u32,
        };

        let device_features = instance.get_physical_device_features(pdevice);
        let device_extension_names_raw = [Swapchain::name().as_ptr()];
        let device_create_info = vk::DeviceCreateInfo {
            s_type: vk::StructureType::DeviceCreateInfo,
            p_next: ptr::null(),
            flags: Default::default(),
            queue_create_info_count: 1,
            p_enabled_features: &device_features,
            enabled_layer_count: layer_names_raw.len() as u32,
            pp_enabled_layer_names: layer_names_raw.as_ptr(),
            enabled_extension_count: device_extension_names_raw.len() as u32,
            pp_enabled_extension_names: device_extension_names_raw.as_ptr(),
            p_queue_create_infos: &queue_info,
        };

        let device = unsafe {
            instance
                .create_device(pdevice, &device_create_info, None)
                .expect("Unable to create device")
        };

        Ok(Device { device })
    }
}
