use ab_glyph::Font;
use std::collections::HashMap;
use core::num::NonZeroU8;
use crate::sequence::GridSequence;
use crate::atlas::populate_atlas;

#[inline]
fn compute_output_size<const W: usize, const H: usize>(font_width: u32, font_height: u32) -> (u32, u32) {
	let output_width = W as u32 * font_width;
	let output_height = H as u32 * font_height;
	(output_width, output_height)
}

pub struct WgpuRenderer<const W: usize, const H: usize> {
	sequence: GridSequence<W, H>,
	lut: HashMap<char, u32>,
	device: wgpu::Device,
	queue: wgpu::Queue,
	idx_grid: wgpu::Buffer,
	output_img: wgpu::Buffer,
	pipeline: wgpu::ComputePipeline,
	bind_group: wgpu::BindGroup,
	output_width: u32,
	output_height: u32
}

impl<const W: usize, const H: usize> WgpuRenderer<W, H> {
	pub async fn new<F: Font>(font: F, sequence: GridSequence<W, H>) -> Self {
		let populated_atlas = populate_atlas(font, &sequence);

		let (output_width, output_height) = compute_output_size::<W, H>(
			populated_atlas.font_width,
			populated_atlas.font_height
		);

		let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
			backends: wgpu::Backends::all(),
			flags: wgpu::InstanceFlags::VALIDATION,
			backend_options: wgpu::BackendOptions::default()
		});

		let adapter = instance.request_adapter(&wgpu::RequestAdapterOptions::default()).await.unwrap();
		// I expect that the size of the atlas should be bounded by the size of the output
		let max_buf_size = output_width * output_height * 4;

		let (device, queue) = adapter.request_device(&wgpu::DeviceDescriptor {
			required_features: wgpu::Features::SHADER_INT64,
			required_limits: wgpu::Limits {
				max_buffer_size: max_buf_size as u64,
				max_storage_buffer_binding_size: max_buf_size,
				..wgpu::Limits::default()
			},
			memory_hints: wgpu::MemoryHints::Performance,
			label: Some("device"),
			trace: wgpu::Trace::Off
		}).await.unwrap();

		let idx_grid = device.create_buffer(&wgpu::BufferDescriptor {
			label: Some("idx_grid"),
			size: (H * W * 4) as u64,
			usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::STORAGE,
			mapped_at_creation: false
		});

		let atlas = device.create_buffer(&wgpu::BufferDescriptor {
			label: Some("atlas"),
			size: populated_atlas.buffer.len() as u64,
			usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::STORAGE,
			mapped_at_creation: false
		});

		// TODO: investigate efficiency of `write_buffer`
		queue.write_buffer(&atlas, 0, &populated_atlas.buffer);

		let output_img = device.create_buffer(&wgpu::BufferDescriptor {
			label: Some("output_buf"),
			size: output_width as u64 * output_height as u64 * 4,
			usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
			mapped_at_creation: false
		});

		let grid_width_uniform = device.create_buffer(&wgpu::BufferDescriptor {
			label: Some("grid_width_uniform"),
			size: 4,
			usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
			mapped_at_creation: false
		});

		let grid_height_uniform = device.create_buffer(&wgpu::BufferDescriptor {
			label: Some("grid_height_uniform"),
			size: 4,
			usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
			mapped_at_creation: false
		});

		queue.write_buffer(&grid_width_uniform, 0, &(W as u32).to_ne_bytes());
		queue.write_buffer(&grid_height_uniform, 0, &(H as u32).to_ne_bytes());

		let img_width_uniform = device.create_buffer(&wgpu::BufferDescriptor {
			label: Some("img_width_uniform"),
			size: 4,
			usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
			mapped_at_creation: false
		});

		let img_height_uniform = device.create_buffer(&wgpu::BufferDescriptor {
			label: Some("img_height_uniform"),
			size: 4,
			usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
			mapped_at_creation: false
		});

		queue.write_buffer(&img_width_uniform, 0, &output_width.to_ne_bytes());
		queue.write_buffer(&img_height_uniform, 0, &output_height.to_ne_bytes());

		let shader = device.create_shader_module(wgpu::include_wgsl!("shader.wgsl"));

		let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
			label: Some("bind_group_layout"),
			entries: &[
				// idx_grid
				wgpu::BindGroupLayoutEntry {
					binding: 0,
					visibility: wgpu::ShaderStages::COMPUTE,
					ty: wgpu::BindingType::Buffer {
						ty: wgpu::BufferBindingType::Storage {
							read_only: true
						},
						has_dynamic_offset: false,
						min_binding_size: None
					},
					count: None
				},
				// atlas
				wgpu::BindGroupLayoutEntry {
					binding: 1,
					visibility: wgpu::ShaderStages::COMPUTE,
					ty: wgpu::BindingType::Buffer {
						ty: wgpu::BufferBindingType::Storage {
							read_only: true
						},
						has_dynamic_offset: false,
						min_binding_size: None
					},
					count: None
				},
				// output_img
				wgpu::BindGroupLayoutEntry {
					binding: 2,
					visibility: wgpu::ShaderStages::COMPUTE,
					ty: wgpu::BindingType::Buffer {
						ty: wgpu::BufferBindingType::Storage {
							read_only: false
						},
						has_dynamic_offset: false,
						min_binding_size: None
					},
					count: None
				},
				// grid_width
				wgpu::BindGroupLayoutEntry {
					binding: 3,
					visibility: wgpu::ShaderStages::COMPUTE,
					ty: wgpu::BindingType::Buffer {
						ty: wgpu::BufferBindingType::Uniform,
						has_dynamic_offset: false,
						min_binding_size: None
					},
					count: None
				},
				// grid_height
				wgpu::BindGroupLayoutEntry {
					binding: 4,
					visibility: wgpu::ShaderStages::COMPUTE,
					ty: wgpu::BindingType::Buffer {
						ty: wgpu::BufferBindingType::Uniform,
						has_dynamic_offset: false,
						min_binding_size: None
					},
					count: None
				},
				// grid_width
				wgpu::BindGroupLayoutEntry {
					binding: 5,
					visibility: wgpu::ShaderStages::COMPUTE,
					ty: wgpu::BindingType::Buffer {
						ty: wgpu::BufferBindingType::Uniform,
						has_dynamic_offset: false,
						min_binding_size: None
					},
					count: None
				},
				// grid_height
				wgpu::BindGroupLayoutEntry {
					binding: 6,
					visibility: wgpu::ShaderStages::COMPUTE,
					ty: wgpu::BindingType::Buffer {
						ty: wgpu::BufferBindingType::Uniform,
						has_dynamic_offset: false,
						min_binding_size: None
					},
					count: None
				}
			]
		});

		let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
			label: Some("pipeline_layout"),
			bind_group_layouts: &[&bind_group_layout],
			push_constant_ranges: &[]
		});

		let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
			label: Some("pipeline"),
			layout: Some(&pipeline_layout),
			module: &shader,
			entry_point: Some("sample_atlas"),
			compilation_options: wgpu::PipelineCompilationOptions::default(),
			cache: None
		});

		let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
			label: Some("bind_group"),
			layout: &bind_group_layout,
			entries: &[
				wgpu::BindGroupEntry {
					binding: 0,
					resource: idx_grid.as_entire_binding()
				},
				wgpu::BindGroupEntry {
					binding: 1,
					resource: atlas.as_entire_binding()
				},
				wgpu::BindGroupEntry {
					binding: 2,
					resource: output_img.as_entire_binding()
				},
				wgpu::BindGroupEntry {
					binding: 3,
					resource: grid_width_uniform.as_entire_binding()
				},
				wgpu::BindGroupEntry {
					binding: 4,
					resource: grid_height_uniform.as_entire_binding()
				},
				wgpu::BindGroupEntry {
					binding: 5,
					resource: img_width_uniform.as_entire_binding()
				},
				wgpu::BindGroupEntry {
					binding: 6,
					resource: img_height_uniform.as_entire_binding()
				}
			]
		});

		Self {
			sequence,
			lut: populated_atlas.lut,
			device,
			queue,
			idx_grid,
			output_img,
			pipeline,
			bind_group,
			output_width,
			output_height
		}
	}
}

pub struct RenderedFrame {
	pub img: image::RgbaImage,
	pub frame_hold: NonZeroU8
}

impl RenderedFrame {
	fn deserialize(width: u32, height: u32, data: Vec<u8>, frame_hold: NonZeroU8) -> Self {
		Self {
			img: image::RgbaImage::from_raw(width, height, data).unwrap(),
			frame_hold
		}
	}
}

#[inline]
fn int_div_round_up(divisor: u32, dividend: u32) -> u32 {
	(divisor / dividend) + match divisor % dividend {
		0 => 0,
		_ => 1
	}
}

impl<const W: usize, const H: usize> Iterator for WgpuRenderer<W, H> {
	type Item = RenderedFrame;

	fn next(&mut self) -> Option<Self::Item> {
		let frame = self.sequence.pop()?;

		let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
			label: Some("encoder")
		});

		let frame_hold = frame.frame_hold;
		self.queue.write_buffer(&self.idx_grid, 0, &frame.serialize(&self.lut));

		let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
			label: Some("sample_compute_pass"),
			timestamp_writes: None
		});
		compute_pass.set_pipeline(&self.pipeline);
		compute_pass.set_bind_group(0, &self.bind_group, &[]);
		compute_pass.dispatch_workgroups(
			int_div_round_up(self.output_width, 16),
			int_div_round_up(self.output_height, 16),
			1
		);
		drop(compute_pass);

		let pixels = self.output_width * self.output_height;
		let map_buf = self.device.create_buffer(&wgpu::BufferDescriptor {
			label: Some("map_buf"),
			size: pixels as u64 * 4,
			usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
			mapped_at_creation: false
		});

		encoder.copy_buffer_to_buffer(
			&self.output_img, 0,
			&map_buf, 0,
			pixels as u64 * 4
		);

		self.queue.submit(std::iter::once(encoder.finish()));

		map_buf.map_async(wgpu::MapMode::Read, .., |r| r.unwrap());
		self.device.poll(wgpu::PollType::Wait).unwrap();

		let serialized_data = map_buf.get_mapped_range(..).to_vec();
		Some(RenderedFrame::deserialize(
			self.output_width,
			self.output_height,
			serialized_data,
			frame_hold
		))
	}
}

mod private {
	use super::WgpuRenderer;

	pub trait Sealed {}

	impl<const W: usize, const H: usize> Sealed for WgpuRenderer<W, H> {}
}

pub trait VideoSrc: Iterator<Item = RenderedFrame> + Send + 'static + private::Sealed {
	fn framerate(&self) -> NonZeroU8;
	fn width(&self) -> u32;
	fn height(&self) -> u32;
}

impl<const W: usize, const H: usize> VideoSrc for WgpuRenderer<W, H> {
	#[inline]
	fn framerate(&self) -> NonZeroU8 {
		self.sequence.framerate
	}

	#[inline]
	fn width(&self) -> u32 {
		self.output_width
	}

	#[inline]
	fn height(&self) -> u32 {
		self.output_height
	}
}
