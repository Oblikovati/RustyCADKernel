use rustycad_tkernel::ncollection::ArrayError;
// The 64-element inline alternative intentionally avoids a heap allocation, as OCCT does.
#[allow(clippy::large_enum_variant)]
pub(super) enum TableStorage<'a> {
    Inline([f64; 64]),
    Owned(Vec<f64>),
    Borrowed(&'a mut [f64]),
}
impl TableStorage<'_> {
    pub(super) fn new(length: usize, inline_allowed: bool) -> Self {
        if inline_allowed && length <= 64 {
            Self::Inline([0.0; 64])
        } else {
            Self::Owned(vec![0.0; length])
        }
    }
    pub(super) fn slice(&self) -> &[f64] {
        match self {
            Self::Inline(values) => values,
            Self::Owned(values) => values,
            Self::Borrowed(values) => values,
        }
    }
    pub(super) fn slice_mut(&mut self) -> &mut [f64] {
        match self {
            Self::Inline(values) => values,
            Self::Owned(values) => values,
            Self::Borrowed(values) => values,
        }
    }
}
#[derive(Clone, Copy)]
pub(super) struct TableShape {
    pub(super) lower: [i32; 2],
    pub(super) size: [usize; 2],
}
impl TableShape {
    pub(super) fn new(lower: [i32; 2], size: [usize; 2]) -> Self {
        size[0]
            .checked_mul(size[1])
            .expect("table dimensions must fit usize");
        Self { lower, size }
    }
    pub(super) fn with_bounds(rl: i32, ru: i32, cl: i32, cu: i32) -> Result<Self, ArrayError> {
        Ok(Self::new(
            [rl, cl],
            [dimension(rl, ru)?, dimension(cl, cu)?],
        ))
    }
    pub(super) fn len(self) -> usize {
        self.size[0] * self.size[1]
    }
    pub(super) fn dimensions(self) -> (usize, usize) {
        (self.size[0], self.size[1])
    }
    pub(super) fn upper(self, axis: usize) -> i32 {
        (i64::from(self.lower[axis]) + self.size[axis] as i64 - 1) as i32
    }
    pub(super) fn indices(self, row: i32, col: i32) -> Result<[usize; 2], ArrayError> {
        Ok([
            bounded(row, self.lower[0], self.upper(0))?,
            bounded(col, self.lower[1], self.upper(1))?,
        ])
    }
    pub(super) fn offset(self, row: usize, col: usize) -> Result<usize, ArrayError> {
        for (index, length) in [(row, self.size[0]), (col, self.size[1])] {
            if index >= length {
                return Err(ArrayError::Index {
                    index: index as i64,
                    lower: 0,
                    upper: length as i64 - 1,
                });
            }
        }
        Ok(row * self.size[1] + col)
    }
}
// Native math_DoubleTab accepts the empty range `[lower, lower - 1]` (RangeSize 0, a
// table whose loops never run). A range below it has a negative RangeSize that native
// code sign-extends into a huge `size_t` over its 64-value stack buffer, so the first
// Init or read runs past the buffer (undefined behaviour, AppCont at degree -2 crashes);
// the port refuses it with Standard_RangeError instead, as NCollection_Array2 does.
fn dimension(lower: i32, upper: i32) -> Result<usize, ArrayError> {
    let size: i64 = i64::from(upper) - i64::from(lower) + 1;
    if size < 0 {
        return Err(ArrayError::Bounds { lower, upper });
    }
    Ok(size as usize)
}
fn bounded(index: i32, lower: i32, upper: i32) -> Result<usize, ArrayError> {
    if index < lower || index > upper {
        return Err(ArrayError::Index {
            index: index.into(),
            lower: lower.into(),
            upper: upper.into(),
        });
    }
    Ok((i64::from(index) - i64::from(lower)) as usize)
}
