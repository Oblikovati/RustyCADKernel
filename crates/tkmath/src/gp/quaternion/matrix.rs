// Note: implementation is based on free samples from
// http://www.gamedev.ru/code/articles/?id=4215&page=3
// and maths found in Wikipedia and elsewhere

use super::Quaternion;
use crate::gp::Matrix3;
impl Quaternion {
    /// Divides by the stored squared norm without first normalizing; `quaternion.matrix()`.
    pub fn matrix(&self) -> Matrix3 {
        let [xx, xy, xz, yy, yz, zz, wx, wy, wz] = self.matrix_products();
        Matrix3::new([
            [1.0 - (yy + zz), xy - wz, xz + wy],
            [xy + wz, 1.0 - (xx + zz), yz - wx],
            [xz - wy, yz + wx, 1.0 - (xx + yy)],
        ])
    }
    fn matrix_products(&self) -> [f64; 9] {
        let (x, y, z, w) = (self.x, self.y, self.z, self.w);
        let scale = 2.0 / self.square_norm();
        let (x2, y2, z2) = (x * scale, y * scale, z * scale);
        [
            x * x2,
            x * y2,
            x * z2,
            y * y2,
            y * z2,
            z * z2,
            w * x2,
            w * y2,
            w * z2,
        ]
    }
    /// Selects the largest-component branch before scaling; `Quaternion::from_matrix(matrix)`.
    pub fn from_matrix(matrix: Matrix3) -> Self {
        let m = *matrix.coefficients();
        let trace = m[0][0] + m[1][1] + m[2][2];
        if trace > 0.0 {
            let raw = Self::new(
                m[2][1] - m[1][2],
                m[0][2] - m[2][0],
                m[1][0] - m[0][1],
                trace + 1.0,
            );
            return raw.scaled(0.5 / raw.w.sqrt());
        }
        let (raw, index) = largest_component(m);
        raw.scaled(0.5 / raw.components()[index].sqrt())
    }
}
fn largest_component(m: [[f64; 3]; 3]) -> (Quaternion, usize) {
    if m[0][0] > m[1][1] && m[0][0] > m[2][2] {
        return (matrix_x(m), 0);
    }
    if m[1][1] > m[2][2] {
        return (matrix_y(m), 1);
    }
    (matrix_z(m), 2)
}
fn matrix_x(m: [[f64; 3]; 3]) -> Quaternion {
    Quaternion::new(
        1.0 + m[0][0] - m[1][1] - m[2][2],
        m[0][1] + m[1][0],
        m[0][2] + m[2][0],
        m[2][1] - m[1][2],
    )
}
fn matrix_y(m: [[f64; 3]; 3]) -> Quaternion {
    Quaternion::new(
        m[0][1] + m[1][0],
        1.0 + m[1][1] - m[0][0] - m[2][2],
        m[1][2] + m[2][1],
        m[0][2] - m[2][0],
    )
}
fn matrix_z(m: [[f64; 3]; 3]) -> Quaternion {
    Quaternion::new(
        m[0][2] + m[2][0],
        m[1][2] + m[2][1],
        1.0 + m[2][2] - m[0][0] - m[1][1],
        m[1][0] - m[0][1],
    )
}
