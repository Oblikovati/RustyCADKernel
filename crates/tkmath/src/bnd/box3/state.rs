// Rust translation of FoundationClasses/TKMath/Bnd/Bnd_Box.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

use super::super::BoundingBox3;
use crate::gp::Point3;
use rustycad_tkernel::standard::Failure;
impl BoundingBox3 {
    /// Resets corners, flags and tolerance; `bounds.set_void()`.
    pub fn set_void(&mut self) {
        *self = Self::default();
    }
    /// Opens every side without resetting the stored finite corners; `bounds.set_whole()`.
    pub fn set_whole(&mut self) {
        self.flags = 126;
    }
    /// Reports all six open flags; `bounds.is_whole()`.
    pub const fn is_whole(&self) -> bool {
        self.flags & 126 == 126
    }
    /// Reports any open side, independently of voidness; `bounds.is_open()`.
    pub const fn is_open(&self) -> bool {
        self.flags & 126 != 0
    }
    /// Reports native finite-part availability using the X interval; `bounds.has_finite_part()`.
    pub fn has_finite_part(&self) -> bool {
        !self.is_void() && self.maximum[0] >= self.minimum[0]
    }
    /// Copies finite corners while clearing open flags; `bounds.finite_part()`.
    pub fn finite_part(&self) -> Self {
        if !self.has_finite_part() {
            return Self::default();
        }
        Self {
            flags: 0,
            ..self.clone()
        }
    }
    /// Increases the gap without cumulatively moving corners (OCC16485); `bounds.enlarge(1e-3)`.
    pub fn enlarge(&mut self, tolerance: f64) {
        if self.gap < tolerance.abs() {
            self.gap = tolerance.abs();
        }
    }
    /// Returns the gap-adjusted lower corner; `bounds.corner_min()?`.
    pub fn corner_min(&self) -> Result<Point3, Failure> {
        let [x, y, z, _, _, _] = self.get()?;
        Ok(Point3::new(x, y, z))
    }
    /// Returns the gap-adjusted upper corner; `bounds.corner_max()?`.
    pub fn corner_max(&self) -> Result<Point3, Failure> {
        let [_, _, _, x, y, z] = self.get()?;
        Ok(Point3::new(x, y, z))
    }
    /// Returns no center for a void box; `bounds.center()`.
    pub fn center(&self) -> Option<Point3> {
        let [x, y, z, u, v, w] = self.get().ok()?;
        Some(Point3::new(0.5 * (x + u), 0.5 * (y + v), 0.5 * (z + w)))
    }
    /// Uses finite stored corners and two gap additions, even for open boxes; `bounds.square_extent()`.
    pub fn square_extent(&self) -> f64 {
        if self.is_void() {
            return 0.0;
        }
        let [x, y, z] =
            std::array::from_fn(|i| self.maximum[i] - self.minimum[i] + self.gap + self.gap);
        x * x + y * y + z * z
    }
    /// Requires every finite dimension to be thin, retaining native NaN comparisons; `bounds.is_thin(1e-7)`.
    pub fn is_thin(&self, tolerance: f64) -> bool {
        if self.is_whole() {
            return false;
        }
        if self.is_void() {
            return true;
        }
        !self.is_open() && !(0..3).any(|i| self.maximum[i] - self.minimum[i] >= tolerance)
    }
    pub(super) fn axis_is_thin(&self, axis: usize, tolerance: f64) -> bool {
        if self.is_whole() {
            return false;
        }
        if self.is_void() {
            return true;
        }
        !self.side_open(axis, false)
            && !self.side_open(axis, true)
            && self.maximum[axis] - self.minimum[axis] < tolerance
    }
    pub(super) fn side_open(&self, axis: usize, upper: bool) -> bool {
        self.flags & side_mask(axis, upper) != 0
    }
    pub(super) fn open_side(&mut self, axis: usize, upper: bool) {
        self.flags |= side_mask(axis, upper);
    }
}
fn side_mask(axis: usize, upper: bool) -> i32 {
    1 << (1 + 2 * axis + usize::from(upper))
}

impl BoundingBox3 {
    /// Opens the X min side; `bounds.open_xmin()`.
    pub fn open_xmin(&mut self) {
        self.open_side(0, false);
    }
    /// Queries the X min flag; `bounds.is_open_xmin()`.
    pub fn is_open_xmin(&self) -> bool {
        self.side_open(0, false)
    }
    /// Opens the X max side; `bounds.open_xmax()`.
    pub fn open_xmax(&mut self) {
        self.open_side(0, true);
    }
    /// Queries the X max flag; `bounds.is_open_xmax()`.
    pub fn is_open_xmax(&self) -> bool {
        self.side_open(0, true)
    }
    /// Checks X thickness without gap; `bounds.is_x_thin(1e-7)`.
    pub fn is_x_thin(&self, tolerance: f64) -> bool {
        self.axis_is_thin(0, tolerance)
    }
    /// Opens the Y min side; `bounds.open_ymin()`.
    pub fn open_ymin(&mut self) {
        self.open_side(1, false);
    }
    /// Queries the Y min flag; `bounds.is_open_ymin()`.
    pub fn is_open_ymin(&self) -> bool {
        self.side_open(1, false)
    }
    /// Opens the Y max side; `bounds.open_ymax()`.
    pub fn open_ymax(&mut self) {
        self.open_side(1, true);
    }
    /// Queries the Y max flag; `bounds.is_open_ymax()`.
    pub fn is_open_ymax(&self) -> bool {
        self.side_open(1, true)
    }
    /// Checks Y thickness without gap; `bounds.is_y_thin(1e-7)`.
    pub fn is_y_thin(&self, tolerance: f64) -> bool {
        self.axis_is_thin(1, tolerance)
    }
    /// Opens the Z min side; `bounds.open_zmin()`.
    pub fn open_zmin(&mut self) {
        self.open_side(2, false);
    }
    /// Queries the Z min flag; `bounds.is_open_zmin()`.
    pub fn is_open_zmin(&self) -> bool {
        self.side_open(2, false)
    }
    /// Opens the Z max side; `bounds.open_zmax()`.
    pub fn open_zmax(&mut self) {
        self.open_side(2, true);
    }
    /// Queries the Z max flag; `bounds.is_open_zmax()`.
    pub fn is_open_zmax(&self) -> bool {
        self.side_open(2, true)
    }
    /// Checks Z thickness without gap; `bounds.is_z_thin(1e-7)`.
    pub fn is_z_thin(&self, tolerance: f64) -> bool {
        self.axis_is_thin(2, tolerance)
    }
}
