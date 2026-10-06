//! Array2's move-out, default-construction, publication, then assignment phases.
use super::{Array2, ArrayError, dimension, upper};
mod assignment;
use assignment::{CloneResizeAssignment, MutableResizeAssignment, ResizeAssignment};

#[derive(Clone, Copy)]
struct ResizeRectangle {
    rows: usize,
    cols: usize,
    row_lower: i32,
    col_lower: i32,
}

impl ResizeRectangle {
    fn from_bounds(rl: i32, ru: i32, cl: i32, cu: i32) -> Result<Self, ArrayError> {
        Ok(Self {
            rows: dimension(rl, ru)?,
            cols: dimension(cl, cu)?,
            row_lower: rl,
            col_lower: cl,
        })
    }

    fn from_sizes(rl: i32, cl: i32, rows: usize, cols: usize) -> Result<Self, ArrayError> {
        Self::from_bounds(rl, upper(rl, rows), cl, upper(cl, cols))
    }

    fn length(self) -> usize {
        self.rows.checked_mul(self.cols).unwrap_or_else(|| {
            panic!(
                "array dimensions ({}, {}) exceed the expected usize product",
                self.rows, self.cols
            )
        })
    }
}

impl<T: Default> Array2<'_, T> {
    /// Resizes in packed order with an explicit native move-assignment policy.
    /// `array.resize_with_assignment(3, 4, true, |target, donor| *target = std::mem::take(donor))?`.
    /// Donors stay valid through the callback. Owned donors drop after the assignment phase;
    /// borrowed donors remain in their buffer. An assignment panic retains the new rectangle
    /// with its assigned prefix and initialized remainder.
    pub fn resize_with_assignment(
        &mut self,
        rows: usize,
        cols: usize,
        copy: bool,
        assign: impl FnMut(&mut T, &mut T),
    ) -> Result<(), ArrayError> {
        self.resize_counts_with_assignment(rows, cols, copy, false, MutableResizeAssignment(assign))
    }

    /// Resizes the common subrectangle with an explicit mutable-donor assignment policy.
    /// `array.resize_trim_with_assignment(3, 4, true, |target, donor| *target = *donor)?`.
    /// Assignment panics retain the published destination shape and initialized cells.
    pub fn resize_trim_with_assignment(
        &mut self,
        rows: usize,
        cols: usize,
        copy: bool,
        assign: impl FnMut(&mut T, &mut T),
    ) -> Result<(), ArrayError> {
        self.resize_counts_with_assignment(rows, cols, copy, true, MutableResizeAssignment(assign))
    }

    /// Resizes inclusive bounds in packed order with mutable native-style donors.
    /// `array.resize_bounds_with_assignment(1, 3, 1, 4, true, |target, donor| *target = *donor)?`.
    /// Old storage moves out before defaults; the new bounds publish before assignment.
    #[allow(clippy::too_many_arguments)]
    pub fn resize_bounds_with_assignment(
        &mut self,
        rl: i32,
        ru: i32,
        cl: i32,
        cu: i32,
        copy: bool,
        assign: impl FnMut(&mut T, &mut T),
    ) -> Result<(), ArrayError> {
        self.reshape_with_assignment(
            ResizeRectangle::from_bounds(rl, ru, cl, cu)?,
            copy,
            false,
            MutableResizeAssignment(assign),
        );
        Ok(())
    }

    /// Resizes inclusive bounds preserving row/column positions with mutable donors.
    /// `array.resize_trim_bounds_with_assignment(1, 3, 1, 4, true, |target, donor| *target = *donor)?`.
    /// `copy == false` performs no assignment callback and follows native no-copy resizing.
    #[allow(clippy::too_many_arguments)]
    pub fn resize_trim_bounds_with_assignment(
        &mut self,
        rl: i32,
        ru: i32,
        cl: i32,
        cu: i32,
        copy: bool,
        assign: impl FnMut(&mut T, &mut T),
    ) -> Result<(), ArrayError> {
        self.reshape_with_assignment(
            ResizeRectangle::from_bounds(rl, ru, cl, cu)?,
            copy,
            true,
            MutableResizeAssignment(assign),
        );
        Ok(())
    }

    fn resize_counts_with_assignment(
        &mut self,
        rows: usize,
        cols: usize,
        copy: bool,
        trim: bool,
        assign: impl ResizeAssignment<T>,
    ) -> Result<(), ArrayError> {
        if rows == 0 || cols == 0 {
            self.elements.resize_no_data(0);
            (self.rows, self.cols) = (rows, cols);
            return Ok(());
        }
        let rectangle = ResizeRectangle::from_sizes(self.row_lower, self.col_lower, rows, cols)?;
        self.reshape_with_assignment(rectangle, copy, trim, assign);
        Ok(())
    }

    fn reshape_with_assignment(
        &mut self,
        rectangle: ResizeRectangle,
        copy: bool,
        trim: bool,
        assign: impl ResizeAssignment<T>,
    ) {
        if copy && (rectangle.rows, rectangle.cols) == (self.rows, self.cols) {
            self.rebase(rectangle.row_lower, rectangle.col_lower);
            return;
        }
        if !copy || self.rows == 0 || self.cols == 0 {
            self.reshape_without_assignment(rectangle);
            return;
        }
        // OCCT moves the receiver into a temporary before allocating or running element code.
        let mut previous = std::mem::take(self);
        self.reshape_without_assignment(rectangle);
        previous.assign_resize_cells(self, trim, assign);
    }

    fn reshape_without_assignment(&mut self, rectangle: ResizeRectangle) {
        self.elements.resize_no_data(rectangle.length());
        (self.rows, self.cols) = (rectangle.rows, rectangle.cols);
        self.rebase(rectangle.row_lower, rectangle.col_lower);
    }

    fn assign_resize_cells(
        &mut self,
        destination: &mut Self,
        trim: bool,
        mut assign: impl ResizeAssignment<T>,
    ) {
        let columns = self.cols.min(destination.cols);
        let stride = if trim { self.cols } else { columns };
        for row in 0..self.rows.min(destination.rows) {
            let source = row * stride;
            let target = row * destination.cols;
            for column in 0..columns {
                assign.assign(
                    &mut destination.as_mut_slice()[target + column],
                    self,
                    source + column,
                );
            }
        }
    }
}

impl<T: Default + Clone> Array2<'_, T> {
    #[allow(clippy::too_many_arguments)]
    pub(super) fn resize_clone_bounds(
        &mut self,
        rl: i32,
        ru: i32,
        cl: i32,
        cu: i32,
        copy: bool,
        trim: bool,
    ) -> Result<(), ArrayError> {
        let rectangle = ResizeRectangle::from_bounds(rl, ru, cl, cu)?;
        self.reshape_with_assignment(rectangle, copy, trim, CloneResizeAssignment);
        Ok(())
    }

    pub(super) fn resize_clone_counts(
        &mut self,
        rows: usize,
        cols: usize,
        copy: bool,
        trim: bool,
    ) -> Result<(), ArrayError> {
        self.resize_counts_with_assignment(rows, cols, copy, trim, CloneResizeAssignment)
    }
}
