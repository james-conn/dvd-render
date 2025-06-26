use crate::grid::Grid;
use core::num::NonZeroU8;
use std::{
	collections::{HashMap, HashSet, VecDeque},
	num::NonZeroUsize,
};

#[derive(Clone)]
pub struct Frame {
	grid: Grid,
	pub frame_hold: NonZeroU8,
}

impl Frame {
	/// display a grid for a single frame
	pub fn single(grid: Grid) -> Self {
		Self {
			grid,
			frame_hold: NonZeroU8::MIN,
		}
	}

	/// hold on a grid for some amount of frames
	pub fn variable(grid: Grid, frame_hold: NonZeroU8) -> Self {
		Self { grid, frame_hold }
	}

	pub(crate) fn serialize(self, lut: &HashMap<char, u32>) -> Vec<u8> {
		self.grid
			.cells()
			.iter()
			.flat_map(|row| {
				row.iter()
					.map(|cell| {
						*lut.get(&cell.character())
							.expect("invariant upheld by type system")
					})
					.flat_map(u32::to_ne_bytes)
			})
			.collect()
	}

	pub(crate) fn serialize_colors(&self) -> Vec<u8> {
		self.grid
			.cells()
			.iter()
			.flat_map(|row| {
				row.iter()
					.flat_map(|cell| [cell.fg_color().0, cell.bg_color().0])
					.flatten()
			})
			.collect()
	}
}

pub enum FontSize {
	Pixel(f32),
	PixelXY { x: f32, y: f32 },
	Point(f32),
}

pub struct Px(pub f32);

impl From<Px> for FontSize {
	fn from(s: Px) -> FontSize {
		FontSize::Pixel(s.0)
	}
}

impl From<(Px, Px)> for FontSize {
	fn from(xy: (Px, Px)) -> FontSize {
		FontSize::PixelXY {
			x: xy.0.0,
			y: xy.1.0,
		}
	}
}

impl From<[Px; 2]> for FontSize {
	fn from(xy: [Px; 2]) -> FontSize {
		FontSize::PixelXY {
			x: xy[0].0,
			y: xy[1].0,
		}
	}
}

pub struct Pt(pub f32);

impl From<Pt> for FontSize {
	fn from(pt: Pt) -> FontSize {
		FontSize::Point(pt.0)
	}
}

pub struct GridSequence {
	pub framerate: NonZeroU8,
	frames: VecDeque<Frame>,
	pub font_scale: FontSize,
	glyph_set: HashSet<char>,
	width: NonZeroUsize,
	height: NonZeroUsize,
}

impl GridSequence {
	pub fn new(width: NonZeroUsize, height: NonZeroUsize, s: impl Into<FontSize>) -> Self {
		Self {
			framerate: NonZeroU8::MIN,
			frames: VecDeque::new(),
			font_scale: s.into(),
			glyph_set: HashSet::new(),
			width,
			height,
		}
	}

	/// Width first, height second
	pub fn get_dimensions(&self) -> (NonZeroUsize, NonZeroUsize) {
		(self.width, self.height)
	}

	/// push a frame to the beginning of the sequence
	pub fn prepend(&mut self, frame: Frame) {
		for c in frame.grid.chars() {
			self.glyph_set.insert(c);
		}

		self.frames.push_front(frame);
	}

	/// push a frame to the end of the sequence
	pub fn append(&mut self, frame: Frame) {
		for c in frame.grid.chars() {
			self.glyph_set.insert(c);
		}

		self.frames.push_back(frame);
	}

	#[inline]
	pub fn glyph_set(&self) -> &HashSet<char> {
		&self.glyph_set
	}

	#[inline]
	pub(crate) fn pop(&mut self) -> Option<Frame> {
		self.frames.pop_front()
	}

	pub(crate) fn resolve_px_scale<F: ab_glyph::Font>(&self, font: F) -> ab_glyph::PxScale {
		match self.font_scale {
			FontSize::Pixel(s) => ab_glyph::PxScale::from(s),
			FontSize::PixelXY { x, y } => ab_glyph::PxScale { x, y },
			FontSize::Point(pt) => font
				.pt_to_px_scale(pt)
				.expect("not sure why this would fail?"),
		}
	}
}
