use crate::pile::Pile;
use crate::piece::Piece;

use delegate::delegate;

mod time_manager;
use time_manager::TimeManager;

pub const END_PLAYING_SCREEN_X : usize = 74;
pub const END_SCREEN_Y : usize = 54;

pub const BLOCK_HEIGHT : usize = 3;
pub const BLOCK_WIDTH : usize = BLOCK_HEIGHT * 2;
pub const LEFT_THRESHOLD : usize = 0;
pub const RIGHT_THRESHOLD : usize = END_PLAYING_SCREEN_X / BLOCK_WIDTH;
pub const BOTTOM_THRESHOLD : usize = END_SCREEN_Y / BLOCK_HEIGHT;
const INITIAL_TICK_TIME_MS : usize = 1000;

pub struct Tetris {
    pub current_piece: Piece,
    pub next_piece: Piece,
    pub spare_piece: Piece,
    pub pile: Pile,

    pub spare_used: bool,
    pub score: usize,
    pub last_combo: usize,
    
    time_manager: TimeManager,
}

impl Tetris {
    pub fn drop_to_bottom (&mut self) {
        while !self.current_piece.touches_on_bottom(&self.pile) {
            self.current_piece.move_down_unsafe();
        }
        self.finish_turn();
    }

    pub fn can_move_down(&self) -> bool {
        return !self.current_piece.touches_on_bottom(&self.pile);
    }

    fn put_in_starting_position(&mut self) {
        self.current_piece.place_at(
            (LEFT_THRESHOLD + RIGHT_THRESHOLD) as i16 / 2 - 2, 0);
    }
    
    pub fn use_spare (&mut self) {
        if !self.spare_used {
            self.spare_used = true;

            self.current_piece.swap_figures(&mut self.spare_piece);
            self.put_in_starting_position();
        }
    }

    pub fn finish_turn (&mut self) {
        self.pile.add(&self.current_piece);
        let cleaned_up = self.pile.cleanup_full_lines();

        self.current_piece.swap_figures(&mut self.next_piece);
        self.next_piece.randomize();

        self.put_in_starting_position();

        self.spare_used = false;
        self.score += cleaned_up * RIGHT_THRESHOLD;

        if cleaned_up > 1 {
            // for combos
            self.score +=  cleaned_up * cleaned_up;
            self.last_combo = cleaned_up;
        }

        self.time_manager.update_tick_speed(cleaned_up);
        self.time_manager.tick();
    }

    pub fn is_over (&self) -> bool {
        self.current_piece.collides((0,0), &self.pile)
    }

    pub fn move_left(&mut self) {
        if !self.current_piece.collides((-1, 0), &self.pile) {
            self.current_piece.move_left_unsafe();
        }
    }

    pub fn move_right(&mut self) {
        if !self.current_piece.collides((1, 0), &self.pile) {
            self.current_piece.move_right_unsafe();
        }
    }

    // each move down is interpreted as a tick of the game
    pub fn move_down (&mut self) {
        if !self.current_piece.touches_on_bottom(&self.pile) {
            self.time_manager.tick();
            self.current_piece.move_down_unsafe();
        }
    }

    pub fn finishing_move_down (&mut self) {
        if self.current_piece.touches_on_bottom(&self.pile) {
            self.finish_turn();
        } else {
            self.current_piece.move_down_unsafe();
        }
    }

    pub fn safe_rotate_clockwise(&mut self) {
        self.safe_rotate_internal(true);
    }

    pub fn safe_rotate_counter_clockwise(&mut self) {
        self.safe_rotate_internal(false);
    }

    fn safe_rotate_internal (&mut self, clockwise: bool) {
        let mut temp = self.current_piece.clone();
        if clockwise {
            temp.rotate_clockwise();
        } else {
            temp.rotate_counter_clockwise();
        }

        if !temp.collides((0, 0), &self.pile) {
            self.current_piece = temp;
        } else
        // move 1 to the left from the initial position
        if !temp.collides((-1, 0), &self.pile) {
            temp.move_left_unsafe();
            self.current_piece = temp;
        } else
        // move 1 to the right from the initial position
        if !temp.collides((1, 0), &self.pile) {
            temp.move_right_unsafe();
            self.current_piece = temp;
        } else
        // move 2 to the left from the initial position
        if !temp.collides((-2, 0), &self.pile) {
            temp.move_left_unsafe();
            temp.move_left_unsafe();
            self.current_piece = temp;
        } else
        // move 2 to the right from the initial position
        if !temp.collides((2, 0), &self.pile) {
            temp.move_right_unsafe();
            temp.move_right_unsafe();
            self.current_piece = temp;
        }
    }

    pub fn get_tick_speed(&self) -> usize {
        self.time_manager.tick_time
    }

    delegate! {
        to self.time_manager {
            pub fn get_timeout(&self) -> usize;
            pub fn should_finish_turn(&self) -> bool;
            pub fn advance_stuck(&mut self);
        }
    }

    pub fn new() -> Self {
        let mut tetris = Tetris {
            pile: Pile::new(RIGHT_THRESHOLD, BOTTOM_THRESHOLD),
            current_piece: Piece::new_random_piece_at(0, 0),
            next_piece: Piece::new_random_piece_at(0, 1),
            spare_piece: Piece::new_random_piece_at(0, 7),
            spare_used: false,
            score: 0,
            last_combo: 0,
            time_manager: TimeManager::new()
        };
        tetris.put_in_starting_position();
        tetris
    }
}
