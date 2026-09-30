// Rust translation of the frame code shared by ModelingData/TKG2d/Geom2dGridEval/*.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Conic frames of the 2D evaluators, pre-extracted with the native operation order.
//! The parameter loops live in `grid_loops`, shared with TKG3d.
pub(super) use super::grid_loops::{
    column, constant_grid, cyclic_coefficients, empty, map_grid, result_array, try_map_grid,
};
use rustycad_tkmath::gp::{Direction2, Point2, Vector2};

/// Center and axis coordinates that the conic evaluators pre-extract
/// (`aCX, aCY, aXX, aXY, aYX, aYY`).
#[derive(Clone, Copy)]
pub(super) struct GridFrame {
    cx: f64,
    cy: f64,
    xx: f64,
    xy: f64,
    yx: f64,
    yy: f64,
}

impl GridFrame {
    /// Frame of a center and its X and Y directions.
    pub(super) fn new(center: Point2, x_dir: Direction2, y_dir: Direction2) -> Self {
        Self {
            cx: center.x(),
            cy: center.y(),
            xx: x_dir.x(),
            xy: x_dir.y(),
            yx: y_dir.x(),
            yy: y_dir.y(),
        }
    }
    /// `a * X + b * Y`, evaluated as `(a * aXX + b * aYX, a * aXY + b * aYY)`.
    pub(super) fn combined(&self, a: f64, b: f64) -> Vector2 {
        Vector2::new(a * self.xx + b * self.yx, a * self.xy + b * self.yy)
    }
    /// `a * X`, as `(a * aXX, a * aXY)`.
    pub(super) fn scaled_x(&self, a: f64) -> Vector2 {
        Vector2::new(a * self.xx, a * self.xy)
    }
    /// `C + a * X + b * Y`, evaluated left to right as `aCX + a * aXX + b * aYX`.
    pub(super) fn offset_point(&self, a: f64, b: f64) -> Point2 {
        Point2::new(
            self.cx + a * self.xx + b * self.yx,
            self.cy + a * self.xy + b * self.yy,
        )
    }
    /// `C + v`, for the circle, whose formula scales the whole combination by `R`.
    pub(super) fn translated(&self, v: Vector2) -> Point2 {
        Point2::new(self.cx + v.x(), self.cy + v.y())
    }
}

/// A frame with major and minor radii: the ellipse and hyperbola terms
/// `C + MajR * a * X + MinR * b * Y` and `MajR * a * X + MinR * b * Y`.
#[derive(Clone, Copy)]
pub(super) struct RadiiFrame {
    pub(super) frame: GridFrame,
    pub(super) major: f64,
    pub(super) minor: f64,
}

impl RadiiFrame {
    /// `aCX + aMajR * a * aXX + aMinR * b * aYX` (and Y).
    pub(super) fn point(&self, a: f64, b: f64) -> Point2 {
        self.frame.offset_point(self.major * a, self.minor * b)
    }
    /// `aMajR * a * aXX + aMinR * b * aYX` (and Y); a negated coefficient gives the
    /// same bits as the native `-aMajR * sinU` spelling.
    pub(super) fn vector(&self, a: f64, b: f64) -> Vector2 {
        self.frame.combined(self.major * a, self.minor * b)
    }
}
