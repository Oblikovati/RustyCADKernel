use super::CellIndex;
pub(super) fn cell_coordinate(value: f64) -> i32 {
    let wrapped: f64 = if value > f64::from(i32::MAX - 1) {
        value % f64::from(i32::MAX)
    } else if value < f64::from(i32::MIN + 1) {
        value % f64::from(i32::MIN)
    } else {
        value
    };
    // OCCT 3d097a03: preserve the pinned x86 cast after native modulus wrapping.
    crate::ncollection::ElementCast::<i32>::cast(wrapped)
}
/// Inclusive cell range over the first `axes` axes, axis zero fastest; later axes keep
/// the minimum corner's values, as the native recursion never touches them.
pub(super) struct CellIndices {
    min: CellIndex,
    max: CellIndex,
    axes: usize,
    current: Option<CellIndex>,
}
impl CellIndices {
    pub(super) fn new(min: CellIndex, max: CellIndex, axes: usize) -> Self {
        let reversed: bool = (0..axes).any(|axis| min.as_slice()[axis] > max.as_slice()[axis]);
        let current: Option<CellIndex> = (!reversed).then(|| min.clone());
        Self {
            min,
            max,
            axes,
            current,
        }
    }
    fn advance(&mut self) {
        let current: &mut CellIndex = self.current.as_mut().unwrap();
        let bounds = self.min.as_slice().iter().zip(self.max.as_slice());
        for (value, (lower, upper)) in current
            .as_mut_slice()
            .iter_mut()
            .zip(bounds)
            .take(self.axes)
        {
            if *value < *upper {
                *value += 1;
                return;
            }
            *value = *lower;
        }
        self.current = None;
    }
}
impl Iterator for CellIndices {
    type Item = CellIndex;
    fn next(&mut self) -> Option<Self::Item> {
        let result: CellIndex = self.current.clone()?;
        self.advance();
        Some(result)
    }
}

#[cfg(test)]
mod conversion_tests {
    use super::cell_coordinate;
    #[test]
    fn native_nonfinite_cell_indices_do_not_alias_origin() {
        for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            assert_eq!(cell_coordinate(value), i32::MIN);
        }
        assert_eq!(cell_coordinate(0.), 0);
        assert_eq!(cell_coordinate(-1.9), -1);
    }
    #[test]
    fn finite_extremes_keep_native_modulus_before_truncation() {
        assert_eq!(cell_coordinate(f64::from(i32::MAX) + 3.), 3);
        assert_eq!(cell_coordinate(f64::from(i32::MIN) - 3.), -3);
    }
}
