use std::thread;

use winit::event_loop::{ControlFlow, EventLoop};

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

pub fn build<F>( info: AppCreateInfo, f: F )
where
  F: FnOnce(&mut App)
{
  let event_loop = EventLoop::new().unwrap();
  let mut app = App::new(info);

  event_loop.set_control_flow(ControlFlow::Wait);

  f(&mut app);
  event_loop.run_app(&mut app).ok();
}