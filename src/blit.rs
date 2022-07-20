use wgpu::*;

pub struct Blit {
	pipeline:   RenderPipeline,
	bind_group: BindGroup,
}

impl Blit {
	pub fn new(device: &Device, source: &TextureView) -> Self {
		let shader_module_descriptor = include_wgsl!("blit.wgsl");
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
				resource: BindingResource::TextureView(source),
			}],
		};
		let bind_group = device.create_bind_group(&bind_group_descriptor);

		let pipeline_layout_descriptor = PipelineLayoutDescriptor {
			label:                None,
			bind_group_layouts:   &[&bind_group_layout],
			push_constant_ranges: &[],
		};
		let pipeline_layout = device.create_pipeline_layout(&pipeline_layout_descriptor);
		let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
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
					format:     TextureFormat::Bgra8Unorm,
					blend:      None,
					write_mask: Default::default(),
				})],
			}),
			multiview:     None,
		});

		Blit {
			pipeline,
			bind_group,
		}
	}

	pub fn blit(&self, encoder: &mut CommandEncoder, destination: &TextureView) {
		let render_pass_descriptor = RenderPassDescriptor {
			label:                    None,
			color_attachments:        &[Some(RenderPassColorAttachment {
				view:           destination,
				resolve_target: None,
				ops:            Operations {
					load:  LoadOp::Clear(Color::GREEN),
					store: true,
				},
			})],
			depth_stencil_attachment: None,
		};
		let mut render_pass = encoder.begin_render_pass(&render_pass_descriptor);
		render_pass.set_pipeline(&self.pipeline);
		render_pass.set_bind_group(0, &self.bind_group, &[]);
		render_pass.draw(0 .. 3, 0 .. 1);
	}
}
