use crate::string_slice::StringSlice;
use ash::{
  Device, Entry, Instance,
  khr::{surface, swapchain},
  vk::{
    self, ApplicationInfo, CompositeAlphaFlagsKHR, DeviceCreateInfo, DeviceQueueCreateInfo,
    DeviceQueueInfo2, Extent2D, Handle, Image, ImageAspectFlags, ImageSubresourceRange,
    ImageUsageFlags, ImageView, ImageViewCreateInfo, ImageViewType, InstanceCreateFlags,
    InstanceCreateInfo, PhysicalDeviceProperties2, PhysicalDeviceType,
    PipelineShaderStageCreateInfo, PipelineVertexInputStateCreateInfo, PresentModeKHR, Queue,
    QueueFamilyProperties2, QueueFlags, ShaderModuleCreateInfo, ShaderStageFlags, SharingMode,
    SurfaceKHR, SwapchainCreateInfoKHR, SwapchainKHR, ValidationFeatureEnableEXT,
    ValidationFeaturesEXT, VertexInputBindingDescription,
  },
};
use sdl2::video::Window;
use std::ffi::{CString, c_void};

pub(super) struct VkEngine {
  _entry: Entry,
  instance: Instance,
  surface: SurfaceKHR,
  surface_instance: surface::Instance,
  device: Device,
  graphics_queue: Queue,
  present_queue: Queue,
  swapchain_device: swapchain::Device,
  swapchain: SwapchainKHR,
  swapchain_images: Vec<Image>,
  swapchain_image_views: Vec<ImageView>,
}

impl VkEngine {
  pub(super) unsafe fn new(window: &Window, prefer_dgpu: bool) -> Self {
    let entry = unsafe { Entry::load().unwrap() };

    let enabled_layers = StringSlice::from(
      #[cfg(debug_assertions)]
      ["VK_LAYER_KHRONOS_validation"].as_slice(),
      #[cfg(not(debug_assertions))]
      [].as_slice(),
    );

    let mut enabled_extensions = window.vulkan_instance_extensions().unwrap();

    #[cfg(debug_assertions)]
    enabled_extensions.extend([
      vk::EXT_DEBUG_UTILS_NAME.to_str().unwrap(),
      vk::EXT_VALIDATION_FEATURES_NAME.to_str().unwrap(),
    ]);

    enabled_extensions.push(vk::KHR_PORTABILITY_ENUMERATION_NAME.to_str().unwrap());

    let enabled_extensions = StringSlice::from(enabled_extensions.as_slice());

    #[cfg(debug_assertions)]
    let enabled_validation_features = [
      ValidationFeatureEnableEXT::BEST_PRACTICES,
      ValidationFeatureEnableEXT::SYNCHRONIZATION_VALIDATION,
    ];

    let app_info = ApplicationInfo {
      api_version: vk::make_api_version(0, 1, 4, 0),
      ..Default::default()
    };

    #[cfg(debug_assertions)]
    let validation_features = ValidationFeaturesEXT {
      enabled_validation_feature_count: enabled_validation_features.len() as _,
      p_enabled_validation_features: enabled_validation_features.as_ptr(),
      ..Default::default()
    };

    let instance_create_info = InstanceCreateInfo {
      flags: InstanceCreateFlags::ENUMERATE_PORTABILITY_KHR,
      p_application_info: &app_info,
      enabled_layer_count: enabled_layers.len() as _,
      pp_enabled_layer_names: enabled_layers.as_ptr(),
      enabled_extension_count: enabled_extensions.len() as _,
      pp_enabled_extension_names: enabled_extensions.as_ptr(),

      #[cfg(debug_assertions)]
      p_next: &validation_features as *const _ as *const c_void,

      ..Default::default()
    };

    let instance = unsafe { entry.create_instance(&instance_create_info, None).unwrap() };

    let surface = SurfaceKHR::from_raw(
      window
        .vulkan_create_surface(instance.handle().as_raw() as _)
        .unwrap(),
    );

    let surface_instance = surface::Instance::new(&entry, &instance);
    let enabled_exts = [vk::KHR_SWAPCHAIN_NAME];

    let (physical_device, graphics_queue_family_index, present_queue_family_index) = unsafe {
      instance
        .enumerate_physical_devices()
        .unwrap()
        .iter()
        .filter_map(|&physical_device| {
          let ext_props = instance
            .enumerate_device_extension_properties(physical_device)
            .unwrap();

          if !enabled_exts.iter().all(|&enabled_ext| {
            ext_props
              .iter()
              .map(|ext_prop| ext_prop.extension_name_as_c_str().unwrap())
              .any(|ext_name| ext_name == enabled_ext)
          }) {
            return None;
          }

          let queue_family_count =
            instance.get_physical_device_queue_family_properties2_len(physical_device);

          let mut queue_family_properties =
            vec![QueueFamilyProperties2::default(); queue_family_count];

          instance.get_physical_device_queue_family_properties2(
            physical_device,
            &mut queue_family_properties,
          );

          if let Some((queue_family_index, _)) = queue_family_properties.iter().enumerate().find(
            |&(queue_family_index, queue_family_properties)| {
              queue_family_properties
                .queue_family_properties
                .queue_flags
                .contains(QueueFlags::GRAPHICS)
                && surface_instance
                  .get_physical_device_surface_support(
                    physical_device,
                    queue_family_index as _,
                    surface,
                  )
                  .unwrap()
            },
          ) {
            return Some((physical_device, queue_family_index, queue_family_index));
          }

          let (graphics_queue_family_index, _) =
            queue_family_properties
              .iter()
              .enumerate()
              .find(|&(_, queue_family_properties)| {
                queue_family_properties
                  .queue_family_properties
                  .queue_flags
                  .contains(QueueFlags::GRAPHICS)
              })?;

          let (present_queue_family_index, _) =
            queue_family_properties
              .iter()
              .enumerate()
              .find(|&(queue_family_index, _)| {
                surface_instance
                  .get_physical_device_surface_support(
                    physical_device,
                    queue_family_index as _,
                    surface,
                  )
                  .unwrap()
              })?;

          Some((
            physical_device,
            graphics_queue_family_index,
            present_queue_family_index,
          ))
        })
        .max_by_key(|&(physical_device, _, _)| {
          let mut physical_device_properties = PhysicalDeviceProperties2::default();

          instance
            .get_physical_device_properties2(physical_device, &mut physical_device_properties);

          match physical_device_properties.properties.device_type {
            PhysicalDeviceType::INTEGRATED_GPU => {
              if prefer_dgpu {
                3
              } else {
                4
              }
            }
            PhysicalDeviceType::DISCRETE_GPU => {
              if prefer_dgpu {
                4
              } else {
                3
              }
            }
            PhysicalDeviceType::VIRTUAL_GPU => 2,
            PhysicalDeviceType::CPU => 1,
            _ => 0,
          }
        })
        .unwrap()
    };

    let queue_priorities = [1.0];

    let mut queue_create_infos = vec![DeviceQueueCreateInfo {
      queue_family_index: graphics_queue_family_index as _,
      queue_count: 1,
      p_queue_priorities: queue_priorities.as_ptr(),
      ..Default::default()
    }];

    if graphics_queue_family_index != present_queue_family_index {
      queue_create_infos.push(DeviceQueueCreateInfo {
        queue_family_index: present_queue_family_index as _,
        queue_count: 1,
        p_queue_priorities: queue_priorities.as_ptr(),
        ..Default::default()
      });
    }

    let enabled_exts = StringSlice::from(
      enabled_exts
        .map(|enabled_ext| enabled_ext.to_str().unwrap())
        .as_slice(),
    );

    let device_create_info = DeviceCreateInfo {
      queue_create_info_count: queue_create_infos.len() as _,
      p_queue_create_infos: queue_create_infos.as_ptr(),
      enabled_extension_count: enabled_exts.len() as _,
      pp_enabled_extension_names: enabled_exts.as_ptr(),
      ..Default::default()
    };

    let device = unsafe {
      instance
        .create_device(physical_device, &device_create_info, None)
        .unwrap()
    };

    let graphics_queue_info = DeviceQueueInfo2 {
      queue_family_index: graphics_queue_family_index as _,
      queue_index: 0,
      ..Default::default()
    };

    let present_queue_info = DeviceQueueInfo2 {
      queue_family_index: present_queue_family_index as _,
      queue_index: 0,
      ..Default::default()
    };

    let graphics_queue = unsafe { device.get_device_queue2(&graphics_queue_info) };
    let present_queue = unsafe { device.get_device_queue2(&present_queue_info) };

    let surface_capabilities = unsafe {
      surface_instance
        .get_physical_device_surface_capabilities(physical_device, surface)
        .unwrap()
    };

    let min_image_count = surface_capabilities.min_image_count + 1;

    let min_image_count = if surface_capabilities.max_image_count > 0 {
      min_image_count.min(surface_capabilities.max_image_count)
    } else {
      min_image_count
    };

    let surface_format = unsafe {
      surface_instance
        .get_physical_device_surface_formats(physical_device, surface)
        .unwrap()[0]
    };

    let surface_extent = if surface_capabilities.current_extent.width != u32::MAX {
      surface_capabilities.current_extent
    } else {
      let (drawable_width, drawable_height) = window.vulkan_drawable_size();

      Extent2D {
        width: drawable_width.clamp(
          surface_capabilities.min_image_extent.width,
          surface_capabilities.max_image_extent.width,
        ),
        height: drawable_height.clamp(
          surface_capabilities.min_image_extent.height,
          surface_capabilities.max_image_extent.height,
        ),
      }
    };

    let surface_sharing_mode = if graphics_queue_family_index == present_queue_family_index {
      SharingMode::EXCLUSIVE
    } else {
      SharingMode::CONCURRENT
    };

    let surface_queue_family_indices = if surface_sharing_mode == SharingMode::EXCLUSIVE {
      vec![]
    } else {
      vec![
        graphics_queue_family_index as _,
        present_queue_family_index as _,
      ]
    };

    let swapchain_device = swapchain::Device::new(&instance, &device);

    let swapchain_create_info = SwapchainCreateInfoKHR {
      surface,
      min_image_count,
      image_format: surface_format.format,
      image_color_space: surface_format.color_space,
      image_extent: surface_extent,
      image_array_layers: 1,
      image_usage: ImageUsageFlags::COLOR_ATTACHMENT,
      image_sharing_mode: surface_sharing_mode,
      queue_family_index_count: surface_queue_family_indices.len() as _,
      p_queue_family_indices: surface_queue_family_indices.as_ptr(),
      pre_transform: surface_capabilities.current_transform,
      composite_alpha: CompositeAlphaFlagsKHR::OPAQUE,
      present_mode: PresentModeKHR::FIFO,
      clipped: vk::TRUE,
      ..Default::default()
    };

    let swapchain = unsafe {
      swapchain_device
        .create_swapchain(&swapchain_create_info, None)
        .unwrap()
    };

    let swapchain_images = unsafe { swapchain_device.get_swapchain_images(swapchain).unwrap() };

    let swapchain_image_views = unsafe {
      swapchain_images.iter().map(|&swapchain_image| {
        let image_view_create_info = ImageViewCreateInfo {
          image: swapchain_image,
          view_type: ImageViewType::TYPE_2D,
          format: surface_format.format,
          subresource_range: ImageSubresourceRange {
            aspect_mask: ImageAspectFlags::COLOR,
            base_mip_level: 0,
            level_count: 1,
            base_array_layer: 0,
            layer_count: 1,
          },
          ..Default::default()
        };

        device
          .create_image_view(&image_view_create_info, None)
          .unwrap()
      })
    }
    .collect();

    const VERT_SHADER_CODE: &[u8] = include_bytes!("../target/shaders/basic.vert.spv");
    const FRAG_SHADER_CODE: &[u8] = include_bytes!("../target/shaders/basic.frag.spv");

    let vert_shader_create_info = ShaderModuleCreateInfo {
      code_size: VERT_SHADER_CODE.len(),
      p_code: VERT_SHADER_CODE.as_ptr() as *const _,
      ..Default::default()
    };

    let frag_shader_create_info = ShaderModuleCreateInfo {
      code_size: FRAG_SHADER_CODE.len(),
      p_code: FRAG_SHADER_CODE.as_ptr() as *const _,
      ..Default::default()
    };

    let vert_shader = unsafe {
      device
        .create_shader_module(&vert_shader_create_info, None)
        .unwrap()
    };

    let frag_shader = unsafe {
      device
        .create_shader_module(&frag_shader_create_info, None)
        .unwrap()
    };

    let shader_entry_point_name = CString::new("main").unwrap();

    let vert_shader_stage_create_info = PipelineShaderStageCreateInfo {
      stage: ShaderStageFlags::VERTEX,
      module: vert_shader,
      p_name: shader_entry_point_name.as_ptr(),
      ..Default::default()
    };

    let frag_shader_stage_create_info = PipelineShaderStageCreateInfo {
      stage: ShaderStageFlags::FRAGMENT,
      module: frag_shader,
      p_name: shader_entry_point_name.as_ptr(),
      ..Default::default()
    };

    // let vert_binding_desc = VertexInputBindingDescription {
    //   binding: 0,
    //   stride: todo!(),
    //   input_rate: todo!(),
    // };

    // let vert_input_stage_create_info = PipelineVertexInputStateCreateInfo {
    //   vertex_binding_description_count: todo!(),
    //   p_vertex_binding_descriptions: todo!(),
    //   vertex_attribute_description_count: todo!(),
    //   p_vertex_attribute_descriptions: todo!(),
    //   ..Default::default()
    // };

    unsafe {
      device.destroy_shader_module(frag_shader, None);
      device.destroy_shader_module(vert_shader, None);
    };

    Self {
      _entry: entry,
      instance,
      surface,
      surface_instance,
      device,
      graphics_queue,
      present_queue,
      swapchain_device,
      swapchain,
      swapchain_images,
      swapchain_image_views,
    }
  }
}

impl Drop for VkEngine {
  fn drop(&mut self) {
    unsafe {
      self.swapchain_image_views.iter().for_each(|&image_view| {
        self.device.destroy_image_view(image_view, None);
      });

      self
        .swapchain_device
        .destroy_swapchain(self.swapchain, None);

      self.device.destroy_device(None);
      self.surface_instance.destroy_surface(self.surface, None);
      self.instance.destroy_instance(None);
    }
  }
}
