use std::sync::Arc;

use winit::{ dpi::LogicalSize, event_loop::ActiveEventLoop, window::Window };

use crate::renderer::vulkan::VulkanBackend;

pub mod vulkan;

pub enum RendererBackend{
  Vulkan(VulkanBackend)
}

pub struct Renderer{
  backend: RendererBackend
}

impl Renderer{
  pub fn new( renderer_type: u8 ) -> Renderer{
    // 0 - Vulkan

    match renderer_type{
      0 => {
        Renderer {
          backend: RendererBackend::Vulkan(VulkanBackend::default())
        }
      }
      _ => panic!("Rendering Backend: #{renderer_type:?} is not supported.")
    }
  }

  pub fn resumed( &mut self, window: Arc<Window>, event_loop: &ActiveEventLoop ){
    match &mut self.backend{
      RendererBackend::Vulkan(bk) => bk.resumed(window, event_loop)
    }
  }

  pub fn resized( &mut self ){
    match &mut self.backend{
      RendererBackend::Vulkan(bk) => bk.resized()
    }
  }

  pub fn render<F>( &mut self, func: F )
  where
    F: FnOnce(&skia_safe::Canvas, LogicalSize<f32>)
  {
    match &mut self.backend{
      RendererBackend::Vulkan(bk) => bk.render(func),
    }
  }
}