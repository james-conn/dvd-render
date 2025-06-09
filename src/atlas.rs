use ab_glyph::{Font, ScaleFont};
use wgpu::COPY_BUFFER_ALIGNMENT as ALIGN;
use std::collections::{HashMap, HashSet};
use crate::sequence::GridSequence;

pub(crate) struct Atlas {
	pub buffer: Vec<u8>,
	pub lut: HashMap<char, u32>,
	pub font_width: u32,
	pub font_height: u32
}

// partially aesthetic, partially a `wgpu` hack for buffer alignment
fn round_up_aligned(n: u32) -> u32 {
	(ALIGN as u32 * (n / ALIGN as u32)) + ALIGN as u32
}

// upper bound of size for the biggest glyph
fn font_size<F: Font, SF: ScaleFont<F>>(font: &SF, glyph_set: &HashSet<char>) -> (u32, u32) {
	let mut font_width = f32::MIN;
	let mut font_height = f32::MIN;

	for glyph_char in glyph_set {
		let glyph = font.scaled_glyph(*glyph_char);
		if let Some(outline) = font.outline_glyph(glyph) {
			font_width = font_width.max(outline.px_bounds().width());
			font_height = font_height.max(outline.px_bounds().height() - font.descent());
		}
	}

	assert!(font_width != f32::MIN && font_height != f32::MIN, "font has no glyphs");

	(round_up_aligned(font_width as u32), round_up_aligned(font_height as u32))
}

pub(crate) fn populate_atlas<const W: usize, const H: usize, F: Font>(font: F, sequence: &GridSequence<W, H>) -> Atlas {
	let font = font.as_scaled(sequence.resolve_px_scale(&font));

	let (font_width, font_height) = font_size(&font, sequence.glyph_set());

	let mut atlas_img = image::GrayImage::new(font_width, sequence.glyph_set().len() as u32 * font_height);
	let mut cursor_y = 0;
	let mut lut = HashMap::new();

	for (i, glyph_char) in sequence.glyph_set().iter().enumerate() {
		let glyph = font.scaled_glyph(*glyph_char);
		let baseline_offset = -font.descent() as u32;
		let Some(outline) = font.outline_glyph(glyph) else {
			// if no outline is present just skip drawing
			lut.insert(*glyph_char, i as u32);
			cursor_y += font_height;
			continue;
		};

		lut.insert(*glyph_char, i as u32);

		let px_bounds = outline.px_bounds();
		let glyph_width = px_bounds.width() as u32;

		// annoying math for figuring out the proper y offset so that letters
		// which descend below the baseline (like 'g' or 'p') get rendered properly
		let baseline = font_height - baseline_offset;
		let glyph_gap_to_baseline = baseline.checked_sub(px_bounds.height() as u32)
			.expect("either your font is wrong or my math is wrong (which is very possible)");
		let glyph_below_baseline = px_bounds.max.y as u32;
		let baseline_diff = glyph_gap_to_baseline + glyph_below_baseline;

		outline.draw(|x, y, c| {
			let luma = (c * u8::MAX as f32) as u8;
			atlas_img.put_pixel(
				x + ((font_width - glyph_width) / 2),
				cursor_y + y + baseline_diff,
				image::Luma([luma])
			);
		});

		cursor_y += font_height;
	}

	Atlas {
		buffer: atlas_img.into_vec(),
		lut,
		font_width,
		font_height
	}
}
