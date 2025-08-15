use std::{ ptr, sync::Arc };

use skia_safe::{gpu::{self, backend_render_targets, direct_contexts, surfaces, vk}, ColorType};
use vulkano::{device::Queue, format::Format, image::{view::ImageView, ImageUsage}, render_pass::{Framebuffer, FramebufferCreateInfo, RenderPass}, swapchain::{acquire_next_image, PresentMode, Surface, Swapchain, SwapchainAcquireFuture, SwapchainCreateInfo, SwapchainPresentInfo}, sync::{self, GpuFuture}, Handle, Validated, VulkanError, VulkanObject};
use winit::{dpi::{LogicalSize, PhysicalSize}, window::Window};

pub struct VulkanRenderer{
  pub window: Arc<Window>,
  queue: Arc<Queue>,
  swapchain: Arc<Swapchain>,
  framebuffers: Vec<Arc<Framebuffer>>,
  render_pass: Arc<RenderPass>,
  last_render: Option<Box<dyn GpuFuture>>,
  skia_ctx: gpu::DirectContext,
  swapchain_is_valid: bool
}

impl Drop for VulkanRenderer{
  fn drop(&mut self) {
      self.skia_ctx.abandon();
  }
}

impl VulkanRenderer{
  pub fn new( window: Arc<Window>, queue: Arc<Queue> ) -> Self{
    let library = queue.device().instance().library();
    let instance = queue.device().instance();
    let device = queue.device();
    let queue = queue.clone();

    let surface = Surface::from_window(instance.clone(), window.clone()).unwrap();
    let window_size = window.inner_size();

    let ( swapchain, _images ) = {
      let surface_capabilities = device
        .physical_device()
        .surface_capabilities(&surface, Default::default())
        .unwrap();

      let ( image_format, _ ) = device
        .physical_device()
        .surface_formats(&surface, Default::default())
        .unwrap()[0];

      Swapchain::new(
        device.clone(),
        surface,
        SwapchainCreateInfo {
          min_image_count: surface_capabilities.min_image_count.max(2),
          image_extent: window_size.into(),
          image_usage: ImageUsage::COLOR_ATTACHMENT,
          image_format,
          present_mode: PresentMode::Fifo,
          composite_alpha: surface_capabilities
            .supported_composite_alpha
            .into_iter()
            .next().unwrap(),
          ..Default::default()
        }
      ).unwrap()
    };

    let render_pass = vulkano::single_pass_renderpass!(
      device.clone(),
      attachments: {
        color: {
          format: swapchain.image_format(),
          samples: 1,
          load_op: DontCare,
          store_op: Store
        }
      },
      pass: {
        color: [color],
        depth_stencil: {}
      }
    ).unwrap();

    let framebuffers = vec![];
    let swapchain_is_valid = false;
    let last_render = Some(sync::now(device.clone()).boxed());
    let skia_ctx = unsafe {
      let get_proc = |gpo| {
        let get_device_proc_addr = instance.fns().v1_0.get_device_proc_addr;

        match gpo{
          vk::GetProcOf::Instance(instance, name) => {
            let vk_instance = ash::vk::Instance::from_raw(instance as _);
            library.get_instance_proc_addr(vk_instance, name)
          }
          vk::GetProcOf::Device(device, name) => {
            let vk_device = ash::vk::Device::from_raw(device as _);
            get_device_proc_addr(vk_device, name)
          }
        }
        .map(|f| f as _)
        .unwrap_or_else(|| {
          println!("Vulkan: failed to resolve {}", gpo.name().to_str().unwrap());
          ptr::null()
        })
      };

      let direct_context = direct_contexts::make_vulkan(
        &vk::BackendContext::new(
          instance.handle().as_raw() as _,
          device.physical_device().handle().as_raw() as _,
          device.handle().as_raw() as _,
          (
            queue.handle().as_raw() as _,
            queue.queue_family_index() as usize
          ),
          &get_proc
        ),
        None
      ).unwrap();

      direct_context
    };

    VulkanRenderer { window, queue, swapchain, framebuffers, render_pass, last_render, skia_ctx, swapchain_is_valid }
  }

  pub fn invalidate_swapchain( &mut self ){
    self.swapchain_is_valid = false;
  }

  pub fn prepare_swapchain(&mut self){
    if let Some(last_render) = self.last_render.as_mut(){
      last_render.cleanup_finished();
    }

    let window_size: PhysicalSize<u32> = self.window.inner_size();
    if window_size.width > 0 && window_size.height > 0 && !self.swapchain_is_valid{
      let ( new_swapchain, new_images ) = self.swapchain.recreate(SwapchainCreateInfo {
        image_extent: window_size.into(),
        ..self.swapchain.create_info()
      }).expect("Failed to recreate swapchain");

      self.swapchain = new_swapchain;
      self.framebuffers = new_images.iter()
        .map(|image| {
          let view = ImageView::new_default(image.clone()).unwrap();

          Framebuffer::new(
            self.render_pass.clone(),
            FramebufferCreateInfo{
              attachments: vec![view],
              ..Default::default()
            }
          ).unwrap()
        })
        .collect::<Vec<_>>();

      self.swapchain_is_valid = true;
    }
  }

  fn get_next_frame(&mut self) -> Option<(u32, SwapchainAcquireFuture)>{
    let ( image_index, suboptimal, aquire_future ) =
      match acquire_next_image(self.swapchain.clone(), None).map_err(Validated::unwrap) {
        Ok(r) => r,
        Err(VulkanError::OutOfDate) => {
          self.swapchain_is_valid = false;
          return None
        },
        Err(e) => panic!("Failed to aquire next image: {e}")
      };

    if suboptimal { self.swapchain_is_valid = false; }

    if self.swapchain_is_valid {
      Some(( image_index, aquire_future ))
    } else{
      None
    }
  }

  pub fn draw_and_present<F>(&mut self, f: F)
  where
    F: FnOnce(&skia_safe::Canvas, LogicalSize<f32>),
  {
    let next_frame = self.get_next_frame().or_else(|| {
      self.prepare_swapchain();
      self.get_next_frame()
    });

    if let Some(( image_index, aquire_future )) = next_frame {
      let framebuffer = self.framebuffers[image_index as usize].clone();
      let mut surface = surface_for_framebuffer(&mut self.skia_ctx, framebuffer.clone());
      let canvas = surface.canvas();

      let extent: PhysicalSize<u32> = self.window.inner_size();
      let size: LogicalSize<f32> = extent.to_logical(self.window.scale_factor());

      let scale = (
        (f64::from(extent.width) / size.width as f64) as f32,
        (f64::from(extent.height) / size.height as f64) as f32
      );

      canvas.reset_matrix();
      canvas.scale(scale);

      f(canvas, size);

      self.skia_ctx.flush_and_submit();
      self.last_render = self.last_render
        .take()
        .unwrap()
        .join(aquire_future)
        .then_swapchain_present(
          self.queue.clone(),
          SwapchainPresentInfo::swapchain_image_index(
            self.swapchain.clone(),
            image_index
          )
        )
        .then_signal_fence_and_flush()
        .map(|f| Box::new(f) as _)
        .ok();
    }
  }
}

fn surface_for_framebuffer( skia_ctx: &mut gpu::DirectContext, framebuffer: Arc<Framebuffer> ) -> skia_safe::Surface{
  let [ width, height ] = framebuffer.extent();
  let image_access = &framebuffer.attachments()[0];
  let image_object = image_access.image().handle().as_raw();

  let format = image_access.format();

  let ( vk_format, color_type ) = match format{
    Format::B8G8R8A8_UNORM => (
      vk::Format::B8G8R8A8_UNORM,
      ColorType::BGRA8888
    ),
    Format::A2R10G10B10_UNORM_PACK32 => (
      vk::Format::A2R10G10B10_UNORM_PACK32,
      ColorType::BGRA1010102
    ),
    _ => panic!("Unsupported colour format: {format:?}")
  };

  let alloc = vk::Alloc::default();
  let image_info = &unsafe {
    vk::ImageInfo::new(
      image_object as _,
      alloc,
      vk::ImageTiling::OPTIMAL,
      vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
      vk_format,
      1,
      None,
      None,
      None,
      None
    )
  };

  let render_target = &backend_render_targets::make_vk(
    ( width.try_into().unwrap(), height.try_into().unwrap() ),
    image_info
  );

  surfaces::wrap_backend_render_target(
    skia_ctx,
    render_target,
    gpu::SurfaceOrigin::TopLeft,
    color_type,
    None,
    None
  ).expect("Invalid Render Target")
}