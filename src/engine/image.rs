use ash::version::{DeviceV1_0, V1_0};
use ash::vk;
use ash::Device;
use std::ptr;

pub fn create_image_views(
    device: &Device<V1_0>,
    images: Vec<vk::Image>,
    surface_format: &vk::SurfaceFormatKHR,
) -> Result<Vec<vk::ImageView>, ()> {
    let image_views = images
        .iter()
        .map(|&image| {
            let create_view_info = vk::ImageViewCreateInfo {
                s_type: vk::StructureType::ImageViewCreateInfo,
                p_next: ptr::null(),
                flags: Default::default(),
                view_type: vk::ImageViewType::Type2d,
                format: surface_format.format,
                components: vk::ComponentMapping {
                    r: vk::ComponentSwizzle::R,
                    g: vk::ComponentSwizzle::G,
                    b: vk::ComponentSwizzle::B,
                    a: vk::ComponentSwizzle::A,
                },
                subresource_range: vk::ImageSubresourceRange {
                    aspect_mask: vk::IMAGE_ASPECT_COLOR_BIT,
                    base_mip_level: 0,
                    level_count: 1,
                    base_array_layer: 0,
                    layer_count: 1,
                },
                image: image,
            };

            unsafe {
                device
                    .create_image_view(&create_view_info, None)
                    .expect("Failed to create image view!")
            }
        }).collect::<Vec<_>>();

    Ok(image_views)
}
