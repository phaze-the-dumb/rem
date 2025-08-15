use std::{ fs::{ self, File }, io::Read, sync::Arc };

use skia_safe::{ Color4f, Font, FontMgr, Paint, Point, Size, Typeface };
use winit::{ application::ApplicationHandler, dpi::PhysicalSize, event::WindowEvent, event_loop::ActiveEventLoop, window::{ Icon, Window, WindowAttributes, WindowId } };

use crate::{ context::VulkanRenderContext, renderer::VulkanRenderer };

#[derive(Default)]
pub struct AppCreateInfo{
  pub title: String,
  pub size: ( u32, u32 ),
  pub icon: Option<String>
}

#[derive(Default)]
pub struct App{
  render_ctx: VulkanRenderContext,
  renderer: Option<VulkanRenderer>,
  create_info: AppCreateInfo
}

impl App{
  pub fn new( info: AppCreateInfo ) -> Self{
    let app = Self {
      create_info: info,
      ..Default::default()
    };

    app
  }
}

impl ApplicationHandler for App{
  fn resumed(&mut self, event_loop: &ActiveEventLoop) {
    let mut icon = None;

    if self.create_info.icon.is_some(){
      let (icon_rgba, icon_width, icon_height) = {
        let image = image::load_from_memory(&fs::read(self.create_info.icon.clone().unwrap()).unwrap()).unwrap().into_rgba8();
        let (width, height) = image.dimensions();
        let rgba = image.into_raw();
        (rgba, width, height)
      };

      icon = Some(Icon::from_rgba(icon_rgba, icon_width, icon_height).unwrap());
    }

    let attr = Window::default_attributes()
      .with_title(self.create_info.title.clone())
      .with_inner_size(winit::dpi::Size::Physical(
        PhysicalSize::new(self.create_info.size.0, self.create_info.size.1)
      ))
      .with_window_icon(icon);

    let window = Arc::new(event_loop.create_window(attr).unwrap());
    self.renderer = Some(self.render_ctx.renderer_for_window(event_loop, window.clone()));
  }

  fn window_event(
    &mut self,
    event_loop: &ActiveEventLoop,
    window_id: WindowId,
    event: WindowEvent,
  ) {
    match event{
      WindowEvent::CloseRequested => {
        event_loop.exit();
      },
      WindowEvent::Resized(_) => {
        if let Some(renderer) = self.renderer.as_mut() {
          renderer.invalidate_swapchain();
        }
      }
      WindowEvent::RedrawRequested => {
        if let Some(renderer) = self.renderer.as_mut(){
          renderer.prepare_swapchain();
          renderer.draw_and_present(| canvas, size | {
            let canvas_size = Size::new(size.width, size.height);
            canvas.clear(Color4f::new(0.0, 0.0, 0.0, 1.0));


          });
        }
      },
      _ => {}
    }
  }
}

// fn get_typeface() -> Result<Typeface, ()>{
//   let fm  = FontMgr::new();
//   let mut file = File::open("/usr/share/fonts/TTF/UbuntuMonoNerdFont-Regular.ttf").unwrap();

//   let mut bytes = Vec::new();
//   file.read_to_end(&mut bytes).unwrap();

//   let tf = fm.new_from_data(&bytes, 0).unwrap();
//   Ok(tf)
// }