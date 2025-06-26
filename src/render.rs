use core::num::NonZeroU8;

pub struct RenderedFrame {
	pub img: image::RgbaImage,
	pub frame_hold: NonZeroU8,
}

mod private {
	pub trait Sealed {}

	#[cfg(feature = "cpu")]
	impl Sealed for crate::cpu_render::CpuRenderer {}

	#[cfg(feature = "gpu")]
	impl Sealed for crate::gpu_render::WgpuRenderer {}
}

pub trait VideoSrc: Iterator<Item = RenderedFrame> + Send + 'static + private::Sealed {
	fn framerate(&self) -> NonZeroU8;
	fn width(&self) -> u32;
	fn height(&self) -> u32;
}
