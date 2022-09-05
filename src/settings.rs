use serde::Deserialize;


#[derive(Deserialize)]
pub struct Settings {
	pub width:  u32,
	pub height: u32,
	pub fullscreen: bool,
}
