use std::{ fs, path::PathBuf, sync::Arc };

use skia_safe::Canvas;
use winit::{ application::ApplicationHandler, dpi::{LogicalSize, PhysicalSize}, event::WindowEvent, event_loop::{ActiveEventLoop, ControlFlow, EventLoop}, window::{ Icon, Window, WindowId } };

use crate::{ app::config::AppConfig, renderer::Renderer, structs::config::Config, AppCreateInfo };

mod config;

pub struct App<F>
where
  F: Fn(&Canvas, LogicalSize<f32>)
{
  renderer: Renderer,
  create_info: AppCreateInfo,
  _config: AppConfig,
  window: Option<Arc<Window>>,
  render: F,
  event: Option<Box<dyn Fn(&App<F>, WindowEvent)>>
}

impl<F> App<F>
where
  F: Fn(&Canvas, LogicalSize<f32>)
{
  pub fn new( info: AppCreateInfo, render: F ) -> Self
  where
    F: Fn(&Canvas, LogicalSize<f32>)
  {
    let config_dir = dirs::config_dir().unwrap().join(info.name);
    if !config_dir.exists() { fs::create_dir(&config_dir).unwrap(); }

    let config_file_dir = config_dir.clone().join("rem.conf");
    let config: Config;

    if !config_file_dir.exists() {
      config = Config::default();
      fs::write(&config_file_dir, serde_json::to_string(&config).unwrap()).unwrap();
    } else{
      let json_str = fs::read_to_string(&config_file_dir).unwrap();
      config = serde_json::from_str(&json_str).unwrap();
    }

    let app = Self {
      create_info: info,
      _config: AppConfig::new(config_file_dir),
      renderer: Renderer::new(config.renderer),
      window: None,
      render: render,
      event: None
    };

    app
  }

  pub fn get_dir( &self ) -> PathBuf{
    dirs::config_dir().unwrap().join(self.create_info.name)
  }

  pub fn events( &mut self, listener: Box<dyn Fn(&App<F>, WindowEvent)> ){
    self.event = Some(listener);
  }

  pub fn run( &mut self ){
    let event_loop = EventLoop::new().unwrap();

    event_loop.set_control_flow(ControlFlow::Wait);
    event_loop.run_app(self).ok();
  }

  pub fn redraw( &self ){
    if self.window.is_some(){
      self.window.as_ref().unwrap().request_redraw(); }
  }
}

impl<F> ApplicationHandler for App<F>
where
  F: Fn(&Canvas, LogicalSize<f32>)
{
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
      .with_title(self.create_info.title)
      .with_inner_size(winit::dpi::Size::Physical(
        PhysicalSize::new(self.create_info.size.0, self.create_info.size.1)
      ));

    let window = Arc::new(event_loop.create_window(attr).unwrap());

    if icon.is_some(){
      window.set_window_icon(icon);
    }

    self.window = Some(window.clone());
    self.renderer.resumed(window, event_loop);
  }

  fn window_event(
    &mut self,
    event_loop: &ActiveEventLoop,
    _window_id: WindowId,
    event: WindowEvent,
  ) {
    match event{
      WindowEvent::CloseRequested => {
        event_loop.exit();
      },
      WindowEvent::Resized(_) => {
        self.renderer.resized();

        let event_cb = &self.event;
        if event_cb.is_some(){
          let event_cb = event_cb.as_ref().unwrap();
          event_cb(self, event);
        }
      }
      WindowEvent::RedrawRequested => {
        self.renderer.render(| canvas: &Canvas, size: LogicalSize<f32> | {
          let render = &self.render;

          render(canvas, size);
        });
      },
      WindowEvent::CursorMoved { device_id: _, position: _ } => {
        let event_cb = &self.event;
        if event_cb.is_some(){
          let event_cb = event_cb.as_ref().unwrap();
          event_cb(self, event);
        }
      },
      WindowEvent::MouseInput { device_id: _, state: _, button: _ } => {
        let event_cb = &self.event;
        if event_cb.is_some(){
          let event_cb = event_cb.as_ref().unwrap();
          event_cb(self, event);
        }
      },
      WindowEvent::MouseWheel { device_id: _, delta: _, phase: _ } => {
        let event_cb = &self.event;
        if event_cb.is_some(){
          let event_cb = event_cb.as_ref().unwrap();
          event_cb(self, event);
        }
      },
      WindowEvent::KeyboardInput { device_id: _, event: _, is_synthetic: _ } => {
        let event_cb = &self.event;
        if event_cb.is_some(){
          let event_cb = event_cb.as_ref().unwrap();
          event_cb(self, event);
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

// let mut p = Paint::new(Color4f::new(0.0, 0.0, 1.0, 1.0), None);
//           p.set_image_filter(image_filters::blur((100.0, 100.0), None, None, None));

//           let rect_size = canvas_size / 2.0;
//           let rect = Rect::from_point_and_size(
//               Point::new(
//                 (canvas_size.width - rect_size.width) / 2.0,
//                 (canvas_size.height - rect_size.height) / 2.0,
//               ),
//               rect_size,
//           );
//           canvas.draw_rect(
//             rect,
//             &p,
//           );