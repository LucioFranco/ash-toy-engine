use ash::version::{DeviceV1_0, V1_0};
use ash::vk;
use ash::Device;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::ptr;

use engine::pipeline::ShaderType;

pub struct Shader {
    pub module: vk::ShaderModule,
    pub shader_type: ShaderType,
}

impl Shader {
    pub fn load<P: AsRef<Path>>(device: &Device<V1_0>, path: P, shader_type: ShaderType) -> Self {
        let file = File::open(path).unwrap();
        let bytes = file.bytes().filter_map(|byte| byte.ok()).collect();

        let module = Shader::create_module(device, bytes);

        Shader {
            module,
            shader_type,
        }
    }

    fn create_module(device: &Device<V1_0>, bytes: Vec<u8>) -> vk::ShaderModule {
        let shader_module_create_info = vk::ShaderModuleCreateInfo {
            s_type: vk::StructureType::ShaderModuleCreateInfo,
            p_next: ptr::null(),
            flags: Default::default(),
            code_size: bytes.len(),
            p_code: bytes.as_ptr() as *const u32,
        };

        unsafe {
            device
                .create_shader_module(&shader_module_create_info, None)
                .expect("Unable to create shader")
        }
    }

    pub fn create_stage(&self) -> vk::PipelineShaderStageCreateInfo {
        vk::PipelineShaderStageCreateInfo {
            s_type: vk::StructureType::PipelineShaderStageCreateInfo,
            p_next: ptr::null(),
            flags: Default::default(),
            stage: self.shader_type.to_vulkan(),
            p_specialization_info: ptr::null(),
            p_name: "main".as_ptr() as *const i8,
            module: self.module,
        }
    }
}
