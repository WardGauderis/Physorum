#![feature(unboxed_closures)]
#![feature(fn_traits)]

use anyhow::*;
use log::*;
use timer::Timer;
use winit::{
	event::{Event, WindowEvent},
	event_loop::ControlFlow,
	*,
};

mod blit;
mod simulation;
mod state;
mod timer;

fn main() -> Result<()> {
	pretty_env_logger::init();

	let event_loop = event_loop::EventLoop::new();
	let window = window::WindowBuilder::new()
		.with_title("physarum")
		.with_inner_size(dpi::PhysicalSize::new(1920.0, 1080.0))
		.build(&event_loop)?;

	let mut state = pollster::block_on(state::State::new(&window))?;

	let mut timer = Timer::new();

	event_loop.run(move |event, _, control| match event {
		Event::RedrawRequested(window_id) if window_id == window.id() => {
			match state.render(timer.update()) {
				Err(e) => {
					error!("{:?}", e);
					*control = ControlFlow::Exit;
				},
				_ => {},
			};
		},
		Event::RedrawEventsCleared => window.request_redraw(),
		Event::WindowEvent {
			window_id,
			event: WindowEvent::CloseRequested,
		} if window_id == window.id() => {
			*control = ControlFlow::Exit;
		},
		_ => {},
	})
}
