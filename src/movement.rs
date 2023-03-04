// without the bother of all of the meaning

use ggez::{winit::event::VirtualKeyCode, Context};

use crate::{
	board::{Board, Square, BOARD_OFFSET_X, BOARD_OFFSET_Y, BOARD_SIZE},
	gravity::{get_gravity, grav_to_rpf},
	piece::{Piece, PieceType},
	DAS_FRAMES,
};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Direction {
	Down,
	Up,
	Right,
	Left,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Rotation {
	CW,
	CCW,
	CCW2,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum RotIndex {
	// default
	Neutral,
	// counter cw (A press)
	CCW,
	// upside down
	U,
	// clockwise (B press)
	CW,
}

impl RotIndex {
	fn rotate(self, rot: Rotation) -> Self {
		match rot {
			Rotation::CW => match self {
				RotIndex::Neutral => RotIndex::CW,
				RotIndex::CW => RotIndex::U,
				RotIndex::U => RotIndex::CCW,
				RotIndex::CCW => RotIndex::Neutral,
			},
			Rotation::CCW | Rotation::CCW2 => match self {
				RotIndex::Neutral => RotIndex::CCW,
				RotIndex::CCW => RotIndex::U,
				RotIndex::U => RotIndex::CW,
				RotIndex::CW => RotIndex::Neutral,
			},
		}
	}
}

#[derive(Default, Debug, PartialEq, Eq, Clone, Copy)]
pub struct Movement {
	pub dir: Option<Direction>,
	pub rot: Option<Rotation>,
}

pub fn parse_movement(ctx: &mut Context) -> Movement {
	let kb = &ctx.keyboard;

	let dir: Option<Direction> = if kb.is_key_pressed(VirtualKeyCode::W) {
		Some(Direction::Up)
	} else if kb.is_key_pressed(VirtualKeyCode::A) {
		Some(Direction::Left)
	} else if kb.is_key_pressed(VirtualKeyCode::D) {
		Some(Direction::Right)
	} else if kb.is_key_pressed(VirtualKeyCode::S) {
		Some(Direction::Down)
	} else {
		None
	};

	let rot: Option<Rotation> = if kb.is_key_pressed(VirtualKeyCode::H) {
		Some(Rotation::CCW)
	} else if kb.is_key_pressed(VirtualKeyCode::J) {
		Some(Rotation::CW)
	} else if kb.is_key_pressed(VirtualKeyCode::K) {
		Some(Rotation::CCW2)
	} else {
		None
	};

	Movement { dir, rot }
}

pub fn is_movement_legal(piece: &Piece, board: &Board) -> bool {
	let bx = piece.get_box().b;

	// first row that contains any cells
	let first_y = bx
		.iter()
		.position(|row| {
			if row.iter().any(|s| matches!(s, Square::Filled(_))) {
				return true;
			}

			false
		})
		.expect("unexpected empty piece? dying.") as i32;

	let last_y = bx
		.iter()
		.rposition(|row| {
			if row.iter().any(|s| matches!(s, Square::Filled(_))) {
				return true;
			}

			false
		})
		.expect("unexpected empty piece? dying.") as i32;

	// first x that contains any cells
	let first_x = {
		let mut min: i32 = 4;

		for row in bx.iter() {
			let mut x = 0;

			for sqr in row {
				if let Square::Filled(_) = sqr {
					break;
				}
				x += 1;
			}

			if x < min {
				min = x;
			}
		}

		min
	};

	// largest x that contains any cells
	let last_x = {
		let mut max: i32 = 0;

		for row in bx.iter() {
			let mut m: i32 = 0;

			for (x, sqr) in row.iter().enumerate() {
				if let Square::Filled(_) = sqr {
					m = x as i32;
				}
			}

			if m > max {
				max = m;
			}
		}

		max
	};

	let min_x = first_x + piece.x;
	let max_x = last_x + piece.x;

	// x and y stuff is inverted and i don't care to fix it
	let min_y = piece.y - last_y;
	let max_y = piece.y - first_y;

	// going out of x bounds
	if min_x < 0 || max_x >= BOARD_SIZE.0 as i32 {
		return false;
	}

	// going out of y bounds
	if min_y <= 0 || max_y > BOARD_SIZE.1 as i32 {
		return false;
	}

	let mut y = piece.y - 1;

	// colliding with block?
	for row in bx {
		let mut x = piece.x;

		if let Some(b_row) = board.state.get(y as usize) {
			for sqr in row {
				// if this square is filled and the board is filled too
				// we have a conflict, this move is not legal.
				if let (Square::Filled(_), Some(Square::Filled(_))) = (sqr, b_row.get(x as usize)) {
					return false;
				}

				x += 1;
			}
		};

		y -= 1;
	}

	true
}

fn apply_rot_kicks(orig_state: Piece, piece: &Piece, board: &Board) -> Piece {
	let mut next_state = piece.to_owned();

	if !is_movement_legal(&next_state, board) {
		// kick handling

		// the I piece has no kicks in this game
		if piece.p_type == PieceType::I {
			// deny this rotation
			return orig_state;
		} else {
			// kick 1 right
			next_state.x += 1;

			if !is_movement_legal(&next_state, board) {
				// kick 1 left (+1 to compensate for prev change)
				next_state.x -= 2;

				if !is_movement_legal(&next_state, board) {
					// not legal
					return orig_state;
				}
			}
		}
	}

	next_state
}

// moves the piece down according to current gravity
pub fn fall(piece: Piece, board: &Board, level: i32, grav_frames: i32) -> Option<Piece> {
	let (rows, frames) = grav_to_rpf(get_gravity(level));

	let mut next_state = piece;

	if grav_frames >= frames {
		// should move down x rows

		for _ in 1..=rows {
			next_state.y -= 1;

			if !is_movement_legal(&next_state, board) {
				next_state.y += 1;

				break;
			}
		}

		// tried to move down, but was not legal. this piece is on the floor.
		if piece.y == next_state.y {
			return None;
		}
	}

	// we need lookahead to know if we're on the floor or not
	let mut lookahead = piece;
	lookahead.y -= 1;

	if !is_movement_legal(&lookahead, board) {
		return None;
	}

	Some(next_state)
}

pub fn apply_movement(
	previous_movement: &Movement,
	movement: &Movement,
	das_frames: i32,
	piece: Piece,
	board: &Board,
) -> Piece {
	let Movement { mut dir, mut rot } = movement;

	// don't allow repeated rotations, ever
	if previous_movement.rot == rot {
		rot = None;
	}

	// only allow repeated dir moves if post-DAS
	// repeated down is always legal though
	if previous_movement.dir == Some(Direction::Down) && movement.dir == Some(Direction::Down)
	// force das to 20hz instead of 60hz
		|| (das_frames >= DAS_FRAMES && (das_frames - DAS_FRAMES % 3 == 0))
	{
	} else if das_frames < DAS_FRAMES && previous_movement.dir == dir {
		dir = None;
	}

	let mut next_state = piece;

	// if rotating, apply rotate
	if let Some(r) = rot {
		next_state.rot_idx = piece.rot_idx.rotate(r.to_owned());
	}

	// if moving, apply movement
	if let Some(d) = dir {
		match d {
			Direction::Down => next_state.y -= 1,
			Direction::Right => next_state.x += 1,
			Direction::Left => next_state.x -= 1,
			Direction::Up => (),
		}
	}

	if !is_movement_legal(&next_state, board) {
		next_state.x = piece.x;
		next_state.y = piece.y;
	}

	// if rotating, apply rotate kicks
	if rot.is_some() {
		// might revert the rotation, might not. tries to kick out of illegal scenarios.
		next_state = apply_rot_kicks(piece, &next_state, board);

		if !is_movement_legal(&next_state, board) {
			next_state.x = piece.x;
			next_state.y = piece.y;
		}
	}

	next_state
}
