use crate::pile::Pile;
use crate::piece::Piece;
use crate::config;

use delegate::delegate;

mod time_manager;
use time_manager::TimeManager;

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

    // TODO: make a macro that marks methods that should finish with project
    fn project(&mut self) {
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
            (config::LEFT_THRESHOLD + config::RIGHT_THRESHOLD) as i16 / 2 - 2, 0);
    }
    
    pub fn use_spare (&mut self) {
        if !self.spare_used {
            self.spare_used = true;

            self.current_piece.swap_figures(&mut self.spare_piece);
            self.spare_piece.refresh();
            self.put_in_starting_position();
            self.project();
        }
    }

    pub fn finish_turn (&mut self) -> bool {
        if self.collides(&self.current_piece, (0, 0)) {
            return true;
        }

        self.pile.add(&self.current_piece);
        let cleaned_up = self.pile.cleanup_full_lines();

        self.current_piece.swap_figures(&mut self.next_piece);
        self.next_piece.randomize();

        self.put_in_starting_position();
        // try fuzzy fitting when the piece just appears
        let mut temp = self.current_piece.clone();
        if self.try_fuzzy_fit(&mut temp) {
            self.current_piece = temp;
        }

        self.spare_used = false;
        self.score += cleaned_up * config::RIGHT_THRESHOLD;

        if cleaned_up > 1 {
            // for combos
            self.score +=  cleaned_up * cleaned_up;
            self.last_combo = cleaned_up;
        }

        self.time_manager.update_tick_speed(cleaned_up);
        self.time_manager.tick();
        self.project();
        return false;
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

            if real_j >= config::RIGHT_THRESHOLD
                || real_i >= config::BOTTOM_THRESHOLD
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
            self.project();
        }
    }

    pub fn move_right(&mut self) {
        if !self.collides(&self.current_piece, (1, 0)) {
            self.current_piece.move_right_unsafe();
            self.project();
        }
    }

    // each move down is interpreted as a tick of the game
    pub fn move_down (&mut self) {
        if !self.touches_on_bottom(&self.current_piece) {
            self.time_manager.tick();
            self.current_piece.move_down_unsafe();
        }
    }

    pub fn safe_rotate_clockwise(&mut self) {
        self.safe_rotate_internal(true);
    }

    pub fn safe_rotate_counter_clockwise(&mut self) {
        self.safe_rotate_internal(false);
    }

    fn try_fuzzy_fit(&self, piece: &mut Piece) -> bool {
        if !self.collides(&piece, (0, 0)) {
            return true;
        }
        // move 1 to the left from the initial position
        if !self.collides(&piece, (-1, 0)) {
            piece.move_left_unsafe();
            return true;
        }
        // move 1 to the right from the initial position
        if !self.collides(&piece, (1, 0)) {
            piece.move_right_unsafe();
            return true;
        }
        // move 2 to the left from the initial position
        if !self.collides(&piece, (-2, 0)) {
            piece.move_left_unsafe();
            piece.move_left_unsafe();
            return true;
        }
        // move 2 to the right from the initial position
        if !self.collides(&piece, (2, 0)) {
            piece.move_right_unsafe();
            piece.move_right_unsafe();
            return true;
        }
        return false;
    }

    fn safe_rotate_internal (&mut self, clockwise: bool) {
        let mut temp = self.current_piece.clone();
        if clockwise {
            temp.rotate_clockwise();
        } else {
            temp.rotate_counter_clockwise();
        }

        if self.try_fuzzy_fit(&mut temp) {
            self.current_piece = temp;
            self.project();
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
            pile: Pile::new(config::RIGHT_THRESHOLD, config::BOTTOM_THRESHOLD),
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
        tetris.project();
        tetris
    }
}
