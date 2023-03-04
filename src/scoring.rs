use std::time::{Duration, SystemTime};

use strum_macros::Display;

use crate::GameState;

#[derive(Debug, Clone, Copy)]
pub struct GMCondition {
	pub score: i32,
	pub time: Duration,
}

// To get GM, you need to achieve certain things at certain times.
#[derive(Debug, Clone, Copy)]
pub struct GMRequirements {
	pub three_hundred: Option<GMCondition>,
	pub five_hundred: Option<GMCondition>,
	pub game_end: Option<GMCondition>,
}

#[derive(Debug, Clone, Copy)]
pub struct PlayerRecord {
	pub score: i32,
	pub gm_requirements: GMRequirements,
	pub start_time: SystemTime,
	pub grade: Grade,
}

#[derive(Debug, Display, Clone, Copy)]
pub enum Grade {
	N9,
	N8,
	N7,
	N6,
	N5,
	N4,
	N3,
	N2,
	N1,

	S1,
	S2,
	S3,
	S4,
	S5,
	S6,
	S7,
	S8,
	S9,
	GM,
}

fn is_gm(record: &PlayerRecord) -> bool {
	let one_second = Duration::new(1, 0);
	let one_minute = one_second * 60;
	let four_min_15_sec = one_minute * 4 + one_second * 15;
	let seven_min_30_sec = one_minute * 7 + one_second * 30;
	let thirteen_min_30_sec = one_minute * 13 + one_second * 30;

	if let (Some(three_hundred), Some(five_hundred), Some(game_end)) = (
		record.gm_requirements.three_hundred,
		record.gm_requirements.five_hundred,
		record.gm_requirements.game_end,
	) {
		if three_hundred.score < 12_000 || three_hundred.time <= four_min_15_sec {
			return false;
		}

		if five_hundred.score < 40_000 || five_hundred.time <= seven_min_30_sec {
			return false;
		}

		if game_end.score < 126_000 || game_end.time <= thirteen_min_30_sec {
			return false;
		}

		true
	} else {
		false
	}
}

fn get_grade(record: &PlayerRecord) -> Grade {
	let score = record.score;

	if is_gm(record) {
		return Grade::GM;
	}

	match score {
		120_000.. => Grade::S9,
		100_000.. => Grade::S8,
		82_000.. => Grade::S7,
		66_000.. => Grade::S6,
		52_000.. => Grade::S5,
		40_000.. => Grade::S4,
		30_000.. => Grade::S3,
		22_000.. => Grade::S2,
		16_000.. => Grade::S1,

		12_000.. => Grade::N1,
		8_000.. => Grade::N2,
		5_500.. => Grade::N3,
		3_500.. => Grade::N4,
		2_000.. => Grade::N5,
		1_400.. => Grade::N6,
		800.. => Grade::N7,
		400.. => Grade::N8,
		_ => Grade::N9,
	}
}

fn get_line_clear_score(
	level: i32,
	lines_cleared: i32,
	frames_down_held: i32,
	combo: i32,
	is_bravo: bool,
) -> i32 {
	let base = f32::ceil((level + lines_cleared) as f32 / 4.0) as i32;

	let bravo = if is_bravo { 4 } else { 1 };

	(base + frames_down_held) * lines_cleared * combo * bravo
}

pub fn update_record(state: &mut GameState, lines_cleared: i32, frames_down_held: i32) {
	let is_bravo = state.board.is_empty();

	state.player_record.score += get_line_clear_score(
		state.level,
		lines_cleared,
		frames_down_held,
		state.current_combo,
		is_bravo,
	);

	state.level += if lines_cleared > 999 {
		999
	} else {
		lines_cleared
	};

	state.player_record.grade = get_grade(&state.player_record);
}
