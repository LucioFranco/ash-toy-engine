use ash::version::{DeviceV1_0, V1_0};
use ash::vk;
use ash::Device;
use std::ptr;

pub struct Semaphore {
    pub semaphore: vk::Semaphore,
}

impl Semaphore {
    pub fn new(device: &Device<V1_0>) -> Self {
        let semaphore_create_info = vk::SemaphoreCreateInfo {
            s_type: vk::StructureType::SemaphoreCreateInfo,
            p_next: ptr::null(),
            flags: Default::default(),
        };

        let semaphore = unsafe {
            device
                .create_semaphore(&semaphore_create_info, None)
                .expect("Unable to create image semaphore")
        };

        Semaphore { semaphore }
    }
}
