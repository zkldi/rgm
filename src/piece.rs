use ggez::graphics::Color;
use rand::seq::SliceRandom;
use rand::Rng;
use strum_macros::EnumIter;

const INITIAL_SPAWN_X: i32 = 3;
const INITIAL_SPAWN_Y: i32 = 21;

use crate::{
	board::{PieceBox, Square},
	movement::RotIndex,
};

#[derive(Debug, EnumIter, Clone, PartialEq, Eq, Copy)]
pub enum PieceType {
	Z,
	S,
	T,
	L,
	J,
	O,
	I,
}

const VEC_PIECES: [PieceType; 7] = [
	PieceType::Z,
	PieceType::S,
	PieceType::T,
	PieceType::L,
	PieceType::J,
	PieceType::O,
	PieceType::I,
];

impl PieceType {
	pub fn random<R: Rng>(rng: &mut R) -> Self {
		let piece = VEC_PIECES
			.choose(rng)
			.expect("vec_pieces was empty, no pieces are defined?");

		*piece
	}

	fn get_color(self) -> Color {
		match self {
			PieceType::Z => Color::GREEN,
			PieceType::S => Color::MAGENTA,
			PieceType::T => Color {
				r: 0.0,
				g: 0.5,
				b: 1.0,
				a: 1.0,
			},
			PieceType::L => Color {
				r: 1.0,
				g: 0.5,
				b: 0.5,
				a: 1.0,
			},
			PieceType::J => Color::BLUE,
			PieceType::O => Color::YELLOW,
			PieceType::I => Color::RED,
		}
	}

	pub fn to_piece(self) -> Piece {
		Piece {
			p_type: self,
			rot_idx: RotIndex::Neutral,
			x: INITIAL_SPAWN_X,
			y: INITIAL_SPAWN_Y,
		}
	}
}

#[derive(Debug, Clone, Copy)]
pub struct Piece {
	pub p_type: PieceType,
	pub rot_idx: RotIndex,
	pub x: i32,
	pub y: i32,
}

impl Piece {
	pub fn get_box(self) -> PieceBox {
		let c = self.p_type.get_color();

		macro_rules! rot2 {
			($n: expr, $cw: expr) => {
				match self.rot_idx {
					RotIndex::Neutral | RotIndex::U => $n,
					RotIndex::CW | RotIndex::CCW => $cw,
				}
			};
		}

		let o = Square::Empty;
		let x = Square::Filled(c);

		let b = match self.p_type {
			PieceType::Z => rot2!(
				[
					// .
					[o, o, o, o],
					[x, x, o, o],
					[o, x, x, o],
					[o, o, o, o],
				],
				[
					// .
					[o, o, x, o],
					[o, x, x, o],
					[o, x, o, o],
					[o, o, o, o],
				]
			),
			PieceType::S => rot2!(
				[
					// .
					[o, o, o, o],
					[o, x, x, o],
					[x, x, o, o],
					[o, o, o, o],
				],
				[
					// .
					[x, o, o, o],
					[x, x, o, o],
					[o, x, o, o],
					[o, o, o, o],
				]
			),
			PieceType::T => match self.rot_idx {
				RotIndex::Neutral => [
					// .
					[o, o, o, o],
					[x, x, x, o],
					[o, x, o, o],
					[o, o, o, o],
				],
				RotIndex::CW => [
					// .
					[o, x, o, o],
					[x, x, o, o],
					[o, x, o, o],
					[o, o, o, o],
				],
				RotIndex::U => [
					// .
					[o, o, o, o],
					[o, x, o, o],
					[x, x, x, o],
					[o, o, o, o],
				],
				RotIndex::CCW => [
					// .
					[o, x, o, o],
					[o, x, x, o],
					[o, x, o, o],
					[o, o, o, o],
				],
			},
			PieceType::L => match self.rot_idx {
				RotIndex::Neutral => [
					// .
					[o, o, o, o],
					[x, x, x, o],
					[x, o, o, o],
					[o, o, o, o],
				],
				RotIndex::CW => [
					// .
					[x, x, o, o],
					[o, x, o, o],
					[o, x, o, o],
					[o, o, o, o],
				],

				RotIndex::U => [
					// .
					[o, o, o, o],
					[o, o, x, o],
					[x, x, x, o],
					[o, o, o, o],
				],
				RotIndex::CCW => [
					// .
					[o, x, o, o],
					[o, x, o, o],
					[o, x, x, o],
					[o, o, o, o],
				],
			},
			PieceType::J => match self.rot_idx {
				RotIndex::Neutral => [
					// .
					[o, o, o, o],
					[x, x, x, o],
					[o, o, x, o],
					[o, o, o, o],
				],
				RotIndex::CW => [
					// .
					[o, x, o, o],
					[o, x, o, o],
					[x, x, o, o],
					[o, o, o, o],
				],
				RotIndex::U => [
					// .
					[o, o, o, o],
					[x, o, o, o],
					[x, x, x, o],
					[o, o, o, o],
				],
				RotIndex::CCW => [
					// .
					[o, x, x, o],
					[o, x, o, o],
					[o, x, o, o],
					[o, o, o, o],
				],
			},
			PieceType::O => [
				// .
				[o, o, o, o],
				[o, x, x, o],
				[o, x, x, o],
				[o, o, o, o],
			],
			PieceType::I => match self.rot_idx {
				RotIndex::Neutral | RotIndex::U => [
					// .
					[o, o, o, o],
					[x, x, x, x],
					[o, o, o, o],
					[o, o, o, o],
				],
				RotIndex::CCW | RotIndex::CW => [
					// .
					[o, o, x, o],
					[o, o, x, o],
					[o, o, x, o],
					[o, o, x, o],
				],
			},
		};

		PieceBox { b }
	}
}
