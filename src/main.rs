mod board;
mod gravity;
mod movement;
mod piece;
mod scoring;

use board::{Board, Square, BOARD_OFFSET_X, BOARD_OFFSET_Y, BOARD_SIZE, SQUARE_SIZE};
use core::panic;
use ggez::conf::{Conf, WindowSetup};
use ggez::event::{self, EventHandler};
use ggez::glam::Vec2;
use ggez::graphics::Text;
use ggez::{
	graphics::{self, Color},
	Context, ContextBuilder, GameResult,
};
use movement::{
	apply_movement, fall, is_movement_legal, parse_movement, Direction, Movement, Rotation,
};
use piece::{Piece, PieceType};
use rand::rngs::ThreadRng;
use rand::Rng;
use scoring::{update_record, GMRequirements, Grade, PlayerRecord};
use std::{collections::VecDeque, time::SystemTime};

const FPS: u32 = 60;
const ARE_FRAMES: i32 = 30;
const DAS_FRAMES: i32 = 16;
const LOCK_DELAY_FRAMES: i32 = 30;
const LINE_CLEAR_FRAMES: i32 = 41;

// a piece is active and the player is moving it around
#[derive(Debug, Clone, Copy)]
struct ActiveState {
	piece: Piece,
	lock_frames: i32,
	das_frames: i32,
	down_frames: i32,
	gravity_frames: i32,
}

// a piece has just been locked
#[derive(Debug, Clone, Copy)]
struct WaitingState {
	waiting_frames: i32,
	das_frames: i32,
	did_clear_line: bool,
}

// a piece has just been locked
#[derive(Debug, Clone, Copy)]
struct GameOverState {
	mono_frames: i32,
}

#[derive(Debug, Clone, Copy)]
enum State {
	Active(ActiveState),
	Waiting(WaitingState),
	GameOver(GameOverState),
}

#[derive(Debug)]
pub struct GameState {
	state: State,
	last_seen: VecDeque<PieceType>,
	next_piece: PieceType,

	rng: ThreadRng,
	level: i32,
	current_combo: i32,

	player_record: PlayerRecord,
	run_start: SystemTime,

	board: Board,
	movement: Movement,
}

const TRIES: i32 = 4;

fn draw_new_piece<R: Rng>(level: i32, rng: &mut R, last_seen: &VecDeque<PieceType>) -> PieceType {
	let mut new_piece: Option<PieceType> = None;

	// try X times to generate a piece we haven't seen before.
	for _ in 0..TRIES {
		let piece = PieceType::random(rng);
		new_piece = Some(piece);

		if !last_seen.contains(&piece) {
			break;
		}
	}

	let new_piece =
		new_piece.expect("The constant TRIES was 0. No piece could ever be generated like this.");

	// if this is the first piece we drew this game
	// dont let it be S/Z/O.
	// We do this recursively instead of a loop, that's probably fine. Right?
	if level == 0
		&& (new_piece == PieceType::S || new_piece == PieceType::Z || new_piece == PieceType::O)
	{
		return draw_new_piece(level, rng, last_seen);
	}

	new_piece
}

impl GameState {
	fn iter_piece(&mut self) -> Piece {
		let current_next_piece = self.next_piece;

		// add new element
		self.last_seen.push_back(current_next_piece);
		self.last_seen.pop_front();

		self.next_piece = draw_new_piece(self.level, &mut self.rng, &self.last_seen);

		// increase level if not at level stop (99, or 998)
		if self.level != 998 && self.level % 100 != 99 {
			self.level += 1;
		}

		let piece = current_next_piece.to_piece();

		let rot = self.movement.rot;

		// apply IRS if any
		apply_movement(
			&Movement::default(),
			&Movement { rot, dir: None },
			0,
			piece,
			&self.board,
		)
	}
}

impl Default for GameState {
	fn default() -> GameState {
		let level = 500;
		let mut rng = rand::thread_rng();
		let last_seen = VecDeque::from([PieceType::Z, PieceType::Z, PieceType::Z, PieceType::Z]);

		GameState {
			state: State::Waiting(WaitingState {
				waiting_frames: 0,
				did_clear_line: false,
				das_frames: 0,
			}),
			next_piece: draw_new_piece(level, &mut rng, &last_seen),
			last_seen,
			rng,
			level,
			current_combo: 0,
			run_start: SystemTime::now(),
			player_record: PlayerRecord {
				score: 0,
				gm_requirements: GMRequirements {
					three_hundred: None,
					five_hundred: None,
					game_end: None,
				},
				start_time: SystemTime::now(),
				grade: Grade::N9,
			},
			board: Board::default(),
			movement: Movement::default(),
		}
	}
}

impl GameState {
	pub fn new(_ctx: &mut Context) -> GameState {
		GameState::default()
	}
}

impl EventHandler for GameState {
	fn update(&mut self, ctx: &mut Context) -> GameResult {
		while ctx.time.check_update_time(FPS) {
			let previous_movement = self.movement;

			self.movement = parse_movement(ctx);

			self.state = match self.state {
				State::Active(mut st) => {
					// if this piece spawned in illegal, you're dead
					if !is_movement_legal(&st.piece, &self.board) {
						State::GameOver(GameOverState { mono_frames: 0 })
					} else {
						// initial fall: todo investigate why

						if let Some(next_st) =
							fall(st.piece, &self.board, self.level, st.gravity_frames as i32)
						{
							st.piece = next_st;
						}

						// move piece
						st.piece = apply_movement(
							&previous_movement,
							&self.movement,
							st.das_frames,
							st.piece,
							&self.board,
						);

						match fall(st.piece, &self.board, self.level, st.gravity_frames as i32) {
							Some(next_st) => {
								// piece is not on the floor
								st.lock_frames = 0;

								// this piece either didn't need to fall, or fell successfully.
								if next_st.y == st.piece.y {
									// didn't need to fall
									// frames since last gravity application increases
									st.gravity_frames += 1;
								} else {
									// frames since last grav application resets
									st.gravity_frames = 1;
								}

								st.piece = next_st;
							}
							None => {
								// piece is on the floor
								st.lock_frames += 1;
							}
						}

						match self.movement.dir {
							// down increases down frames
							Some(Direction::Down) => st.down_frames += 1,
							// repeated holds in the same direction update DAS
							Some(Direction::Left) | Some(Direction::Right) => {
								if previous_movement.dir == self.movement.dir {
									st.das_frames += 1;
								} else {
									st.das_frames = 0;
								}
							}
							_ => st.das_frames = 0,
						}

						if (st.lock_frames >= LOCK_DELAY_FRAMES)
							|| (st.lock_frames > 0 && self.movement.dir == Some(Direction::Down))
						{
							let lines = self.board.lock_piece(st.piece);

							if lines > 0 {
								self.current_combo += 1;
							} else {
								self.current_combo = 0;
							}

							update_record(self, lines, st.down_frames);

							// piece needs to lock
							State::Waiting(WaitingState {
								waiting_frames: 0,
								das_frames: 0,
								// cba
								did_clear_line: false,
							})
						} else {
							State::Active(st)
						}
					}
				}

				State::Waiting(mut st) => {
					match self.movement.dir {
						// repeated holds in the same direction update DAS
						Some(Direction::Left) | Some(Direction::Right) => {
							if previous_movement.dir == self.movement.dir {
								st.das_frames += 1;
							} else {
								st.das_frames = 0;
							}
						}
						_ => st.das_frames = 0,
					}

					if (st.did_clear_line && st.waiting_frames >= LINE_CLEAR_FRAMES)
						|| st.waiting_frames >= ARE_FRAMES
					{
						// go into playable state

						let mut piece = self.iter_piece();

						// always perform a fall on the first frame.
						if let Some(n_piece) = fall(piece, &self.board, self.level, 1) {
							piece = n_piece;
						}

						State::Active(ActiveState {
							piece,
							lock_frames: 0,
							das_frames: st.das_frames,
							down_frames: 0,
							gravity_frames: 0,
						})
					} else {
						st.waiting_frames += 1;

						State::Waiting(st)
					}
				}

				State::GameOver(mut st) => {
					st.mono_frames += 1;

					self.board.monoify(st.mono_frames);

					State::GameOver(st)
				}
			}
		}

		Ok(())
	}

	fn draw(&mut self, ctx: &mut Context) -> GameResult {
		let mut canvas = graphics::Canvas::from_frame(ctx, Color::BLACK);

		self.board.draw(&mut canvas);

		// next box
		let next_box = self.next_piece.to_piece().get_box();

		next_box.draw(
			&mut canvas,
			BOARD_OFFSET_X + SQUARE_SIZE * (BOARD_SIZE.0 as f32 / 3.0),
			BOARD_OFFSET_Y - SQUARE_SIZE * 3.25,
		);

		// board state
		let mut x: f32 = BOARD_OFFSET_X;
		let mut y: f32 = BOARD_OFFSET_Y + SQUARE_SIZE * (BOARD_SIZE.1 - 2) as f32;
		for row in self.board.state {
			for square in row {
				square.draw(&mut canvas, x, y);

				x += SQUARE_SIZE;
			}

			x = BOARD_OFFSET_X;
			y -= SQUARE_SIZE;
		}

		canvas.draw(
			&Text::new(format!("level {}", self.level)),
			Vec2::new(400., 400.),
		);
		canvas.draw(
			&Text::new(format!("grade {}", self.player_record.grade)),
			Vec2::new(400., 200.),
		);

		canvas.draw(
			&Text::new(format!("score\n{}", self.player_record.score)),
			Vec2::new(400., 300.),
		);

		if let State::Active(a) = self.state {
			canvas.draw(
				&Text::new(format!("{:#?}", LOCK_DELAY_FRAMES - a.lock_frames)),
				Vec2::new(400., 100.),
			);

			// current piece
			a.piece.get_box().draw(
				&mut canvas,
				BOARD_OFFSET_X + SQUARE_SIZE * a.piece.x as f32,
				BOARD_OFFSET_Y + SQUARE_SIZE * (BOARD_SIZE.1 as i32 - 1 - a.piece.y) as f32,
			);
		}

		canvas.finish(ctx)
	}
}

fn main() {
	unsafe { backtrace_on_stack_overflow::enable() };

	let (mut ctx, event_loop) = ContextBuilder::new("RGM", "zkldi")
		.default_conf(Conf {
			window_setup: WindowSetup {
				title: "RGM".to_string(),
				..Default::default()
			},
			..Default::default()
		})
		.build()
		.expect("for what reason can this even fail, cya lol");

	// Create an instance of your event handler.
	// Usually, you should provide it with the Context object to
	// use when setting your game up.
	let state = GameState::new(&mut ctx);

	// Run!
	event::run(ctx, event_loop, state);
}
