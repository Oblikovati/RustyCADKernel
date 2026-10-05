//! Safe borrowed views matching the flat column-major storage accepted by Mat3/Mat4 `Map`.

use super::{Mat3, Mat4};

/// Borrows one column-major matrix without copying; create one with `Mat3::map(&values)`.
pub struct MatrixView<'a, T, const N: usize> {
    elements: &'a [T],
}

/// Mutably borrows one column-major matrix without copying; create one with `Mat4::map_mut(&mut values)`.
pub struct MatrixViewMut<'a, T, const N: usize> {
    elements: &'a mut [T],
}

impl<'a, T, const N: usize> MatrixView<'a, T, N> {
    fn from_elements(elements: &'a [T]) -> Self {
        Self { elements }
    }

    /// Returns the borrowed flat storage in native column-major order, as `view.as_slice()`.
    pub fn as_slice(&self) -> &'a [T] {
        self.elements
    }

    /// Borrows an element by row and column, as `view.element(1, 2)`.
    pub fn element(&self, row: usize, column: usize) -> &T {
        &self.elements[flat_index::<N>(row, column)]
    }

    /// Reads a copied value like native `GetValue`, as `view.get_value(1, 2)`.
    pub fn get_value(&self, row: usize, column: usize) -> T
    where
        T: Copy,
    {
        *self.element(row, column)
    }
}

impl<'a, T, const N: usize> MatrixViewMut<'a, T, N> {
    fn from_elements(elements: &'a mut [T]) -> Self {
        Self { elements }
    }

    /// Returns an immutable view of the same storage, as `view.as_view().get_value(0, 0)`.
    pub fn as_view(&self) -> MatrixView<'_, T, N> {
        MatrixView::from_elements(self.elements)
    }

    /// Borrows the backing storage in native column-major order, as `view.as_slice()`.
    pub fn as_slice(&self) -> &[T] {
        self.elements
    }

    /// Borrows the backing storage mutably, as `view.as_mut_slice()[0] = value`.
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        self.elements
    }

    /// Borrows an element by row and column, as `view.element(1, 2)`.
    pub fn element(&self, row: usize, column: usize) -> &T {
        &self.elements[flat_index::<N>(row, column)]
    }

    /// Reads a copied value like native `GetValue`, as `view.get_value(1, 2)`.
    pub fn get_value(&self, row: usize, column: usize) -> T
    where
        T: Copy,
    {
        *self.element(row, column)
    }

    /// Writes through the view into its backing array, as `view.set_value(1, 2, value)`.
    pub fn set_value(&mut self, row: usize, column: usize, value: T) {
        self.elements[flat_index::<N>(row, column)] = value;
    }
}

const fn flat_index<const N: usize>(row: usize, column: usize) -> usize {
    column * N + row
}

impl<T> Mat3<T> {
    /// Borrows a nine-element column-major buffer, as native `Map(const Element_t*)` does.
    /// Example: `let view = Mat3::map(&values); assert_eq!(view.get_value(1, 2), values[7]);`.
    pub fn map(elements: &[T; 9]) -> MatrixView<'_, T, 3> {
        MatrixView::from_elements(elements)
    }

    /// Mutably borrows a nine-element buffer, as native `Map(Element_t*)` does.
    /// Example: `Mat3::map_mut(&mut values).set_value(1, 2, 4.0);` updates `values[7]`.
    pub fn map_mut(elements: &mut [T; 9]) -> MatrixViewMut<'_, T, 3> {
        MatrixViewMut::from_elements(elements)
    }
}

impl<T> Mat4<T> {
    /// Borrows a sixteen-element column-major buffer, as native `Map(const Element_t*)` does.
    /// Example: `let view = Mat4::map(&values); assert_eq!(view.get_value(1, 2), values[9]);`.
    pub fn map(elements: &[T; 16]) -> MatrixView<'_, T, 4> {
        MatrixView::from_elements(elements)
    }

    /// Mutably borrows a sixteen-element buffer, as native `Map(Element_t*)` does.
    /// Example: `Mat4::map_mut(&mut values).set_value(1, 2, 4.0);` updates `values[9]`.
    pub fn map_mut(elements: &mut [T; 16]) -> MatrixViewMut<'_, T, 4> {
        MatrixViewMut::from_elements(elements)
    }
}
