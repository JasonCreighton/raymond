use std::cell::Cell;
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

thread_local! {
    static PRNG_STATE: Cell<u64> = const { Cell::new(1) };
}

/// Returns a uniformly distributed pseudorandom u32
pub fn rand_u32() -> u32 {
    PRNG_STATE.with(|state| {
        // This is what musl does, hopefully it's not too bad
        let new_state = state
            .get()
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1);
        state.set(new_state);

        (new_state >> 32) as u32
    })
}

/// Returns a uniformly distributed pseudorandom u64
pub fn rand_u64() -> u64 {
    (rand_u32() as u64) | ((rand_u32() as u64) << 32)
}

/// Returns a pseudorandom f32 in the range [0, 1.0) with a uniform
/// distribution.
///
/// Due to a simple implementation, not all f32 bit patterns in that range
/// will be produced, and 1.0 will be generated occasionally.
pub fn rand_f32() -> f32 {
    let scale_factor: f32 = 1.0 / ((1u64 << 63) as f32);

    // x86 (and maybe others) only has efficient int -> float functions for
    // signed ints, so we convert to a non-negative signed int before
    // converting to float.
    (((rand_u64() >> 1) as i64) as f32) * scale_factor
}
