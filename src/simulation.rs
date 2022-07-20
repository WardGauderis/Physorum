use std::f32::consts::PI;
use nalgebra::Vector2;
use rand::prelude::*;

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Agent {
	pub position: [f32; 2],
	pub angle:    f32,
	pub _fill:    f32,
}

impl Agent {
	pub fn random() -> Self {
		let center: Vector2<f32> = [1920.0 / 2.0, 1080.0 / 2.0].into();
		let rand: Vector2<f32> = rand_distr::UnitDisc.sample(&mut thread_rng()).into();
		Self {
			position: (center + rand * 1080.0 * 0.45).into(),
			angle:    random::<f32>() * 2.0 * PI,
			_fill:    0.0,
		}
	}

	pub fn generate_random(n: u32) -> Vec<Self> { (0 .. n).map(|_| Self::random()).collect() }
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Config {
	pub num_agents: u32,

	pub move_speed:             f32,
	pub turn_speed:             f32,
	pub sensor_angle:           f32,
	pub sensor_offset_distance: f32,
	pub sensor_width:           i32,

	pub diffuse_rate: f32,
	pub decay_rate:   f32,

	pub time:       f32,
	pub delta_time: f32,
}
