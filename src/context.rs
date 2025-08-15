use std::sync::Arc;

use vulkano::{
  device::{
    physical::PhysicalDeviceType, Device, DeviceCreateInfo, DeviceExtensions, Queue, QueueCreateInfo, QueueFlags
  },
  instance::{
    Instance, InstanceCreateFlags, InstanceCreateInfo
  },
  swapchain::Surface,
  VulkanLibrary
};

use winit::{ event_loop::ActiveEventLoop, window::Window };

use crate::renderer::VulkanRenderer;

#[derive(Default)]
pub struct VulkanRenderContext{
  pub queue: Option<Arc<Queue>>
}

impl VulkanRenderContext{
  pub fn renderer_for_window( &mut self, event_loop: &ActiveEventLoop, window: Arc<Window> ) -> VulkanRenderer{
    let queue = self.queue.get_or_insert_with(|| Self::shared_queue(event_loop, window.clone()));
    VulkanRenderer::new(window.clone(), queue.clone())
  }

  fn shared_queue( event_loop: &ActiveEventLoop, window: Arc<Window> ) -> Arc<Queue>{
    let library = VulkanLibrary::new().expect("Vulkan libraries not found");
    let required_extensions = Surface::required_extensions(event_loop).unwrap();

    let instance = Instance::new(library, InstanceCreateInfo {
      flags: InstanceCreateFlags::ENUMERATE_PORTABILITY,
      enabled_extensions: required_extensions,
      ..Default::default()
    }).unwrap_or_else(|_| { panic!("Could not create instance supporting: {required_extensions:?}") });

    let device_extensions = DeviceExtensions { khr_swapchain: true, ..DeviceExtensions::empty() };
    let surface = Surface::from_window(instance.clone(), window.clone()).unwrap();

    let ( physical_device, queue_family_index ) = instance
      .enumerate_physical_devices()
      .unwrap()
      .filter(|p| { p.supported_extensions().contains(&device_extensions) })
      .filter_map(|p| {
        p.queue_family_properties()
          .iter()
          .enumerate()
          .position(|(i, q)| {
            q.queue_flags.intersects(QueueFlags::GRAPHICS) && p.surface_support(i as u32, &surface).unwrap_or(false)
          })
          .map(|i| ( p, i as u32 ))
      })
      .min_by_key(|(p, _)| {
        match p.properties().device_type {
          PhysicalDeviceType::DiscreteGpu => 0,
          PhysicalDeviceType::IntegratedGpu => 1,
          PhysicalDeviceType::VirtualGpu => 2,
          PhysicalDeviceType::Cpu => 3,
          PhysicalDeviceType::Other => 4,
          _ => 5
        }
      })
      .expect("No suitable physical device found");

    println!("Using: {} (type: {:?})", physical_device.properties().device_name, physical_device.properties().device_type,);

    let ( _, mut queues ) = Device::new(physical_device, DeviceCreateInfo {
      enabled_extensions: device_extensions,
      queue_create_infos: vec![QueueCreateInfo {
        queue_family_index,
        ..Default::default()
      }],
      ..Default::default()
    }).expect("Device initialisation failed");

    queues.next().unwrap()
  }
}