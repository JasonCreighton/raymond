use strided;
use strided::{MutStride, Stride};

/// Fixed size two dimensional array
pub struct Array2D<T> {
    pub rows: usize,
    pub columns: usize,
    data: Vec<T>,
}

#[allow(dead_code)]
impl<T: Clone> Array2D<T> {
    pub fn new(rows: usize, columns: usize, fill_element: &T) -> Array2D<T> {
        let data = vec![fill_element.clone(); rows * columns];
        Array2D {
            rows,
            columns,
            data,
        }
    }

    pub fn get(&self, row: usize, column: usize) -> &T {
        debug_assert!(row < self.rows);
        debug_assert!(column < self.columns);

        &self.data[(row * self.columns) + column]
    }

    pub fn set(&mut self, row: usize, column: usize, val: &T) {
        debug_assert!(row < self.rows);
        debug_assert!(column < self.columns);

        self.data[(row * self.columns) + column] = val.clone();
    }

    pub fn slice_within_row(&self, row: usize, column_range: std::ops::Range<usize>) -> &[T] {
        debug_assert!(row < self.rows);
        debug_assert!(column_range.start < self.columns);
        debug_assert!(column_range.end <= self.columns);

        let offset = row * self.columns;
        &self.data[(offset + column_range.start)..(offset + column_range.end)]
    }

    pub fn iter_rows(&self) -> impl Iterator<Item = &[T]> {
        self.data.chunks_exact(self.columns)
    }

    pub fn iter_rows_mut(&mut self) -> impl Iterator<Item = &mut [T]> {
        self.data.chunks_exact_mut(self.columns)
    }

    // TODO: I would prefer not to expose Stride/MutStride here
    pub fn iter_columns(&self) -> impl Iterator<Item = Stride<T>> {
        Stride::new(&self.data).substrides(self.columns)
    }

    pub fn iter_columns_mut(&mut self) -> impl Iterator<Item = MutStride<T>> {
        MutStride::new(&mut self.data).substrides_mut(self.columns)
    }
}
