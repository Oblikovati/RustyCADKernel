#[derive(Clone, Default)]
struct TestBnd1D {
    min: f64,
    max: f64,
}
impl TestBnd1D {
    fn new(min: f64, max: f64) -> Self {
        Self { min, max }
    }
}
impl BoundingVolume for TestBnd1D {
    fn add(&mut self, other: &Self) {
        self.min = self.min.min(other.min);
        self.max = self.max.max(other.max);
    }
    fn is_out(&self, other: &Self) -> bool {
        other.min > self.max || other.max < self.min
    }
    fn square_extent(&self) -> f64 {
        let distance: f64 = self.max - self.min;
        distance * distance
    }
}
struct TestSelector1D {
    query: TestBnd1D,
    results: Vec<i32>,
    maximum: usize,
}
impl TestSelector1D {
    fn new(query: TestBnd1D) -> Self {
        Self {
            query,
            results: Vec::new(),
            maximum: usize::MAX,
        }
    }
    fn stopping(query: TestBnd1D, maximum: usize) -> Self {
        Self {
            query,
            results: Vec::new(),
            maximum,
        }
    }
}
impl TreeSelector<i32, TestBnd1D> for TestSelector1D {
    fn reject(&self, bounds: &TestBnd1D) -> bool {
        self.query.is_out(bounds)
    }
    fn accept(&mut self, object: &i32) -> bool {
        self.results.push(*object);
        true
    }
    fn stop(&self) -> bool {
        self.results.len() >= self.maximum
    }
}
