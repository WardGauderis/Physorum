use serde::Deserialize;

#[derive(Deserialize)]
pub struct Display {
	pub width:      u32,
	pub height:     u32,
	pub fullscreen: bool,
}

#[derive(Deserialize)]
pub struct Simulation {
	pub num_agents: u32,

	pub move_speed:    f32,
	pub turn_speed:    f32,
	pub sensor_angle:  f32,
	pub sensor_offset: f32,
	pub sensor_width:  i32,

	pub diffuse_rate: f32,
	pub decay_rate:   f32,
}

#[derive(Deserialize)]
pub struct Settings {
	pub display:    Display,
	pub simulation: Simulation,
}
