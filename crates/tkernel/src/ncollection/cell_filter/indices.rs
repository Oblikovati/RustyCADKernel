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
    min: Vec<i32>,
    max: Vec<i32>,
    axes: usize,
    current: Option<Vec<i32>>,
}
impl CellIndices {
    pub(super) fn new(min: Vec<i32>, max: Vec<i32>, axes: usize) -> Self {
        let reversed: bool = (0..axes).any(|axis| min[axis] > max[axis]);
        let current: Option<Vec<i32>> = (!reversed).then(|| min.clone());
        Self {
            min,
            max,
            axes,
            current,
        }
    }
    fn advance(&mut self) {
        let current: &mut Vec<i32> = self.current.as_mut().unwrap();
        let bounds = self.min.iter().zip(&self.max);
        for (value, (lower, upper)) in current.iter_mut().zip(bounds).take(self.axes) {
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
    type Item = Vec<i32>;
    fn next(&mut self) -> Option<Self::Item> {
        let result: Vec<i32> = self.current.clone()?;
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
