use winit::event_loop::{ControlFlow, EventLoop};

use crate::app::{ App, AppCreateInfo };

mod context;
mod renderer;
mod app;

pub fn build( info: AppCreateInfo ) {
  let event_loop = EventLoop::new().unwrap();
  let mut app = App::new(info);

  event_loop.set_control_flow(ControlFlow::Wait);
  event_loop.run_app(&mut app).ok();
}