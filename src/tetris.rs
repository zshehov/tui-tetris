use crate::pile::Pile;
use crate::piece::Piece;


pub struct Tetris {
    pub current_piece: Piece,
    pub next_piece: Piece,
    pub spare_piece: Piece,
    pub pile: Pile,

    pub spare_used: bool,
    pub score: usize,
    pub last_combo: usize,
    pub tick_time: i128,
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
            (super::LEFT_THRESHOLD + super::RIGHT_THRESHOLD) as i16 / 2 - 2, 0);
    }
    
    pub fn use_spare (&mut self) {
        if !self.spare_used {
            self.spare_used = true;

            self.current_piece.swap_figures(&mut self.spare_piece);
            self.put_in_starting_position();
        }
    }

    // returns whether the game is over after this turn
    fn finish_turn (&mut self) -> bool {
        self.pile.extend(self.current_piece.get_positions().iter().cloned());
        let cleaned_up = self.pile.cleanup_full_lines();

        self.current_piece.swap_figures(&mut self.next_piece);
        self.next_piece.randomize();

        self.put_in_starting_position();
        if self.current_piece.collides((0,0), &self.pile) {
            return true;
        }
        self.spare_used = false;
        self.score += cleaned_up * super::RIGHT_THRESHOLD;

        if cleaned_up > 1 {
            // for combos
            self.score +=  cleaned_up * cleaned_up;
            self.last_combo = cleaned_up;
        }

        const QUICKENING_COEF: i128 = 20;
        self.tick_time -= QUICKENING_COEF * cleaned_up as i128;

        return false;
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

    pub fn move_down (&mut self) {
        if !self.current_piece.touches_on_bottom(&self.pile) {
            self.current_piece.move_down_unsafe();
        }
    }

    // returns whether the game is over
    pub fn finishing_move_down (&mut self) -> bool {
        if self.current_piece.touches_on_bottom(&self.pile) {
            return self.finish_turn();
        } else {
            self.current_piece.move_down_unsafe();
            return false
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
}
