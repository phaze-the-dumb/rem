use skia_safe::Canvas;
use winit::dpi::LogicalSize;

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

pub fn build<F>( info: AppCreateInfo, render: F ) -> App<F>
where
  F: Fn(&Canvas, LogicalSize<f32>)
{
  return App::new(info, render);
}