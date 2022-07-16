use std::time::*;

use log::*;

pub(crate) struct Timer {
	last_call:  Instant,
	call_count: u64,
	accum_time: Duration,
}

impl Timer {
	pub fn new() -> Self {
		Self {
			last_call:  Instant::now(),
			call_count: 0,
			accum_time: Duration::default(),
		}
	}

	pub fn update(&mut self) {
		self.accum_time += self.last_call.elapsed();
		self.last_call = Instant::now();
		self.call_count += 1;
		if self.accum_time > Duration::from_secs(1) {
			debug!(
				"{} fps",
				self.call_count as f32 / self.accum_time.as_secs_f32() * 1000.0
			);
			self.accum_time = Duration::default();
			self.call_count = 0;
		}
	}
}
