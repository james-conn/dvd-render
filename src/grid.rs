// TODO: add colors
#[derive(Copy, Clone)]
pub struct GridCell {
	character: char
}

impl GridCell {
	#[inline]
	pub fn new(character: char) -> Self {
		Self { character }
	}

	#[inline]
	pub fn space() -> Self {
		Self { character: ' ' }
	}

	#[inline]
	pub fn character(&self) -> char {
		self.character
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
