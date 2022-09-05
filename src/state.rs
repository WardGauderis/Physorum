use anyhow::*;
use wgpu::{
	util::{BufferInitDescriptor, DeviceExt},
	*,
};
use winit::*;

use crate::{
	blit, simulation,
	simulation::{Agent, Config},
};

// TODO: bundle and reuse commands (do as little as possible in render)
// TODO: merge into create_pipeline, create_texture
// TODO: resize
// TODO: music
// TODO: multiple types
// TODO: modulo, init config

pub struct State {
	surface: Surface,
	device:  Device,
	queue:   Queue,

	extend_3d: Extent3d,

	compute_pipeline:   ComputePipeline,
	compute_bind_group: BindGroup,
	compute_texture:    Texture,

	render_pipeline:     RenderPipeline,
	render_bind_group:   BindGroup,
	render_texture:      Texture,
	render_texture_view: TextureView,

	blit: blit::Blit,

	config: simulation::Config,
}

impl State {
	pub async fn new(window: &window::Window, config: Config) -> Result<State> {
		let instance = Instance::new(Backends::VULKAN);

		let surface = unsafe { instance.create_surface(window) };

		let request_adapter_options = RequestAdapterOptions {
			power_preference:       Default::default(),
			force_fallback_adapter: false,
			compatible_surface:     Some(&surface),
		};
		let adapter = instance
			.request_adapter(&request_adapter_options)
			.await
			.ok_or(anyhow!("No adapter found"))?;

		let device_descriptor = DeviceDescriptor {
			label:    None,
			features: Features::TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES | Features::PUSH_CONSTANTS,
			limits:   Limits {
				max_push_constant_size: std::mem::size_of::<simulation::Config>() as u32,
				..Default::default()
			},
		};
		let (device, queue) = adapter.request_device(&device_descriptor, None).await?;

		let surface_configuration = SurfaceConfiguration {
			usage:        TextureUsages::RENDER_ATTACHMENT,
			format:       TextureFormat::Bgra8Unorm,
			width:        window.inner_size().width,
			height:       window.inner_size().height,
			present_mode: PresentMode::Fifo,
		};
		surface.configure(&device, &surface_configuration);

		////////////////////////////////////////////////////////////////////////////////////////////
		// Simulation
		////////////////////////////////////////////////////////////////////////////////////////////

		let agents = Agent::generate_random(config.num_agents,  window.inner_size().width, window.inner_size().height);

		////////////////////////////////////////////////////////////////////////////////////////////
		// Compute pipeline
		////////////////////////////////////////////////////////////////////////////////////////////

		let extend_3d = Extent3d {
			width:                 surface_configuration.width,
			height:                surface_configuration.height,
			depth_or_array_layers: 1,
		};

		let mut texture_descriptor = TextureDescriptor {
			label:           None,
			size:            extend_3d,
			mip_level_count: 1,
			sample_count:    1,
			dimension:       TextureDimension::D2,
			format:          TextureFormat::Rgba8Unorm,
			usage:           TextureUsages::TEXTURE_BINDING
				| TextureUsages::STORAGE_BINDING
				| TextureUsages::COPY_DST,
		};
		let compute_texture = device.create_texture(&texture_descriptor);

		let texture_view_descriptor = TextureViewDescriptor::default();
		let texture_view = compute_texture.create_view(&texture_view_descriptor);

		let shader_module_descriptor = include_wgsl!("compute.wgsl");
		let shader_module = device.create_shader_module(shader_module_descriptor);

		let buffer_init_descriptor = BufferInitDescriptor {
			label:    None,
			contents: bytemuck::cast_slice(&agents),
			usage:    BufferUsages::STORAGE,
		};
		let buffer = device.create_buffer_init(&buffer_init_descriptor);

		let bind_group_layout_descriptor = BindGroupLayoutDescriptor {
			label:   None,
			entries: &[
				BindGroupLayoutEntry {
					binding:    0,
					visibility: ShaderStages::COMPUTE,
					ty:         BindingType::StorageTexture {
						access:         StorageTextureAccess::ReadWrite,
						format:         texture_descriptor.format,
						view_dimension: Default::default(),
					},
					count:      None,
				},
				BindGroupLayoutEntry {
					binding:    1,
					visibility: ShaderStages::COMPUTE,
					ty:         BindingType::Buffer {
						ty:                 BufferBindingType::Storage { read_only: false },
						has_dynamic_offset: false,
						min_binding_size:   None,
					},
					count:      None,
				},
			],
		};
		let bind_group_layout = device.create_bind_group_layout(&bind_group_layout_descriptor);
		let bind_group_descriptor = BindGroupDescriptor {
			label:   None,
			layout:  &bind_group_layout,
			entries: &[
				BindGroupEntry {
					binding:  0,
					resource: BindingResource::TextureView(&texture_view),
				},
				BindGroupEntry {
					binding:  1,
					resource: buffer.as_entire_binding(),
				},
			],
		};
		let compute_bind_group = device.create_bind_group(&bind_group_descriptor);

		let pipeline_layout_descriptor = PipelineLayoutDescriptor {
			label:                None,
			bind_group_layouts:   &[&bind_group_layout],
			push_constant_ranges: &[PushConstantRange {
				stages: ShaderStages::COMPUTE,
				range:  0 .. std::mem::size_of::<simulation::Config>() as u32,
			}],
		};
		let pipeline_layout = device.create_pipeline_layout(&pipeline_layout_descriptor);
		let pipeline_descriptor = ComputePipelineDescriptor {
			label:       None,
			layout:      Some(&pipeline_layout),
			module:      &shader_module,
			entry_point: "main",
		};
		let compute_pipeline = device.create_compute_pipeline(&pipeline_descriptor);

		////////////////////////////////////////////////////////////////////////////////////////////
		// Render pipeline
		////////////////////////////////////////////////////////////////////////////////////////////

		texture_descriptor.usage = TextureUsages::TEXTURE_BINDING
			| TextureUsages::RENDER_ATTACHMENT
			| TextureUsages::COPY_SRC;
		let render_texture = device.create_texture(&texture_descriptor);

		let render_texture_view = render_texture.create_view(&texture_view_descriptor);

		let shader_module_descriptor = include_wgsl!("shader.wgsl");
		let shader_module = device.create_shader_module(shader_module_descriptor);

		let bind_group_layout_descriptor = BindGroupLayoutDescriptor {
			label:   None,
			entries: &[BindGroupLayoutEntry {
				binding:    0,
				visibility: ShaderStages::FRAGMENT,
				ty:         BindingType::Texture {
					sample_type:    Default::default(),
					view_dimension: Default::default(),
					multisampled:   false,
				},
				count:      None,
			}],
		};
		let bind_group_layout = device.create_bind_group_layout(&bind_group_layout_descriptor);
		let bind_group_descriptor = BindGroupDescriptor {
			label:   None,
			layout:  &bind_group_layout,
			entries: &[BindGroupEntry {
				binding:  0,
				resource: BindingResource::TextureView(&texture_view),
			}],
		};
		let render_bind_group = device.create_bind_group(&bind_group_descriptor);

		let pipeline_layout_descriptor = PipelineLayoutDescriptor {
			label:                None,
			bind_group_layouts:   &[&bind_group_layout],
			push_constant_ranges: &[PushConstantRange {
				stages: ShaderStages::FRAGMENT,
				range:  0 .. std::mem::size_of::<simulation::Config>() as u32,
			}],
		};
		let pipeline_layout = device.create_pipeline_layout(&pipeline_layout_descriptor);
		let render_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
			label:         None,
			layout:        Some(&pipeline_layout),
			vertex:        VertexState {
				module:      &shader_module,
				entry_point: "vs_main",
				buffers:     &[],
			},
			primitive:     Default::default(),
			depth_stencil: None,
			multisample:   Default::default(),
			fragment:      Some(FragmentState {
				module:      &shader_module,
				entry_point: "fs_main",
				targets:     &[Some(ColorTargetState {
					format:     TextureFormat::Rgba8Unorm,
					blend:      None,
					write_mask: Default::default(),
				})],
			}),
			multiview:     None,
		});

		let blit = blit::Blit::new(&device, &render_texture_view);

		Ok(Self {
			surface,
			device,
			queue,

			extend_3d,

			compute_pipeline,
			compute_bind_group,
			compute_texture,

			render_pipeline,
			render_bind_group,
			render_texture,
			render_texture_view,

			blit,

			config,
		})
	}

	pub fn render(&mut self, delta_time: f32) -> Result<()> {
		self.config.time += delta_time;
		self.config.delta_time = delta_time;

		let command_encoder_descriptor = CommandEncoderDescriptor::default();
		let mut encoder = self
			.device
			.create_command_encoder(&command_encoder_descriptor);

		let output = self.surface.get_current_texture()?;
		let view = output
			.texture
			.create_view(&TextureViewDescriptor::default());

		let compute_pass_descriptor = ComputePassDescriptor::default();
		let mut compute_pass = encoder.begin_compute_pass(&compute_pass_descriptor);

		compute_pass.set_pipeline(&self.compute_pipeline);
		compute_pass.set_bind_group(0, &self.compute_bind_group, &[]);
		compute_pass.set_push_constants(0, bytemuck::cast_slice(&[self.config]));
		compute_pass.dispatch_workgroups(
			(self.config.num_agents as f32 / 64.0).ceil() as u32,
			1,
			1,
		);

		drop(compute_pass);

		let render_pass_descriptor = RenderPassDescriptor {
			label:                    None,
			color_attachments:        &[Some(RenderPassColorAttachment {
				view:           &self.render_texture_view,
				resolve_target: None,
				ops:            Operations {
					load:  LoadOp::Clear(Color::RED),
					store: true,
				},
			})],
			depth_stencil_attachment: None,
		};
		let mut render_pass = encoder.begin_render_pass(&render_pass_descriptor);
		render_pass.set_pipeline(&self.render_pipeline);
		render_pass.set_bind_group(0, &self.render_bind_group, &[]);
		render_pass.set_push_constants(
			ShaderStages::FRAGMENT,
			0,
			bytemuck::cast_slice(&[self.config]),
		);
		render_pass.draw(0 .. 3, 0 .. 1);

		drop(render_pass);

		encoder.copy_texture_to_texture(
			self.render_texture.as_image_copy(),
			self.compute_texture.as_image_copy(),
			self.extend_3d,
		);

		self.blit.blit(&mut encoder, &view);

		self.queue.submit(std::iter::once(encoder.finish()));
		output.present();
		Ok(())
	}
}
