use std::f32::consts::PI;

use nalgebra::Vector2;
use rand::prelude::*;

use crate::Settings;

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Agent {
	pub position: [f32; 2],
	pub angle:    f32,
	pub _fill:    f32,
}

impl Agent {
	pub fn random(width: u32, height: u32) -> Self {
		let center: Vector2<f32> = [width as f32 * 2.0 / 2.0, height as f32* 2.0/ 2.0].into();
		let rand: Vector2<f32> = rand_distr::UnitDisc.sample(&mut thread_rng()).into();
		Self {
			position: (center + rand * width.min(height) as f32 * 0.3).into(),
			angle:    random::<f32>() * 2.0 * PI,
			_fill:    0.0,
		}
	}

	pub fn generate_random(n: u32, width: u32, height: u32) -> Vec<Self> { (0 .. n).map(|_| Self::random(width, height)).collect() }
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Config {
	pub num_agents: u32,

	pub move_speed:             f32,
	pub turn_speed:             f32,
	pub sensor_angle:           f32,
	pub sensor_offset: f32,
	pub sensor_width:           i32,

	pub diffuse_rate: f32,
	pub decay_rate:   f32,

	pub time:       f32,
	pub delta_time: f32,
}

impl From<Settings> for Config {
	fn from(settings: Settings) -> Self {
		Self {
			num_agents: settings.simulation.num_agents,

			move_speed:             settings.simulation.move_speed,
			turn_speed:             settings.simulation.turn_speed,
			sensor_angle:           settings.simulation.sensor_angle * PI / 180.0,
			sensor_offset: settings.simulation.sensor_offset,
			sensor_width:           settings.simulation.sensor_width,

			diffuse_rate: settings.simulation.diffuse_rate,
			decay_rate:   settings.simulation.decay_rate,

			time:       0.0,
			delta_time: 0.0,
		}
	}
}
