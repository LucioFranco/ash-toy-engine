use ash::version::{DeviceV1_0, V1_0};
use ash::vk;
use ash::Device;
use std::ptr;

pub struct Fence {
    pub fence: vk::Fence,
}

impl Fence {
    pub fn new(device: &Device<V1_0>) -> Self {
        let fence_info = vk::FenceCreateInfo {
            s_type: vk::StructureType::FenceCreateInfo,
            p_next: ptr::null(),
            flags: vk::FENCE_CREATE_SIGNALED_BIT,
        };

        let fence = unsafe { device.create_fence(&fence_info, None).unwrap() };

        Fence { fence }
    }
}
