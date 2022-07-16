use anyhow::*;
use wgpu::*;
use winit::*;

pub(crate) struct State {
	surface: Surface,
	device:  Device,
	queue:   Queue,
}

impl State {
	pub async fn new(window: &window::Window) -> Result<Self> {
		let instance = Instance::new(Backends::VULKAN);

		let surface = unsafe { instance.create_surface(window) };

		let request_adapter_options = RequestAdapterOptions {
			compatible_surface: Some(&surface),
			..Default::default()
		};
		let adapter = instance
			.request_adapter(&request_adapter_options)
			.await
			.ok_or(anyhow!("No adapter found"))?;

		let device_descriptor = DeviceDescriptor::default();
		let (device, queue) = adapter.request_device(&device_descriptor, None).await?;

		let surface_configuration = SurfaceConfiguration {
			usage:        TextureUsages::RENDER_ATTACHMENT,
			format:       TextureFormat::Bgra8UnormSrgb,
			width:        window.inner_size().width,
			height:       window.inner_size().height,
			present_mode: PresentMode::Fifo,
		};
		surface.configure(&device, &surface_configuration);

		Ok(Self {
			surface,
			device,
			queue,
		})
	}

	pub fn render(&self) -> Result<()> {
		let output = self.surface.get_current_texture()?;

		let texture_view_descriptor = TextureViewDescriptor::default();
		let view = output.texture.create_view(&texture_view_descriptor);

		let command_encoder_descriptor = CommandEncoderDescriptor::default();
		let mut encoder = self
			.device
			.create_command_encoder(&command_encoder_descriptor);

		let render_pass_descriptor = RenderPassDescriptor {
			color_attachments: &[Some(RenderPassColorAttachment {
				view:           &view,
				resolve_target: None,
				ops:            Operations::default(),
			})],
			..Default::default()
		};

		let render_pass = encoder.begin_render_pass(&render_pass_descriptor);

		drop(render_pass);

		self.queue.submit(std::iter::once(encoder.finish()));
		output.present();
		Ok(())
	}
}
