// Rust translation of cacheGeometry and of the local coordinates of solveQuartic and
// solveCubic in ExtremaPC_Hyperbola.hxx and ExtremaPC_Parabola.hxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The cached frame of the 3D hyperbola and parabola evaluators (`cacheGeometry`).
use rustycad_tkmath::gp::{Axis2, Point3};

/// The origin, X, Y and main axis directions of a conic, as plain coordinates.
#[derive(Clone, Copy, Debug)]
pub(crate) struct ConicFrame {
    origin: [f64; 3],
    x_dir: [f64; 3],
    y_dir: [f64; 3],
    axis: [f64; 3],
}

impl ConicFrame {
    /// The frame of a conic position (`Location`, `XAxis`, `YAxis`, `Axis`).
    pub fn of(position: Axis2) -> Self {
        let origin = position.location();
        let (x, y, n) = (
            position.x_direction(),
            position.y_direction(),
            position.direction(),
        );
        Self {
            origin: [origin.x(), origin.y(), origin.z()],
            x_dir: [x.x(), x.y(), x.z()],
            y_dir: [y.x(), y.y(), y.z()],
            axis: [n.x(), n.y(), n.z()],
        }
    }
    /// `origin + a X + b Y`, summed left to right per coordinate as natively.
    pub fn point(&self, a: f64, b: f64) -> Point3 {
        let coordinate = |i: usize| self.origin[i] + a * self.x_dir[i] + b * self.y_dir[i];
        Point3::new(coordinate(0), coordinate(1), coordinate(2))
    }
    /// The X and Y coordinates of `point` projected on the conic plane: the height
    /// along the axis is removed from the point before the origin is subtracted.
    pub fn local_coordinates(&self, point: Point3) -> (f64, f64) {
        let p = [point.x(), point.y(), point.z()];
        let d = [
            p[0] - self.origin[0],
            p[1] - self.origin[1],
            p[2] - self.origin[2],
        ];
        let height: f64 = d[0] * self.axis[0] + d[1] * self.axis[1] + d[2] * self.axis[2];
        let op = |i: usize| (p[i] - height * self.axis[i]) - self.origin[i];
        let (ox, oy, oz) = (op(0), op(1), op(2));
        let x: f64 = ox * self.x_dir[0] + oy * self.x_dir[1] + oz * self.x_dir[2];
        let y: f64 = ox * self.y_dir[0] + oy * self.y_dir[1] + oz * self.y_dir[2];
        (x, y)
    }
}
