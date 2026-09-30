// Rust translation of FoundationClasses/TKMath/Bnd/Bnd_Box2d.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

use super::super::text_dump::{self, DumpSide};
use super::BoundingBox2;
use crate::gp::{Direction2, Point2, Transform2};
use std::io;

/// Gap-adjusted limits in the native `Bnd_Box2d::Limits` field order (Xmin, Xmax, Ymin, Ymax).
///
/// Open sides read ±1e100. Example: `let BoxLimits2 { x_min, x_max, .. } = bounds.limits();`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BoxLimits2 {
    /// Minimum X coordinate.
    pub x_min: f64,
    /// Maximum X coordinate.
    pub x_max: f64,
    /// Minimum Y coordinate.
    pub y_min: f64,
    /// Maximum Y coordinate.
    pub y_max: f64,
}

impl BoundingBox2 {
    /// Unchecked structured `Get()`, including void-box sentinel values; `bounds.limits().x_max`.
    pub fn limits(&self) -> BoxLimits2 {
        BoxLimits2 {
            x_min: self.x_min(),
            x_max: self.x_max(),
            y_min: self.y_min(),
            y_max: self.y_max(),
        }
    }
    /// `GetXMin`: -1e100 when open, else `Xmin - Gap`; `bounds.x_min()`.
    pub fn x_min(&self) -> f64 {
        self.bound(0, false)
    }
    /// `GetXMax`: 1e100 when open, else `Xmax + Gap`; `bounds.x_max()`.
    pub fn x_max(&self) -> f64 {
        self.bound(0, true)
    }
    /// `GetYMin`: -1e100 when open, else `Ymin - Gap`; `bounds.y_min()`.
    pub fn y_min(&self) -> f64 {
        self.bound(1, false)
    }
    /// `GetYMax`: 1e100 when open, else `Ymax + Gap`; `bounds.y_max()`.
    pub fn y_max(&self) -> f64 {
        self.bound(1, true)
    }
    /// `Update(X,Y)`: takes the point on a void box, otherwise widens closed sides only;
    /// `bounds.update_point(1.0, 2.0)`.
    pub fn update_point(&mut self, x: f64, y: f64) {
        self.update_bounds([x, y], [x, y]);
    }
    /// `Set(P,D)`: clears flags and gap, then bounds the half-line; `bounds.set_ray(point, direction)`.
    pub fn set_ray(&mut self, point: Point2, direction: Direction2) {
        self.flags = 1;
        self.gap = 0.0;
        self.add_ray(point, direction);
    }
    /// `Add(P,D)`: adds the point, then opens the sides the direction points to;
    /// `bounds.add_ray(point, direction)`.
    pub fn add_ray(&mut self, point: Point2, direction: Direction2) {
        self.add_point(point);
        self.add_direction(direction);
    }
    /// `IsOut(Other,T)`: tests against the transformed other box; `bounds.is_out_transformed(&other, transform)`.
    pub fn is_out_transformed(&self, other: &Self, transform: Transform2) -> bool {
        self.is_out_box(&other.transformed(transform))
    }
    /// `IsOut(T1,Other,T2)`: transforms both boxes before testing; `bounds.is_out_both(first, &other, second)`.
    pub fn is_out_both(&self, first: Transform2, other: &Self, second: Transform2) -> bool {
        self.transformed(first)
            .is_out_box(&other.transformed(second))
    }
    /// `Dump()`: writes the raw stored sides (without gap) and the gap as the native console
    /// text; `bounds.dump(&mut std::io::stdout())?`.
    pub fn dump(&self, output: &mut impl io::Write) -> io::Result<()> {
        let labels = ["Xmin", "Xmax", "Ymin", "Ymax"];
        let sides: [DumpSide; 4] = std::array::from_fn(|index| {
            let (axis, upper) = (index / 2, index % 2 == 1);
            let stored = if upper {
                self.maximum[axis]
            } else {
                self.minimum[axis]
            };
            (
                labels[index],
                (!self.side_open(axis, upper)).then_some(stored),
            )
        });
        let state = (self.is_void(), self.is_whole());
        text_dump::write_box_text(output, "Box2d : ", state, &sides, self.gap)
    }
}
