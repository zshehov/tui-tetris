use crate::piece;
use crate::matrix::Matrix;
use std::collections::{HashMap, HashSet};

// TODO: making matrix iterable by lines will remove this
use crate::tetris;


pub struct Pile {
    // for easier collision detection
    pub field: Matrix,
    // for easier block rendering
    pub map: HashMap<(usize, usize), piece::PieceColor>
}

impl Pile {
    pub fn contains(&self, coords: (usize, usize)) -> bool {
        self.field[coords]
    }

    pub fn add<I: IntoIterator<Item = (usize, usize)>>(&mut self, iter: I,
                                                       color: piece::PieceColor) {
        for coords in iter {
            self.field[coords] = true;
            self.map.insert(coords, color.clone());
        }
    }

    fn is_complete_line_with(&self, line: usize, additional: &HashSet<(usize, usize)>) -> bool {
        for (idx, value) in self.field.get_row(line).iter().enumerate() {
            if !*value && !additional.contains(&(line, idx)){
                return false;
            }
        }
        return true;
    }

    pub fn get_complete_lines_with(&self, positions: &[(usize, usize); 4]) -> Vec<usize> {
        let positions_set :HashSet<(usize, usize)> = positions.iter().cloned().collect();
        let mut result : Vec<usize> = Vec::new();

        for (i, _) in positions {
            if self.is_complete_line_with(*i, &positions_set) {
                result.push(*i);
            }
        }

        return result;
    }

    fn copy_line(&mut self, from: usize, to: usize) {
        self.field.copy_row(from, to);

        for (idx, value) in self.field.get_row(to).iter().enumerate() {
            if *value {
                self.map.insert((to, idx),
                    // just a random color for the unwrap fail as it can never happen
                    self.map.get(&(from, idx)).cloned().unwrap_or(piece::PieceColor::Green));
            }
        }
    }

    pub fn cleanup_full_lines(&mut self) -> usize {
        let mut current : usize = tetris::BOTTOM_THRESHOLD as usize - 1;
        let mut cleaned_up = 0;

        //TODO: Make matrix iterable line by line
        for line in (1..tetris::BOTTOM_THRESHOLD as usize).rev() {
            if self.is_complete_line_with(line, &HashSet::new()) {
                self.remove_line(line);
                cleaned_up += 1;
                continue;
            } else {
                if line != current {
                    self.copy_line(line, current);
                    self.remove_line(line);
                }
                current -= 1;
            }
        }

        for line in 0..(current) {
            self.remove_line(line);
        }
        return cleaned_up;
    }

    fn remove_line(&mut self, line: usize) {
        for (idx, value) in self.field.get_row_mut(line).iter_mut().enumerate() {
            *value = false;
            self.map.remove(&(line, idx));
        }
    }

    pub fn new(col_count: usize, row_count: usize) -> Self {
        Pile {
            field: Matrix::new(col_count, row_count),
            map: HashMap::new()
        }
    }
}
