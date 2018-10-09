use ash::version::{DeviceV1_0, V1_0};
use ash::vk;
use ash::Device;
use std::ptr;

use super::pipeline::Pipeline;

pub struct CommandPool {
    pub command_pool: vk::CommandPool,
    pub command_buffers: Vec<vk::CommandBuffer>,
}

impl CommandPool {
    pub fn new(device: &Device<V1_0>, swapchain_buffers_len: u32, queue_index: u32) -> Self {
        let command_pool_info = vk::CommandPoolCreateInfo {
            s_type: vk::StructureType::CommandPoolCreateInfo,
            p_next: ptr::null(),
            flags: Default::default(),
            queue_family_index: queue_index as u32,
        };

        let command_pool = unsafe {
            device
                .create_command_pool(&command_pool_info, None)
                .expect("Unable to create command pool")
        };

        let command_buffer_alloc_info = vk::CommandBufferAllocateInfo {
            s_type: vk::StructureType::CommandBufferAllocateInfo,
            p_next: ptr::null(),
            level: vk::CommandBufferLevel::Primary,
            command_pool: command_pool,
            command_buffer_count: swapchain_buffers_len,
        };

        let command_buffers = unsafe {
            device
                .allocate_command_buffers(&command_buffer_alloc_info)
                .expect("Unable to allocate command buffers")
        };

        CommandPool {
            command_pool,
            command_buffers,
        }
    }

    pub fn setup_command_buffers(
        &self,
        device: &Device<V1_0>,
        swapchain_buffers: &Vec<vk::Framebuffer>,
        graphics_pipelines: &Pipeline,
        surface_resolution: vk::Extent2D,
    ) -> Result<(), ()> {
        // let command_buffers = self
        //     .create_command_buffers(device, swapchain_buffers.len())
        //     .unwrap();

        self.command_buffers.iter().for_each(|&buffer| {
            let begin_info = vk::CommandBufferBeginInfo {
                s_type: vk::StructureType::CommandBufferBeginInfo,
                p_next: ptr::null(),
                flags: vk::COMMAND_BUFFER_USAGE_SIMULTANEOUS_USE_BIT,
                p_inheritance_info: ptr::null(),
            };

            unsafe {
                device
                    .begin_command_buffer(buffer, &begin_info)
                    .expect("Unable to begin buffer");
            };

            let clear_values = [vk::ClearValue {
                color: vk::ClearColorValue {
                    float32: [0.0, 0.0, 0.0, 0.0],
                },
            }];

            swapchain_buffers.iter().for_each(|&framebuffer| {
                let render_pass_info = vk::RenderPassBeginInfo {
                    s_type: vk::StructureType::RenderPassBeginInfo,
                    p_next: ptr::null(),
                    render_pass: graphics_pipelines.render_pass.render_pass,
                    framebuffer: framebuffer,
                    render_area: vk::Rect2D {
                        offset: vk::Offset2D { x: 0, y: 0 },
                        extent: surface_resolution.clone(),
                    },
                    clear_value_count: 1,
                    p_clear_values: clear_values.as_ptr(),
                };

                let graphics_pipeline = graphics_pipelines.graphics_pipelines.get(0).unwrap();

                unsafe {
                    device.cmd_begin_render_pass(
                        buffer,
                        &render_pass_info,
                        vk::SubpassContents::Inline,
                    );

                    device.cmd_bind_pipeline(
                        buffer,
                        vk::PipelineBindPoint::Graphics,
                        *graphics_pipeline,
                    );

                    // TODO: implement other commands
                    device.cmd_draw(buffer, 3, 1, 0, 0);

                    device.cmd_end_render_pass(buffer);
                }
            });

            unsafe {
                device
                    .end_command_buffer(buffer)
                    .expect("Unable to record command buffer");
            }
        });

        Ok(())
    }
}
