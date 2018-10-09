use ash::version::{EntryV1_0, V1_0};
use ash::vk;
use ash::{Entry, Instance};
use std::ffi::CString;
use std::ptr;

pub fn create_instance(
    entry: &Entry<V1_0>,
    app_name: CString,
    engine_name: CString,
    layer_names_raw: &Vec<*const i8>,
    extension_names_raw: &Vec<*const i8>,
) -> Result<Instance<V1_0>, ()> {
    let app_info = vk::ApplicationInfo {
        s_type: vk::StructureType::ApplicationInfo,
        p_next: ptr::null(),
        p_application_name: app_name.as_ptr(),
        application_version: 0,
        p_engine_name: engine_name.as_ptr(),
        engine_version: 0,
        api_version: vk_make_version!(1, 0, 36),
    };

    let create_info = vk::InstanceCreateInfo {
        s_type: vk::StructureType::InstanceCreateInfo,
        p_application_info: &app_info,
        flags: Default::default(),
        p_next: ptr::null(),
        pp_enabled_layer_names: layer_names_raw.as_ptr(),
        enabled_layer_count: layer_names_raw.len() as u32,
        pp_enabled_extension_names: extension_names_raw.as_ptr(),
        enabled_extension_count: extension_names_raw.len() as u32,
    };

    let instance = unsafe {
        entry
            .create_instance(&create_info, None)
            .expect("failed to create instance!")
    };

    Ok(instance)
}
