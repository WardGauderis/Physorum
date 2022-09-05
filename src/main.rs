use anyhow::*;
use config::{Config, File};
use log::*;
use settings::Settings;
use timer::Timer;
use winit::{
	event::{Event, WindowEvent},
	event_loop::ControlFlow,
	*,
};

mod blit;
mod settings;
mod simulation;
mod state;
mod timer;

fn main() -> Result<()> {
	pretty_env_logger::init();

	let settings: Settings = Config::builder()
		.add_source(File::with_name("physorum.toml"))
		.build()?
		.try_deserialize()?;

	let event_loop = event_loop::EventLoop::new();
	let window = window::WindowBuilder::new()
		.with_title("physarum")
		.with_inner_size(dpi::PhysicalSize::new(settings.width, settings.height))
		.with_fullscreen(if settings.fullscreen {
			Some(window::Fullscreen::Borderless(None))
		} else {
			None
		})
		.build(&event_loop)?;

	let mut state = pollster::block_on(state::State::new(&window))?;

	let mut timer = Timer::new();

	event_loop.run(move |event, _, control| match event {
		Event::RedrawRequested(window_id) if window_id == window.id() => {
			if let Err(e) = state.render(timer.update()) {
				error!("{:?}", e);
				*control = ControlFlow::Exit;
			};
		},
		Event::RedrawEventsCleared => window.request_redraw(),
		Event::WindowEvent { window_id, event } if window_id == window.id() => match event {
			WindowEvent::CloseRequested => *control = ControlFlow::Exit,
			WindowEvent::Resized(size) => {},
			_ => {},
		},
		_ => {},
	})
}
