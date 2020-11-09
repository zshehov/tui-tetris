pub struct Matrix { 
    pub col_count: usize,
    pub backing: Vec<bool>
}

impl std::ops::Index<(usize, usize)> for Matrix {
    type Output = bool;

    fn index(&self, (i, j): (usize, usize)) -> &Self::Output {
        &self.backing[i * self.col_count + j]
    }
}

impl std::ops::IndexMut<(usize, usize)> for Matrix {
    fn index_mut(&mut self, (i, j): (usize, usize)) -> &mut Self::Output {
        &mut self.backing[i * self.col_count + j]
    }
}

impl std::ops::Index<usize> for Matrix {
    type Output = bool;

    fn index(&self, i: usize) -> &Self::Output {
        &self.backing[i]
    }
}

impl std::ops::IndexMut<usize> for Matrix {
    fn index_mut(&mut self, i: usize) -> &mut Self::Output {
        &mut self.backing[i]
    }
}

impl Matrix {
    pub fn get_row(&self, row: usize) -> &[bool] {
        &self.backing[row * self.col_count .. (row + 1) * self.col_count]
    }

    pub fn get_row_mut(&mut self, row: usize) -> &mut [bool] {
        &mut self.backing[row * self.col_count .. (row + 1) * self.col_count]
    }

    pub fn idx_to_coords(&self, idx: usize) -> (usize, usize) {
        (idx / self.col_count, idx % self.col_count)
    }

    pub fn copy_row(&mut self, from: usize, to: usize) {
        let to_idx =  to * self.col_count;
        let from_idx =  from * self.col_count;

        let (left, right) = self.backing.split_at_mut(to_idx);

        right[0..self.col_count].copy_from_slice(&left[from_idx..(from_idx + self.col_count)]);
    }

    pub fn new(col_count: usize, row_count: usize) -> Self {
        Matrix {
            col_count,
            backing: vec![false; col_count * row_count],
        }
    }
}

impl Clone for Matrix {
    fn clone(&self) -> Self {
        Matrix {col_count: self.col_count, backing: self.backing.clone()}
    }
}

