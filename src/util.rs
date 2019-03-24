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
    /// Construct a "rows" by "columns" two dimensional array, filled with "fill_element"
    pub fn new(rows: usize, columns: usize, fill_element: &T) -> Array2D<T> {
        let data = vec![fill_element.clone(); rows * columns];
        Array2D {
            rows,
            columns,
            data,
        }
    }

    /// Get a reference to the element at the given (zero-indexed) row and column
    pub fn get(&self, row: usize, column: usize) -> &T {
        debug_assert!(row < self.rows);
        debug_assert!(column < self.columns);

        &self.data[(row * self.columns) + column]
    }

    /// Set the element at the given (zero-indxed) row and column to a clone of the passed "val"
    pub fn set(&mut self, row: usize, column: usize, val: &T) {
        debug_assert!(row < self.rows);
        debug_assert!(column < self.columns);

        self.data[(row * self.columns) + column] = val.clone();
    }

    pub fn iter_rows(&self) -> impl Iterator<Item = &[T]> {
        self.data.chunks_exact(self.columns)
    }

    pub fn iter_rows_mut(&mut self) -> impl Iterator<Item = &mut [T]> {
        self.data.chunks_exact_mut(self.columns)
    }

    // TODO: I would prefer not to expose Stride/MutStride here, it would be nice
    // if the iter_rows() and iter_columns() functions had the same type signature
    pub fn iter_columns(&self) -> impl Iterator<Item = Stride<T>> {
        Stride::new(&self.data).substrides(self.columns)
    }

    pub fn iter_columns_mut(&mut self) -> impl Iterator<Item = MutStride<T>> {
        MutStride::new(&mut self.data).substrides_mut(self.columns)
    }
}
