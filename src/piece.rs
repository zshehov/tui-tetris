use crate::matrix::Matrix;
use crate::pile::Pile;
use crate::tetris;
use rand::Rng;

#[derive(Clone)]
pub enum PieceType {
    Square,
    L,
    Straight,
    ReverseL,
    T,
    Worm,
    ReverseWorm
}

#[derive(Clone)]
pub enum PieceColor {
    Red,
    Blue,
    LightBlue,
    Yellow,
    LightYellow,
    Green,
    Magenta,
}

pub fn get_piece_color(piece_type: &PieceType) -> PieceColor {
    match piece_type {
        PieceType::Square => PieceColor::Red,
        PieceType::L => PieceColor::Green,
        PieceType::Straight => PieceColor::LightBlue,
        PieceType::ReverseL => PieceColor::Blue,
        PieceType::T => PieceColor::LightYellow,
        PieceType::Worm => PieceColor::Yellow,
        PieceType::ReverseWorm => PieceColor::Magenta,
    }
}

impl rand::distributions::Distribution<PieceType> for rand::distributions::Standard {
    fn sample<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> PieceType {
        match rng.gen_range(0, 7) {
            0 => PieceType::Square,
            1 => PieceType::L,
            2 => PieceType::Straight,
            3 => PieceType::ReverseL,
            4 => PieceType::T,
            5 => PieceType::Worm,
            _ => PieceType::ReverseWorm,
        }
    }
}


#[derive(Clone)]
pub struct Piece {
    // anchor_x/y can be negative, but combined with a block from the template
    // they always result in a positive usize
    pub anchor_x: i16,
    pub anchor_y: i16,
    pub piece_type: PieceType,
    template: Matrix,
}

impl Piece {
    pub fn rotate_clockwise(&mut self) {
        let mut new_template = self.template.clone();

        for i in 0..self.template.col_count {
            for j in 0..self.template.col_count {
                new_template[(i, j)] =
                    self.template[(self.template.col_count - j - 1, i)];
            }
        }

        self.template = new_template;
    }

    pub fn rotate_counter_clockwise(&mut self) {
        let mut new_template = self.template.clone();

        for i in 0..self.template.col_count {
            for j in 0..self.template.col_count {
                new_template[(i, j)] =
                    self.template[(j, self.template.col_count - i - 1)];
            }
        }

        self.template = new_template;
    }

    fn get_piece_template(piece_type: &PieceType) -> Matrix {
        match piece_type {
            PieceType::Square => Matrix::from_vec(2, SQUARE.to_vec()),
            PieceType::L => Matrix::from_vec(3, L.to_vec()),
            PieceType::ReverseL => Matrix::from_vec(3, REVERSE_L.to_vec()),
            PieceType::Straight => Matrix::from_vec(4, STRAIGHT.to_vec()),
            PieceType::T => Matrix::from_vec(3, T.to_vec()),
            PieceType::Worm => Matrix::from_vec(3, WORM.to_vec()),
            PieceType::ReverseWorm => Matrix::from_vec(3, REVERSE_WORM.to_vec()),
        }
    }

    fn get_random_piece_type() -> PieceType {
        rand::thread_rng().gen()
    }

    pub fn new_random_piece_at(anchor_x: i16, anchor_y: i16) -> Self {
        let piece_type: PieceType = Self::get_random_piece_type();

        Piece {anchor_x, anchor_y,
               template: Self::get_piece_template(&piece_type),
               piece_type}
    }

    pub fn randomize(&mut self) {
        let piece_type: PieceType = Self::get_random_piece_type();
        self.template = Self::get_piece_template(&piece_type);
        self.piece_type = piece_type;
    }

    pub fn swap_figures(&mut self, other: &mut Piece) {
        std::mem::swap(&mut self.template,
                       &mut other.template);

        std::mem::swap(&mut self.piece_type,
                       &mut other.piece_type);
    }

    // returns the piece to its initial orientation
    pub fn refresh(&mut self) {
        self.template = Self::get_piece_template(&self.piece_type);
    }

    pub fn get_positions(&self) -> [(usize, usize); 4] {
        let mut result : [(usize, usize); 4] = [(0, 0); 4];
        let mut x = 0;

        for (i, value) in self.template.backing.iter().enumerate() {
            if *value {
                let (unanchored_i, unanchored_j) = self.template.idx_to_coords(i);
                assert!(unanchored_i as i16 + self.anchor_y >= 0);
                assert!(unanchored_j as i16 + self.anchor_x >= 0);
                result[x] = ((unanchored_i as i16 + self.anchor_y) as usize,
                             (unanchored_j as i16 + self.anchor_x) as usize);
                x += 1;
            }
        }

        return result;
    }
    // positions from this may include off-field coordinates
    fn get_positions_unsafe(&self) -> [(i16, i16); 4] {
        let mut result : [(i16, i16); 4] = [(0, 0); 4];
        let mut x = 0;

        for (i, value) in self.template.backing.iter().enumerate() {
            if *value {
                let (unanchored_i, unanchored_j) = self.template.idx_to_coords(i);
                result[x] = (unanchored_i as i16 + self.anchor_y,
                             unanchored_j as i16 + self.anchor_x);
                x += 1;
            }
        }

        return result;
    }

    pub fn touches_on_bottom (&self, pile: &Pile) -> bool {
        self.collides((0, 1), pile)
    }

    pub fn collides (&self, (offset_x, offset_y): (i16, i16), pile: &Pile) -> bool {
        for (i, j) in self.get_positions_unsafe().iter() {
            let real_i = match *i + offset_y {
                x if x < 0 => return true,
                x => x
            } as usize;

            let real_j = match *j + offset_x {
                x if x < 0 => return true,
                x => x
            } as usize;

            if real_j >= tetris::RIGHT_THRESHOLD
                || real_i >= tetris::BOTTOM_THRESHOLD
                || pile.contains((real_i, real_j)) {
                return true;
            }
        }

        return false;
    }

    // this is for when a check has been done prior to this call
    pub fn move_down_unsafe(&mut self) {
        self.anchor_y += 1;
    }

    // this is for when a check has been done prior to this call
    pub fn move_left_unsafe(&mut self) {
        self.anchor_x -= 1;
    }

    // this is for when a check has been done prior to this call
    pub fn move_right_unsafe(&mut self) {
        self.anchor_x += 1;
    }

    pub fn place_at(&mut self, anchor_x: i16, anchor_y: i16) {
        self.anchor_x = anchor_x;
        self.anchor_y = anchor_y;
    }
}

static SQUARE       : [bool;  4] = [true, true,
                                    true, true];

static L            : [bool;  9] = [false, false, true,
                                    true,  true,  true,
                                    false, false, false];

static REVERSE_L    : [bool;  9] = [true,  false, false,
                                    true,  true,  true,
                                    false, false, false];

static STRAIGHT     : [bool; 16] = [false, false, false, false,
                                    true, true, true, true,
                                    false, false, false, false,
                                    false, false, false, false];

static T            : [bool;  9] = [false, true,  false,
                                    true,  true,  true,
                                    false, false, false];

static WORM         : [bool;  9] = [true,  true,  false,
                                    false, true,  true,
                                    false, false, false];

static REVERSE_WORM : [bool;  9] = [false, true,  true,
                                    true,  true,  false,
                                    false, false, false];
