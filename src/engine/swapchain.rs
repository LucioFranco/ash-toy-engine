use ash::extensions::Swapchain;
use ash::vk;
use std::ptr;

pub fn create_swapchain(
    swapchain_loader: &Swapchain,
    surface: vk::SurfaceKHR,
    desired_image_count: u32,
    surface_format: &vk::SurfaceFormatKHR,
    surface_resolution: vk::Extent2D,
    pre_transform: vk::SurfaceTransformFlagsKHR,
    present_mode: vk::PresentModeKHR,
) -> Result<vk::SwapchainKHR, ()> {
    let swapchain_create_info = vk::SwapchainCreateInfoKHR {
        s_type: vk::StructureType::SwapchainCreateInfoKhr,
        p_next: ptr::null(),
        flags: Default::default(),
        surface: surface,
        min_image_count: desired_image_count,
        image_color_space: surface_format.color_space,
        image_format: surface_format.format,
        image_extent: surface_resolution.clone(),
        image_usage: vk::IMAGE_USAGE_COLOR_ATTACHMENT_BIT,
        image_array_layers: 1,
        image_sharing_mode: vk::SharingMode::Exclusive,
        queue_family_index_count: 0,
        p_queue_family_indices: ptr::null(),
        pre_transform: pre_transform,
        composite_alpha: vk::COMPOSITE_ALPHA_OPAQUE_BIT_KHR,
        present_mode,
        clipped: 1,
        old_swapchain: vk::SwapchainKHR::null(),
    };

    let swapchain = unsafe {
        swapchain_loader
            .create_swapchain_khr(&swapchain_create_info, None)
            .unwrap()
    };

    Ok(swapchain)
}
