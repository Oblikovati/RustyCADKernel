use super::{Array1, checked_array_extent};
use crate::ncollection::ArrayError;

impl<'a, T> Array1<'a, T> {
    /// Borrows a requested prefix without owning or initializing its elements.
    ///
    /// ```
    /// use rustycad_tkernel::ncollection::Array1;
    /// let mut values = [3, 4, 99];
    /// let array = Array1::from_slice_size(&mut values, 2).unwrap();
    /// assert_eq!(array.as_slice(), [3, 4]);
    /// ```
    pub fn from_slice_size(buffer: &'a mut [T], size: usize) -> Result<Self, ArrayError> {
        let actual = buffer.len();
        let prefix = buffer
            .get_mut(..size)
            .ok_or(ArrayError::DimensionMismatch {
                expected: size,
                actual,
            })?;
        Ok(Self::from_slice(prefix))
    }

    /// Borrows inclusive native bounds while retaining direct mutable buffer storage.
    ///
    /// ```
    /// use rustycad_tkernel::ncollection::Array1;
    /// let mut values = [3, 4, 99];
    /// let mut array = Array1::from_slice_bounds(&mut values, -1, 0).unwrap();
    /// array[-1] = 7;
    /// assert_eq!(array.as_slice(), [7, 4]);
    /// ```
    pub fn from_slice_bounds(
        buffer: &'a mut [T],
        lower: i32,
        upper: i32,
    ) -> Result<Self, ArrayError> {
        let mut array = Self::from_slice_size(buffer, checked_array_extent(lower, upper)?)?;
        array.lower = lower;
        Ok(array)
    }
}

impl<'a, T: Default> Array1<'a, T> {
    /// Borrows a requested prefix or default-constructs an independent array.
    /// When `use_buffer` is false the input length and values are ignored.
    ///
    /// ```
    /// use rustycad_tkernel::ncollection::Array1;
    /// let mut values = [99];
    /// let array = Array1::from_buffer_size(&mut values, 3, false).unwrap();
    /// assert_eq!(array.len(), 3);
    /// assert!(array.is_deletable());
    /// ```
    pub fn from_buffer_size(
        buffer: &'a mut [T],
        size: usize,
        use_buffer: bool,
    ) -> Result<Self, ArrayError> {
        if !use_buffer {
            return Ok(Self::with_size(size));
        }
        Self::from_slice_size(buffer, size)
    }

    /// Selects native bounded buffer reuse or independent default construction.
    /// When `use_buffer` is false the supplied buffer is neither read nor copied.
    ///
    /// ```
    /// use rustycad_tkernel::ncollection::Array1;
    /// let mut values = [99];
    /// let array = Array1::from_buffer_bounds(&mut values, -1, 1, false).unwrap();
    /// assert_eq!((array.lower(), array.upper()), (-1, 1));
    /// assert!(array.is_deletable());
    /// ```
    pub fn from_buffer_bounds(
        buffer: &'a mut [T],
        lower: i32,
        upper: i32,
        use_buffer: bool,
    ) -> Result<Self, ArrayError> {
        if !use_buffer {
            return Self::with_bounds(lower, upper);
        }
        Self::from_slice_bounds(buffer, lower, upper)
    }
}
