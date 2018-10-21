use ash::version::{DeviceV1_0, EntryV1_0, InstanceV1_0, V1_0};
use ash::vk;
use ash::{Device, Entry, Instance};
use std::cell::RefCell;
use std::ffi::CStr;
use std::ffi::CString;
use std::ptr;
use std::{u32, u64};
use winit::{ControlFlow, Event, WindowEvent};
use winit::{EventsLoop, Window, WindowBuilder};

use shader::Shader;

use engine::device::Device as EngineDevice;
use engine::{command_pool, fence, image, instance, pipeline, semaphore, surface, swapchain};

#[cfg(target_os = "windows")]
use ash::extensions::Win32Surface;

#[cfg(target_os = "macos")]
use ash::extensions::MacOSSurface;

use ash::extensions::{DebugReport, Surface, Swapchain};

pub struct Application {
    window: Window,
    events_loop: RefCell<EventsLoop>,
    entry: Entry<V1_0>,
    instance: Instance<V1_0>,
    validation_layers: Vec<CString>,
    debug_callback: vk::DebugReportCallbackEXT,
    debug_report_loader: DebugReport,
    device: Device<V1_0>,
    surface_loader: Surface,
    surface: vk::SurfaceKHR,
    swapchain: vk::SwapchainKHR,
    swapchain_loader: Swapchain,
    image_views: Vec<vk::ImageView>,
    graphics_pipelines: pipeline::Pipeline,
    swapchain_buffers: Vec<vk::Framebuffer>,
    command_pool: command_pool::CommandPool,
    //command_buffers: Vec<vk::CommandBuffer>,
    image_available_semaphore: Vec<semaphore::Semaphore>,
    render_finished_semaphore: Vec<semaphore::Semaphore>,
    present_queue: vk::Queue,
    surface_resolution: vk::Extent2D,
    in_flight_fences: Vec<fence::Fence>,
}

impl Application {
    pub fn new() -> Self {
        Application::init_vulkan()
    }

    pub fn run(&mut self) -> Result<(), ()> {
        self.main_loop();

        Ok(())
    }

    fn init_vulkan() -> Self {
        Application::create_instance()
    }

    fn create_instance() -> Self {
        // Create the window and get the event_loop
        let (window, events_loop) = Application::init_window();

        // Width and height of the window
        let (width, height) = window.get_inner_size().unwrap().into();

        // Create new entry
        let entry = Entry::new().unwrap();

        // Set of validation layers to request and load
        let validation_layers = vec![CString::new("VK_LAYER_LUNARG_standard_validation").unwrap()];

        // Check that we have access to the validation layers
        if !Application::check_validation_layer_support(&entry, &validation_layers) {
            panic!("Requested validation layer not available!");
        }

        // Names
        let app_name = CString::new("test").unwrap();
        let engine_name = CString::new("test").unwrap();

        // Translate valiation layers to a value that can be passed
        // to ash
        let layer_names_raw: Vec<*const i8> = validation_layers
            .iter()
            .map(|raw_name| raw_name.as_ptr())
            .collect();

        // Get requested extension names
        let extension_names_raw = extension_names();

        // Create a vulkan instance
        let instance = instance::create_instance(
            &entry,
            app_name,
            engine_name,
            &layer_names_raw,
            &extension_names_raw,
        ).unwrap();

        // Load the debug report and its callback
        let (debug_report_loader, debug_callback) =
            Application::setup_debug_callback(&entry, &instance);

        // Load extensions from the sdk
        let extensions = entry
            .enumerate_instance_extension_properties()
            .expect("unable to extract extensions");

        // Print extensions
        for extension in extensions {
            let name = CString::new(
                extension
                    .extension_name
                    .iter()
                    .map(|char| *char as u8)
                    .filter(|char| *char != 0 as u8)
                    .collect::<Vec<_>>(),
            ).unwrap();

            println!("Extension: {:?}", name);
        }

        // Create surface
        let surface = surface::create_surface(&entry, &instance, &window).unwrap();
        let surface_loader =
            Surface::new(&entry, &instance).expect("Unable to load the Surface extension");

        // Pick our suitable physical device, and its queue index
        let (pdevice, queue_index) =
            Application::pick_physical_device(&instance, surface, &surface_loader);

        // Create device, we will use this to interact with the physical device
        //
        // This connects the physical device to the instance, connecting all the
        // validation layers and extensions.
        let device = {
            let device = EngineDevice::new(
                &instance,
                layer_names_raw,
                extension_names_raw,
                queue_index as u32,
                pdevice,
            ).unwrap();
            device.device
        };

        // Create the presentation queue from the device, with the queue index
        let present_queue = unsafe { device.get_device_queue(queue_index as u32, 0) };

        let surface_format =
            surface::select_surface_format(present_queue, pdevice, &surface_loader, surface)
                .unwrap();

        // Load the capabilities of the surface we selected
        let surface_capabilities = surface_loader
            .get_physical_device_surface_capabilities_khr(pdevice, surface)
            .unwrap();

        // Select the desired image count
        let desired_image_count = surface::select_desired_image_count(&surface_capabilities);

        // Get the surface extent
        let surface_resolution = match surface_capabilities.current_extent.width {
            u32::MAX => vk::Extent2D { width, height },
            _ => surface_capabilities.current_extent,
        };

        // Set up the surface transform identity
        let pre_transform = if surface_capabilities
            .supported_transforms
            .subset(vk::SURFACE_TRANSFORM_IDENTITY_BIT_KHR)
        {
            vk::SURFACE_TRANSFORM_IDENTITY_BIT_KHR
        } else {
            surface_capabilities.current_transform
        };

        // Fetch the proper present mode
        let present_mode = surface::select_present_mode(&surface_loader, pdevice, surface).unwrap();

        // Fetch swapchain extension
        let swapchain_loader =
            Swapchain::new(&instance, &device).expect("Unable to load swapchain");

        let swapchain = swapchain::create_swapchain(
            &swapchain_loader,
            surface,
            desired_image_count,
            &surface_format,
            surface_resolution,
            pre_transform,
            present_mode,
        ).unwrap();

        let images = swapchain_loader
            .get_swapchain_images_khr(swapchain)
            .unwrap();

        let image_views = image::create_image_views(&device, images, &surface_format).unwrap();

        let vert_shader = Shader::load(
            &device,
            "assets/shaders/vert.spv",
            pipeline::ShaderType::Vertex,
        );
        let frag_shader = Shader::load(
            &device,
            "assets/shaders/frag.spv",
            pipeline::ShaderType::Fragment,
        );

        let pipeline_layout = pipeline::PipelineLayout::empty(&device);
        let render_pass = pipeline::RenderPass::new(&device, surface_format);

        let graphics_pipelines = pipeline::Pipeline::build()
            .with_shader_stage(vert_shader)
            .with_shader_stage(frag_shader)
            .with_vertex_input_state()
            .with_input_assembly_state()
            .with_viewport(surface_resolution)
            .with_rasterizer()
            .with_multisample()
            .with_color_blend()
            .with_layout(pipeline_layout)
            .with_render_pass(render_pass)
            .create(&device)
            .unwrap();

        let swapchain_buffers = image_views
            .iter()
            .map(|&image_view| {
                let framebuffer_info = vk::FramebufferCreateInfo {
                    s_type: vk::StructureType::FramebufferCreateInfo,
                    p_next: ptr::null(),
                    flags: Default::default(),
                    render_pass: graphics_pipelines.render_pass.render_pass,
                    attachment_count: 1,
                    p_attachments: [image_view].as_ptr(),
                    width: surface_resolution.width,
                    height: surface_resolution.height,
                    layers: 1,
                };

                unsafe {
                    device
                        .create_framebuffer(&framebuffer_info, None)
                        .expect("Unable to create framebuffer")
                }
            }).collect::<Vec<_>>();

        let command_pool = command_pool::CommandPool::new(
            &device,
            swapchain_buffers.len() as u32,
            queue_index as u32,
        );

        let image_available_semaphore = [0; 2]
            .iter()
            .map(|_| semaphore::Semaphore::new(&device))
            .collect::<Vec<_>>();
        let render_finished_semaphore = [0; 2]
            .iter()
            .map(|_| semaphore::Semaphore::new(&device))
            .collect::<Vec<_>>();

        let in_flight_fences = [0; 2]
            .iter()
            .map(|_| fence::Fence::new(&device))
            .collect::<Vec<_>>();

        command_pool
            .setup_command_buffers(
                &device,
                &swapchain_buffers,
                &graphics_pipelines,
                surface_resolution,
            ).unwrap();

        Application {
            window,
            events_loop,
            entry,
            debug_callback,
            debug_report_loader,
            validation_layers,
            instance,
            device,
            surface_loader,
            surface,
            swapchain,
            swapchain_loader,
            surface_resolution,
            image_views,
            graphics_pipelines,
            swapchain_buffers,
            command_pool,
            //command_buffers,
            image_available_semaphore,
            render_finished_semaphore,
            present_queue,
            in_flight_fences,
        }
    }

    fn draw_frame(&self, current_frame: u8) {
        let image_available_semaphore = self
            .image_available_semaphore
            .get(current_frame as usize)
            .unwrap();

        let render_finished_semaphore = self
            .render_finished_semaphore
            .get(current_frame as usize)
            .unwrap();

        let in_flight_fence = self.in_flight_fences.get(current_frame as usize).unwrap();

        unsafe {
            self.device
                .wait_for_fences(&[in_flight_fence.fence], false, 0)
                .unwrap();
            self.device.reset_fences(&[in_flight_fence.fence]).unwrap();
        };

        let image_index = unsafe {
            self.swapchain_loader
                .acquire_next_image_khr(
                    self.swapchain,
                    u64::MAX,
                    image_available_semaphore.semaphore,
                    vk::Fence::null(),
                ).expect("Unable to acquire next image")
        };

        let submit_info = vk::SubmitInfo {
            s_type: vk::StructureType::SubmitInfo,
            p_next: ptr::null(),
            p_wait_semaphores: [image_available_semaphore.semaphore].as_ptr(),
            wait_semaphore_count: 1,
            p_wait_dst_stage_mask: [vk::PIPELINE_STAGE_COLOR_ATTACHMENT_OUTPUT_BIT].as_ptr(),
            command_buffer_count: 1,
            p_command_buffers: self
                .command_pool
                .command_buffers
                .get(image_index as usize)
                .unwrap(),
            signal_semaphore_count: 1,
            p_signal_semaphores: [render_finished_semaphore.semaphore].as_ptr(),
        };

        unsafe {
            self.device
                .queue_submit(self.present_queue, &[submit_info], in_flight_fence.fence)
                .expect("Unable to submit queue");
        };

        let present_info = vk::PresentInfoKHR {
            s_type: vk::StructureType::PresentInfoKhr,
            p_next: ptr::null(),
            wait_semaphore_count: 1,
            p_wait_semaphores: [render_finished_semaphore.semaphore].as_ptr(),
            swapchain_count: 1,
            p_swapchains: [self.swapchain].as_ptr(),
            p_image_indices: [image_index].as_ptr(),
            p_results: ptr::null_mut(),
        };

        unsafe {
            self.swapchain_loader
                .queue_present_khr(self.present_queue, &present_info)
                .expect("Unable to queue present");
        };
    }

    fn pick_physical_device(
        instance: &Instance<V1_0>,
        surface: vk::SurfaceKHR,
        surface_loader: &Surface,
    ) -> (vk::PhysicalDevice, usize) {
        // TODO: pick a suitable device via score
        let pdevices = instance
            .enumerate_physical_devices()
            .expect("Unable to enumerate physical devices");

        pdevices
            .iter()
            .map(|pdevice| {
                instance
                    .get_physical_device_queue_family_properties(*pdevice)
                    .iter()
                    .enumerate()
                    .filter_map(|(index, ref info)| {
                        let supports_graphic_and_surface =
                            info.queue_flags.subset(vk::QUEUE_GRAPHICS_BIT) && surface_loader
                                .get_physical_device_surface_support_khr(
                                    *pdevice,
                                    index as u32,
                                    surface,
                                );
                        match supports_graphic_and_surface {
                            true => Some((*pdevice, index)),
                            _ => None,
                        }
                    }).nth(0)
            }).filter_map(|v| v)
            .nth(0)
            .expect("Couldn't find suitable device.")
    }

    fn check_validation_layer_support(
        entry: &Entry<V1_0>,
        validation_layers: &Vec<CString>,
    ) -> bool {
        let layers = entry
            .enumerate_instance_layer_properties()
            .expect("unable to fetch layer properties");

        let mut layer_found = false;
        for layer in layers {
            let name = CString::new(
                layer
                    .layer_name
                    .iter()
                    .map(|char| *char as u8)
                    .filter(|char| *char != 0 as u8)
                    .collect::<Vec<_>>(),
            ).unwrap();

            println!("Layer: {:?}", name);

            for validation_layer in validation_layers.iter() {
                if validation_layer == &name {
                    println!("Validation: Found matching validation layer: {:?}", name);
                    layer_found = true
                }
            }
        }

        layer_found
    }

    fn setup_debug_callback(
        entry: &Entry<V1_0>,
        instance: &Instance<V1_0>,
    ) -> (DebugReport, vk::DebugReportCallbackEXT) {
        let debug_info = vk::DebugReportCallbackCreateInfoEXT {
            s_type: vk::StructureType::DebugReportCallbackCreateInfoExt,
            p_next: ptr::null(),
            flags: vk::DEBUG_REPORT_ERROR_BIT_EXT
                | vk::DEBUG_REPORT_WARNING_BIT_EXT
                | vk::DEBUG_REPORT_PERFORMANCE_WARNING_BIT_EXT,
            pfn_callback: vulkan_debug_callback,
            p_user_data: ptr::null_mut(),
        };

        let debug_report_loader =
            DebugReport::new(entry, instance).expect("Unable to load debug report");

        let debug_callback;
        unsafe {
            debug_callback = debug_report_loader
                .create_debug_report_callback_ext(&debug_info, None)
                .unwrap();
        }

        (debug_report_loader, debug_callback)
    }

    fn main_loop(&mut self) {
        let mut current_frame = 0;

        self.events_loop.borrow_mut().run_forever(|event| {
            // println!("{:?}", event);

            match event {
                Event::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    ..
                } => ControlFlow::Break,
                _ => {
                    current_frame = (current_frame + 1) % 2;
                    self.draw_frame(current_frame);
                    self.device.device_wait_idle().unwrap();

                    ControlFlow::Continue
                }
            }
        });
    }

    fn init_window() -> (Window, RefCell<EventsLoop>) {
        let events_loop = RefCell::new(EventsLoop::new());

        let window = WindowBuilder::new()
            .with_title("Ash test")
            .build(&*events_loop.borrow())
            .unwrap();

        (window, events_loop)
    }
}

impl Drop for Application {
    fn drop(&mut self) {
        unsafe {
            self.in_flight_fences
                .iter()
                .for_each(|ref fence| self.device.destroy_fence(fence.fence, None));

            self.render_finished_semaphore
                .iter()
                .for_each(|ref semaphore| {
                    self.device.destroy_semaphore(semaphore.semaphore, None);
                });

            self.image_available_semaphore
                .iter()
                .for_each(|ref semaphore| {
                    self.device.destroy_semaphore(semaphore.semaphore, None);
                });

            self.device
                .destroy_command_pool(self.command_pool.command_pool, None);

            self.swapchain_buffers.iter().for_each(|&buffer| {
                self.device.destroy_framebuffer(buffer, None);
            });

            self.graphics_pipelines
                .graphics_pipelines
                .iter()
                .for_each(|&pipeline| {
                    self.device.destroy_pipeline(pipeline, None);
                });

            let render_pass = self.graphics_pipelines.render_pass.render_pass;
            self.device.destroy_render_pass(render_pass, None);

            let layout = self.graphics_pipelines.layout.layout;
            self.device.destroy_pipeline_layout(layout, None);

            self.graphics_pipelines
                .shaders
                .iter()
                .for_each(|ref shader_module| {
                    self.device
                        .destroy_shader_module(shader_module.module, None);
                });

            self.image_views.iter().for_each(|&image_view| {
                self.device.destroy_image_view(image_view, None);
            });

            self.swapchain_loader
                .destroy_swapchain_khr(self.swapchain, None);

            self.surface_loader.destroy_surface_khr(self.surface, None);

            self.device.destroy_device(None);

            self.debug_report_loader
                .destroy_debug_report_callback_ext(self.debug_callback, None);

            self.instance.destroy_instance(None);
        };
    }
}

unsafe extern "system" fn vulkan_debug_callback(
    _: vk::DebugReportFlagsEXT,
    _: vk::DebugReportObjectTypeEXT,
    _: vk::uint64_t,
    _: vk::size_t,
    _: vk::int32_t,
    _: *const vk::c_char,
    p_message: *const vk::c_char,
    _: *mut vk::c_void,
) -> u32 {
    println!("{:?}", CStr::from_ptr(p_message));
    vk::VK_FALSE
}

#[cfg(all(unix, not(target_os = "android"), not(target_os = "macos")))]
fn extension_names() -> Vec<*const i8> {
    vec![
        Surface::name().as_ptr(),
        XlibSurface::name().as_ptr(),
        DebugReport::name().as_ptr(),
    ]
}

#[cfg(target_os = "macos")]
fn extension_names() -> Vec<*const i8> {
    vec![
        Surface::name().as_ptr(),
        MacOSSurface::name().as_ptr(),
        DebugReport::name().as_ptr(),
    ]
}

#[cfg(all(windows))]
fn extension_names() -> Vec<*const i8> {
    vec![
        Surface::name().as_ptr(),
        Win32Surface::name().as_ptr(),
        DebugReport::name().as_ptr(),
    ]
}
