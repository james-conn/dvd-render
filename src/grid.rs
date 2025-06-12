#[derive(Copy, Clone)]
pub struct GridCell {
	character: char,
	fg_color: image::Rgba<u8>,
	bg_color: image::Rgba<u8>
}

impl GridCell {
	#[inline]
	pub fn new(character: char) -> Self {
		Self {
			character,
			fg_color: image::Rgba([u8::MAX, u8::MAX, u8::MAX, u8::MAX]),
			bg_color: image::Rgba([u8::MIN, u8::MIN, u8::MIN, u8::MAX])
		}
	}

	#[inline]
	pub fn new_fg_color(character: char, fg_color: image::Rgba<u8>) -> Self {
		Self {
			character,
			fg_color,
			bg_color: image::Rgba([u8::MIN, u8::MIN, u8::MIN, u8::MAX])
		}
	}

	#[inline]
	pub fn new_full_color(
		character: char,
		fg_color: image::Rgba<u8>,
		bg_color: image::Rgba<u8>
	) -> Self {
		Self {
			character,
			fg_color,
			bg_color
		}
	}

	#[inline]
	pub fn space() -> Self {
		Self::new(' ')
	}

	#[inline]
	pub fn set_fg_color(&mut self, fg_color: image::Rgba<u8>) {
		self.fg_color = fg_color;
	}

	#[inline]
	pub fn set_bg_color(&mut self, bg_color: image::Rgba<u8>) {
		self.bg_color = bg_color;
	}

	#[inline]
	pub fn character(&self) -> char {
		self.character
	}

	#[inline]
	pub fn fg_color(&self) -> image::Rgba<u8> {
		self.fg_color
	}

	#[inline]
	pub fn bg_color(&self) -> image::Rgba<u8> {
		self.bg_color
	}
}

#[derive(Clone)]
pub struct Grid<const W: usize, const H: usize> {
	cells: [[GridCell; W]; H]
}

impl<const W: usize, const H: usize> Default for Grid<W, H> {
	fn default() -> Self {
		Self {
			cells: [[GridCell::space(); W]; H]
		}
	}
}

impl<const W: usize, const H: usize> Grid<W, H> {
	/// panics if out of bounds
	pub fn set(&mut self, x: usize, y: usize, c: GridCell) {
		self.cells[y][x] = c;
	}

	pub fn get_cell(&self, x: usize, y: usize) -> &GridCell {
		&self.cells[y][x]
	}

	pub fn get_cell_mut(&mut self, x: usize, y: usize) -> &mut GridCell {
		&mut self.cells[y][x]
	}

	pub(crate) fn chars(&self) -> Vec<char> {
		let mut chars = vec![];

		for row in self.cells {
			for cell in row {
				if !chars.contains(&cell.character) {
					chars.push(cell.character);
				}
			}
		}

		chars
	}

	#[inline]
	pub(crate) fn cells(&self) -> &[[GridCell; W]; H] {
		&self.cells
	}
}
