use crate::pile::Pile;
use crate::piece::Piece;

use delegate::delegate;

mod time_manager;
use time_manager::TimeManager;

pub const END_PLAYING_SCREEN_X : usize = 74;
pub const END_SCREEN_Y : usize = 54;

pub const BLOCK_HEIGHT : usize = 2;
pub const BLOCK_WIDTH : usize = BLOCK_HEIGHT * 2;
pub const LEFT_THRESHOLD : usize = 0;
pub const RIGHT_THRESHOLD : usize = END_PLAYING_SCREEN_X / BLOCK_WIDTH;
pub const BOTTOM_THRESHOLD : usize = END_SCREEN_Y / BLOCK_HEIGHT;
const INITIAL_TICK_TIME_MS : usize = 1000;

pub struct Tetris {
    pub current_piece: Piece,
    pub next_piece: Piece,
    pub spare_piece: Piece,
    pub projected_piece: Piece,
    pub pile: Pile,

    pub spare_used: bool,
    pub score: usize,
    pub last_combo: usize,
    
    time_manager: TimeManager,
}

impl Tetris {
    pub fn drop_to_bottom (&mut self) {
        while self.can_move_down() {
            self.current_piece.move_down_unsafe();
        }
        self.finish_turn();
    }

    pub fn project(&mut self) {
        self.projected_piece = self.current_piece.clone();
        while !self.touches_on_bottom(&self.projected_piece) {
            self.projected_piece.move_down_unsafe();
        }
    }

    pub fn can_move_down(&self) -> bool {
        !self.touches_on_bottom(&self.current_piece)
    }

    fn put_in_starting_position(&mut self) {
        self.current_piece.place_at(
            (LEFT_THRESHOLD + RIGHT_THRESHOLD) as i16 / 2 - 2, 0);
    }
    
    pub fn use_spare (&mut self) {
        if !self.spare_used {
            self.spare_used = true;

            self.current_piece.swap_figures(&mut self.spare_piece);
            self.spare_piece.refresh();
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
        self.collides(&self.current_piece, (0,0))
    }

    fn collides(&self, piece: &Piece, (offset_x, offset_y): (i16, i16)) -> bool {
        for (i, j) in piece.get_positions_unsafe().iter() {
            let real_i = match *i + offset_y {
                x if x < 0 => return true,
                x => x
            } as usize;

            let real_j = match *j + offset_x {
                x if x < 0 => return true,
                x => x
            } as usize;

            if real_j >= RIGHT_THRESHOLD
                || real_i >= BOTTOM_THRESHOLD
                || self.pile.contains((real_i, real_j)) {
                return true;
            }
        }

        return false;
    }

    fn touches_on_bottom(&self, piece: &Piece) -> bool {
        self.collides(piece, (0, 1))
    }

    pub fn move_left(&mut self) {
        if !self.collides(&self.current_piece, (-1, 0)) {
            self.current_piece.move_left_unsafe();
        }
    }

    pub fn move_right(&mut self) {
        if !self.collides(&self.current_piece, (1, 0)) {
            self.current_piece.move_right_unsafe();
        }
    }

    // each move down is interpreted as a tick of the game
    pub fn move_down (&mut self) {
        if !self.touches_on_bottom(&self.current_piece) {
            self.time_manager.tick();
            self.current_piece.move_down_unsafe();
        }
    }

    pub fn finishing_move_down (&mut self) {
        if self.touches_on_bottom(&self.current_piece) {
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

        if !self.collides(&temp, (0, 0)) {
            self.current_piece = temp;
        } else
        // move 1 to the left from the initial position
        if !self.collides(&temp, (-1, 0)) {
            temp.move_left_unsafe();
            self.current_piece = temp;
        } else
        // move 1 to the right from the initial position
        if !self.collides(&temp, (1, 0)) {
            temp.move_right_unsafe();
            self.current_piece = temp;
        } else
        // move 2 to the left from the initial position
        if !self.collides(&temp, (-2, 0)) {
            temp.move_left_unsafe();
            temp.move_left_unsafe();
            self.current_piece = temp;
        } else
        // move 2 to the right from the initial position
        if !self.collides(&temp, (22, 0)) {
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
            projected_piece: Piece::new_random_piece_at(0, 0),
            spare_used: false,
            score: 0,
            last_combo: 0,
            time_manager: TimeManager::new()
        };
        tetris.put_in_starting_position();
        tetris
    }
}
