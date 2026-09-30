pub(super) fn cell_coordinate(value: f64) -> i32 {
    let wrapped: f64 = if value > f64::from(i32::MAX - 1) {
        value % f64::from(i32::MAX)
    } else if value < f64::from(i32::MIN + 1) {
        value % f64::from(i32::MIN)
    } else {
        value
    };
    wrapped as i32
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
