//! Segment cases include signed zero, endpoint tolerance and degenerate directions.
use rustycad_tkmath::gp::Xy;
pub fn segment(index: i32) -> (Xy, Xy) {
    let values = [
        [-1.0, -1.0, 1.0, 1.0],
        [-1.0, 1.0, 1.0, -1.0],
        [0.0, 0.0, 2.0, 0.0],
        [1.0, 0.0, 3.0, 0.0],
        [2.0, 0.0, 3.0, 0.0],
        [0.0, 0.0, 0.0, 0.0],
        [0.0, 0.0, 0.0, 1.0],
        [1.0, 0.0, 1.0, 1.0],
        [1e-10, 0.0, 1e-10, 1.0],
        [1e-8, 0.0, 1e-8, 1.0],
        [1.0, 1.0, 2.0, 1.0],
        [1.0, 0.0, 0.0, 0.0],
        [-0.0, -0.0, 1e-200, 1e-200],
        [1e5, 1e-10, -1e5, 1e-10],
        [1e-10, 0.0, 1.0, 1e-10],
    ];
    let [x1, y1, x2, y2] = values[index as usize];
    (Xy::new(x1, y1), Xy::new(x2, y2))
}
