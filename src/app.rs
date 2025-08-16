use std::{ fs, path::PathBuf, sync::Arc };

use skia_safe::{ Canvas, Color4f, Size };
use winit::{ application::ApplicationHandler, dpi::{LogicalSize, PhysicalSize}, event::WindowEvent, event_loop::ActiveEventLoop, window::{ Icon, Window, WindowId } };

use crate::{ app::{config::AppConfig, node::Node}, renderer::Renderer, structs::config::Config, AppCreateInfo };

mod config;
pub mod node;

pub struct App{
  renderer: Renderer,
  create_info: AppCreateInfo,
  config: AppConfig,
  loaded_view: Vec<Node>,
  window: Option<Arc<Window>>
}

impl App{
  pub fn new( info: AppCreateInfo ) -> Self{
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
      config: AppConfig::new(config_file_dir),
      renderer: Renderer::new(config.renderer),
      loaded_view: Vec::new(),
      window: None
    };

    app
  }

  pub fn get_dir( &self ) -> PathBuf{
    dirs::config_dir().unwrap().join(self.create_info.name)
  }

  pub fn load_view( &mut self, nodes: Vec<Node> ){
    self.loaded_view = nodes;
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
      .with_title(self.create_info.title)
      .with_inner_size(winit::dpi::Size::Physical(
        PhysicalSize::new(self.create_info.size.0, self.create_info.size.1)
      ))
      .with_window_icon(icon);

    let window = Arc::new(event_loop.create_window(attr).unwrap());

    self.window = Some(window.clone());
    self.renderer.resumed(window, event_loop);
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
        self.renderer.resized();
      }
      WindowEvent::RedrawRequested => {
        self.renderer.render(| canvas: &Canvas, size: LogicalSize<f32> | {
          let canvas_size = Size::new(size.width, size.height);
          canvas.clear(Color4f::new(1.0, 1.0, 1.0, 1.0));

          for node in &self.loaded_view{ node.render(canvas); }
        });
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