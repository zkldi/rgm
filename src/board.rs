use std::{array::IntoIter, fmt::Display};

use ggez::graphics::{self, Color, DrawParam};

use crate::piece::Piece;

pub const BOARD_SIZE: (usize, usize) = (10, 21);
pub const SQUARE_SIZE: f32 = 20.0;
pub const BOARD_OFFSET_X: f32 = SQUARE_SIZE * 6.0;
pub const BOARD_OFFSET_Y: f32 = SQUARE_SIZE * 3.0;
const BOARD_BORDER: f32 = 1.0;

#[derive(Debug, Clone, Copy)]
pub enum Square {
	Empty,
	Filled(Color),
}

#[derive(Debug)]
pub struct Board {
	pub state: [[Square; BOARD_SIZE.0]; BOARD_SIZE.1],
}

impl Default for Board {
	fn default() -> Self {
		let state = [[Square::Empty; BOARD_SIZE.0]; BOARD_SIZE.1];

		Board { state }
	}
}

impl Display for Board {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		for row in self.state.iter().rev() {
			for sqr in row {
				if matches!(sqr, Square::Filled(_)) {
					write!(f, "X")?;
				} else {
					write!(f, ".")?;
				}
			}

			write!(f, "\n")?;
		}

		Ok(())
	}
}

impl Board {
	pub fn draw(&self, canvas: &mut graphics::Canvas) {
		let c = Color {
			a: 0.5,
			r: 1.0,
			g: 1.0,
			b: 1.0,
		};

		let width = SQUARE_SIZE * BOARD_SIZE.0 as f32 + BOARD_BORDER;

		let height = SQUARE_SIZE * (BOARD_SIZE.1 - 1) as f32 + BOARD_BORDER;

		canvas.draw(
			&graphics::Quad,
			DrawParam::default()
				.color(c)
				.scale([BOARD_BORDER, height])
				.dest([BOARD_OFFSET_X - BOARD_BORDER, BOARD_OFFSET_Y - BOARD_BORDER]),
		);

		canvas.draw(
			&graphics::Quad,
			DrawParam::default()
				.color(c)
				.scale([BOARD_BORDER, height])
				.dest([
					BOARD_OFFSET_X + BOARD_BORDER + width,
					BOARD_OFFSET_Y + BOARD_BORDER,
				]),
		);

		canvas.draw(
			&graphics::Quad,
			DrawParam::default()
				.color(c)
				.scale([width, BOARD_BORDER])
				.dest([BOARD_OFFSET_X - BOARD_BORDER, BOARD_OFFSET_Y - BOARD_BORDER]),
		);

		canvas.draw(
			&graphics::Quad,
			DrawParam::default()
				.color(c)
				.scale([width, BOARD_BORDER])
				.dest([
					BOARD_OFFSET_X + BOARD_BORDER,
					BOARD_OFFSET_Y + BOARD_BORDER + height,
				]),
		);
	}

	pub fn lock_piece(&mut self, piece: Piece) -> i32 {
		let bx = piece.get_box().b;

		let mut y: i32 = piece.y - 1;

		for row in bx {
			let mut x: i32 = piece.x;

			for sqr in row {
				if matches!(sqr, Square::Filled(_)) {
					self.state[y as usize][x as usize] = sqr;
				}

				x += 1;
			}

			y -= 1;
		}

		self.clear_lines()
	}

	fn clear_lines(&mut self) -> i32 {
		let mut lines_cleared = 0;

		let array_iter = self.state.into_iter();

		let new_rows: Vec<[Square; BOARD_SIZE.0]> = array_iter
			.filter(|row| {
				if row.iter().all(|s| matches!(s, Square::Filled(_))) {
					lines_cleared += 1;
					false
				} else {
					true
				}
			})
			.collect();

		for y in 0..BOARD_SIZE.1 {
			if let Some(r) = new_rows.get(y) {
				self.state[y] = *r;
			} else {
				self.state[y] = [Square::Empty; BOARD_SIZE.0];
			}
		}

		lines_cleared
	}

	pub fn monoify(&mut self, mono_frames: i32) {
		let array_iter = self.state.into_iter();

		let mut y = 0;

		let new_rows: Vec<[Square; BOARD_SIZE.0]> = array_iter
			.map_while(|row| {
				y += 1;

				if y > mono_frames / 10 {
					return None;
				}

				Some(row.map(|s| match s {
					Square::Empty => Square::Empty,
					Square::Filled(_) => Square::Filled(Color::new(1.0, 1.0, 1.0, 0.7)),
				}))
			})
			.collect();

		for y in 0..BOARD_SIZE.1 {
			if let Some(r) = new_rows.get(y) {
				self.state[y] = *r;
			}
		}
	}

	pub fn is_empty(&self) -> bool {
		self.state
			.iter()
			.all(|row| row.iter().all(|s| matches!(s, Square::Empty)))
	}
}

// All pieces play in a 4x4 box.
#[derive(Clone, Copy, Debug)]
pub struct PieceBox {
	pub b: [[Square; 4]; 4],
}

impl PieceBox {
	pub fn draw(self, canvas: &mut graphics::Canvas, x: f32, y: f32) {
		let orig_x = x;

		let mut x = x;
		let mut y = y;

		for row in self.b {
			for sqr in row {
				sqr.draw(canvas, x, y);

				x += SQUARE_SIZE;
			}

			x = orig_x;
			y += SQUARE_SIZE;
		}
	}
}

impl Square {
	pub fn draw(self, canvas: &mut graphics::Canvas, x: f32, y: f32) {
		match self {
			Square::Empty => (),
			Square::Filled(color) => canvas.draw(
				&graphics::Quad,
				DrawParam::default()
					.color(color)
					.scale([SQUARE_SIZE, SQUARE_SIZE])
					.dest([x, y]),
			),
		}
	}
}
