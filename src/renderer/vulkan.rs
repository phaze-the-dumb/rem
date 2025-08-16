use std::sync::Arc;

use winit::{ dpi::LogicalSize, event_loop::ActiveEventLoop, window::Window };

use crate::renderer::{ vulkan::{ vulkan_context::VulkanRenderContext, vulkan_renderer::VulkanRenderer } };

pub mod vulkan_context;
pub mod vulkan_renderer;

#[derive(Default)]
pub struct VulkanBackend{
  render_ctx: VulkanRenderContext,
  renderer: Option<VulkanRenderer>,
}

impl VulkanBackend{
  pub fn resumed( &mut self, window: Arc<Window>, event_loop: &ActiveEventLoop ){
    self.renderer = Some(self.render_ctx.renderer_for_window(event_loop, window.clone()));
  }

  pub fn resized( &mut self ){
    if let Some(renderer) = self.renderer.as_mut() {
      renderer.invalidate_swapchain();
    }
  }

  pub fn render<F>( &mut self, f: F )
  where
    F: FnOnce(&skia_safe::Canvas, LogicalSize<f32>)
  {
    if let Some(renderer) = self.renderer.as_mut(){
      renderer.prepare_swapchain();
      renderer.draw_and_present(f);
    }
  }
}