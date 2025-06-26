use crate::sequence::GridSequence;

/*
pub trait VideoSrc: Iterator<Item = RenderedFrame> + Send + 'static + private::Sealed {
	fn framerate(&self) -> NonZeroU8;
	fn width(&self) -> u32;
	fn height(&self) -> u32;
}
*/

pub struct CpuRenderer {
	sequence: GridSequence
}
