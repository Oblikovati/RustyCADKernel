// Rust translation of FoundationClasses/TKMath/Bnd/Bnd_Box2d.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Planar box behavior; open bounds deliberately differ from Bnd_Box.
use crate::gp::Point2;
use rustycad_tkernel::standard::Failure;
/// Four open flags, stored bounds and a separate gap; `BoundingBox2::default()`.
#[derive(Clone, Debug)]
pub struct BoundingBox2 {
    minimum: [f64; 2],
    maximum: [f64; 2],
    gap: f64,
    flags: i32,
}
impl Default for BoundingBox2 {
    fn default() -> Self {
        Self {
            minimum: [f64::MAX; 2],
            maximum: [-f64::MAX; 2],
            gap: 0.0,
            flags: 1,
        }
    }
}
impl BoundingBox2 {
    /// Clears corners, flags and gap; `bounds.set_void()`.
    pub fn set_void(&mut self) {
        *self = Self::default();
    }
    /// Opens every side without changing stored corners or gap; `bounds.set_whole()`.
    pub fn set_whole(&mut self) {
        self.flags = 30;
    }
    /// Reports all four open flags; `bounds.is_whole()`.
    pub const fn is_whole(&self) -> bool {
        self.flags & 30 == 30
    }
    /// Reports the independent void bit; `bounds.is_void()`.
    pub const fn is_void(&self) -> bool {
        self.flags & 1 != 0
    }
    /// Stores absolute tolerance; `bounds.set_gap(1e-3)`.
    pub fn set_gap(&mut self, tolerance: f64) {
        self.gap = tolerance.abs();
    }
    /// Returns the stored gap; `bounds.gap()`.
    pub const fn gap(&self) -> f64 {
        self.gap
    }
    /// Increases tolerance without moving corners; `bounds.enlarge(1e-3)`.
    pub fn enlarge(&mut self, tolerance: f64) {
        if self.gap < tolerance.abs() {
            self.gap = tolerance.abs();
        }
    }
    /// Returns checked xmin,ymin,xmax,ymax, including gap; `bounds.get()?`.
    pub fn get(&self) -> Result<[f64; 4], Failure> {
        if self.is_void() {
            return Err(Failure::with_type(
                "Standard_ConstructionError",
                "Bnd_Box is void",
                "",
            ));
        }
        Ok(self.corner_limits())
    }
    /// Unchecked xmin, ymin, xmax, ymax (the `Get(x,y,X,Y)` output order), including void-box values.
    pub(super) fn corner_limits(&self) -> [f64; 4] {
        std::array::from_fn(|index| self.bound(index % 2, index >= 2))
    }
    fn bound(&self, axis: usize, upper: bool) -> f64 {
        if self.side_open(axis, upper) {
            return if upper { 1e100 } else { -1e100 };
        }
        if upper {
            self.maximum[axis] + self.gap
        } else {
            self.minimum[axis] - self.gap
        }
    }
    /// Returns a center for every nonvoid box; `bounds.center()`.
    pub fn center(&self) -> Option<Point2> {
        let [x, y, u, v] = self.get().ok()?;
        Some(Point2::new(0.5 * (x + u), 0.5 * (y + v)))
    }
    /// Uses stored finite corners even for open boxes; `bounds.square_extent()`.
    pub fn square_extent(&self) -> f64 {
        if self.is_void() {
            return 0.0;
        }
        let [x, y] =
            std::array::from_fn(|i| self.maximum[i] - self.minimum[i] + self.gap + self.gap);
        x * x + y * y
    }
    fn side_open(&self, axis: usize, upper: bool) -> bool {
        self.flags & (1 << (1 + 2 * axis + usize::from(upper))) != 0
    }
    fn open_side(&mut self, axis: usize, upper: bool) {
        self.flags |= 1 << (1 + 2 * axis + usize::from(upper));
    }
}
mod accumulation;
mod limits;
pub use limits::BoxLimits2;
mod queries;
mod transforms;

impl BoundingBox2 {
    /// Opens X min; `bounds.open_xmin()`.
    pub fn open_xmin(&mut self) {
        self.open_side(0, false);
    }
    /// Queries X min; `bounds.is_open_xmin()`.
    pub fn is_open_xmin(&self) -> bool {
        self.side_open(0, false)
    }
    /// Opens X max; `bounds.open_xmax()`.
    pub fn open_xmax(&mut self) {
        self.open_side(0, true);
    }
    /// Queries X max; `bounds.is_open_xmax()`.
    pub fn is_open_xmax(&self) -> bool {
        self.side_open(0, true)
    }
    /// Opens Y min; `bounds.open_ymin()`.
    pub fn open_ymin(&mut self) {
        self.open_side(1, false);
    }
    /// Queries Y min; `bounds.is_open_ymin()`.
    pub fn is_open_ymin(&self) -> bool {
        self.side_open(1, false)
    }
    /// Opens Y max; `bounds.open_ymax()`.
    pub fn open_ymax(&mut self) {
        self.open_side(1, true);
    }
    /// Queries Y max; `bounds.is_open_ymax()`.
    pub fn is_open_ymax(&self) -> bool {
        self.side_open(1, true)
    }
}
mod tree_volume;
