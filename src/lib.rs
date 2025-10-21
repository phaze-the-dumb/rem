use skia_safe::Canvas;
use winit::{ dpi::LogicalSize, event_loop::{ ControlFlow, EventLoop } };

use crate::app::App;

mod renderer;
mod structs;
pub mod app;

#[derive(Default)]
pub struct AppCreateInfo{
  pub title: &'static str,
  pub size: ( u32, u32 ),
  pub icon: Option<&'static str>,

  pub name: &'static str
}

pub fn build<F>( info: AppCreateInfo, render: F )
where
  F: Fn(&Canvas, LogicalSize<f32>)
{
  let event_loop = EventLoop::new().unwrap();
  let mut app = App::new(info, render);

  event_loop.set_control_flow(ControlFlow::Wait);
  event_loop.run_app(&mut app).ok();
}