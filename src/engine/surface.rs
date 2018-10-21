use ash::extensions::Surface;
use ash::version::{EntryV1_0, InstanceV1_0};
use ash::vk;
use std::ptr;
use winit::Window;

#[cfg(all(unix, not(target_os = "android"), not(target_os = "macos")))]
pub fn create_surface<E: EntryV1_0, I: InstanceV1_0>(
    entry: &E,
    instance: &I,
    window: &winit::Window,
) -> Result<vk::SurfaceKHR, vk::Result> {
    use winit::os::unix::WindowExt;
    let x11_display = window.get_xlib_display().unwrap();
    let x11_window = window.get_xlib_window().unwrap();
    let x11_create_info = vk::XlibSurfaceCreateInfoKHR {
        s_type: vk::StructureType::XlibSurfaceCreateInfoKhr,
        p_next: ptr::null(),
        flags: Default::default(),
        window: x11_window as vk::Window,
        dpy: x11_display as *mut vk::Display,
    };
    let xlib_surface_loader =
        XlibSurface::new(entry, instance).expect("Unable to load xlib surface");
    unsafe { xlib_surface_loader.create_xlib_surface_khr(&x11_create_info, None) }
}

#[cfg(target_os = "macos")]
use ash::extensions::MacOSSurface;
#[cfg(target_os = "macos")]
use cocoa::appkit::{NSView, NSWindow};
#[cfg(target_os = "macos")]
use cocoa::base::id as cocoa_id;
#[cfg(target_os = "macos")]
use metal::CoreAnimationLayer;
#[cfg(target_os = "macos")]
use objc::runtime::YES;
#[cfg(target_os = "macos")]
use std::mem;

#[cfg(target_os = "macos")]
pub fn create_surface<E: EntryV1_0, I: InstanceV1_0>(
    entry: &E,
    instance: &I,
    window: &Window,
) -> Result<vk::SurfaceKHR, vk::Result> {
    use std::mem;
    use winit::os::macos::WindowExt;

    unsafe {
        let wnd: cocoa_id = mem::transmute(window.get_nswindow());

        let layer = CoreAnimationLayer::new();

        layer.set_edge_antialiasing_mask(0);
        layer.set_presents_with_transaction(false);
        layer.remove_all_animations();

        let view = wnd.contentView();

        layer.set_contents_scale(view.backingScaleFactor());
        view.setLayer(mem::transmute(layer.as_ref()));
        view.setWantsLayer(YES);
    }

    let create_info = vk::MacOSSurfaceCreateInfoMVK {
        s_type: vk::StructureType::MacOSSurfaceCreateInfoMvk,
        p_next: ptr::null(),
        flags: Default::default(),
        p_view: window.get_nsview() as *const vk::types::c_void,
    };

    let macos_surface_loader =
        MacOSSurface::new(entry, instance).expect("Unable to load macOS surface");
    unsafe { macos_surface_loader.create_macos_surface_mvk(&create_info, None) }
}

#[cfg(target_os = "windows")]
pub fn create_surface<E: EntryV1_0, I: InstanceV1_0>(
    entry: &E,
    instance: &I,
    window: &Window,
) -> Result<vk::SurfaceKHR, vk::Result> {
    use ash::extensions::Win32Surface;
    use winapi::shared::windef::HWND;
    use winapi::um::winuser::GetWindow;
    use winit::os::windows::WindowExt;

    unsafe {
        let hwnd = window.get_hwnd() as HWND;
        let hinstance = unsafe { GetWindow(hwnd, 0) as *const vk::c_void };
        let win32_create_info = vk::Win32SurfaceCreateInfoKHR {
            s_type: vk::StructureType::Win32SurfaceCreateInfoKhr,
            p_next: ptr::null(),
            flags: Default::default(),
            hinstance: hinstance,
            hwnd: hwnd as *const vk::c_void,
        };
        let win32_surface_loader =
            Win32Surface::new(entry, instance).expect("Unable to load win32 surface");
        unsafe { win32_surface_loader.create_win32_surface_khr(&win32_create_info, None) }
    }
}

/// Fetch the optimal surface format from the present queue
pub fn select_surface_format(
    present_queue: vk::Queue,
    pdevice: vk::PhysicalDevice,
    surface_loader: &Surface,
    surface: vk::SurfaceKHR,
) -> Result<vk::SurfaceFormatKHR, ()> {
    // Fetch the surface formats
    let surface_formats = surface_loader
        .get_physical_device_surface_formats_khr(pdevice, surface)
        .unwrap();

    let surface_format = surface_formats
        .iter()
        .map(|sfmt| match sfmt.format {
            vk::Format::Undefined => vk::SurfaceFormatKHR {
                format: vk::Format::B8g8r8Unorm,
                color_space: sfmt.color_space,
            },
            _ => sfmt.clone(),
        }).nth(0)
        .expect("Unable to find suitable surface format.");

    Ok(surface_format)
}

pub fn select_desired_image_count(surface_capabilities: &vk::SurfaceCapabilitiesKHR) -> u32 {
    let mut desired_image_count = surface_capabilities.min_image_count + 1;
    if surface_capabilities.max_image_count > 0
        && desired_image_count > surface_capabilities.max_image_count
    {
        desired_image_count = surface_capabilities.max_image_count;
    }

    desired_image_count
}

pub fn select_present_mode(
    surface_loader: &Surface,
    pdevice: vk::PhysicalDevice,
    surface: vk::SurfaceKHR,
) -> Result<vk::PresentModeKHR, ()> {
    // Get the physical devices surface preset
    let present_modes = surface_loader
        .get_physical_device_surface_present_modes_khr(pdevice, surface)
        .unwrap();

    let present_mode = present_modes
        .iter()
        .cloned()
        .find(|&mode| mode == vk::PresentModeKHR::Mailbox)
        .unwrap_or(vk::PresentModeKHR::Fifo);

    Ok(present_mode)
}
