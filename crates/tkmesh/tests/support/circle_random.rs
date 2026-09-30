//! Local glibc TYPE_3 rand state preserves the pinned Linux srand(42) fixture.
use rustycad_tkmath::gp::Xy;
pub struct CircleRandom {
    state: [u32; 31],
    front: usize,
    rear: usize,
}
impl CircleRandom {
    pub fn seeded() -> Self {
        let mut state = [0; 31];
        state[0] = 42;
        for index in 1..31 {
            state[index] = (u64::from(state[index - 1]) * 16807 % 2147483647) as u32;
        }
        let mut generator = Self {
            state,
            front: 3,
            rear: 0,
        };
        for _ in 0..310 {
            generator.next_integer();
        }
        generator
    }
    pub fn next_integer(&mut self) -> u32 {
        let value = self.state[self.front].wrapping_add(self.state[self.rear]);
        self.state[self.front] = value;
        self.front = (self.front + 1) % 31;
        self.rear = (self.rear + 1) % 31;
        value >> 1
    }
    pub fn next_points(&mut self) -> [Xy; 3] {
        std::array::from_fn(|_| {
            // GCC evaluates the two SetCoord arguments right-to-left in the oracle.
            let y = f64::from(self.next_integer()) / 2147483647.;
            let x = f64::from(self.next_integer()) / 2147483647.;
            Xy::new(x, y)
        })
    }
    pub fn accepted_triangle(&mut self) -> [Xy; 3] {
        loop {
            let points = self.next_points();
            if !degenerate(&points) {
                return points;
            }
        }
    }
}
pub fn degenerate(points: &[Xy; 3]) -> bool {
    let first = points[1] - points[0];
    let second = points[2] - points[0];
    let precision = rustycad_tkernel::precision::p_confusion();
    first.square_modulus() <= precision * precision
        || second.square_modulus() <= precision * precision
        || first.crossed(second) <= 5. * std::f64::consts::PI / 180.
}
pub fn on_circle(point: Xy, center: Xy, radius: f64) -> bool {
    let precision = rustycad_tkernel::precision::p_confusion();
    (point - center).square_modulus() - radius * radius < precision * precision
}
