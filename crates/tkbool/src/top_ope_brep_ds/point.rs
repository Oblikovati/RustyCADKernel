// Rust translation of ModelingAlgorithms/TKBool/TopOpeBRepDS/TopOpeBRepDS_Point.hxx/.cxx and
// TopOpeBRepDS_Surface.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The point and surface geometries of the data structure, with their tolerances and
//! keep flags.
use crate::top_ope_brep_tool::TopOpeBRepToolShapeTool;
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::GeomSurface;
use rustycad_tkmath::gp::Point3;

/// A point of the data structure (`TopOpeBRepDS_Point`); natively the default
/// tolerance is uninitialized (0 here);
/// `let p = TopOpeBRepDSPoint::new(Point3::new(0., 0., 0.), 1e-7);`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TopOpeBRepDSPoint {
    my_point: Point3,
    my_tolerance: f64,
    my_keep: bool,
}

impl Default for TopOpeBRepDSPoint {
    /// `TopOpeBRepDS_Point()`: the origin, kept.
    fn default() -> Self {
        Self {
            my_point: Point3::default(),
            my_tolerance: 0.,
            my_keep: true,
        }
    }
}

impl TopOpeBRepDSPoint {
    /// `TopOpeBRepDS_Point(P, T)`: kept; `TopOpeBRepDSPoint::new(p, 1e-7)`.
    pub fn new(p: Point3, t: f64) -> Self {
        Self {
            my_point: p,
            my_tolerance: t,
            my_keep: true,
        }
    }

    /// `TopOpeBRepDS_Point(S)`: the point and tolerance of a vertex
    /// (Standard_ProgramError for another shape); natively the keep flag is left
    /// uninitialized, it is true here; `TopOpeBRepDSPoint::from_shape(&vertex)?`.
    pub fn from_shape(s: &Shape) -> Result<Self, Failure> {
        Ok(Self::new(
            TopOpeBRepToolShapeTool::pnt(s)?,
            TopOpeBRepToolShapeTool::tolerance(s)?,
        ))
    }

    /// `IsEqual(other)`: within the greater tolerance; `p.is_equal(&q)`.
    pub fn is_equal(&self, other: &TopOpeBRepDSPoint) -> bool {
        let t = self.my_tolerance.max(other.tolerance());
        self.my_point.is_equal(&other.point(), t)
    }

    /// `Point()`; `p.point()`.
    pub fn point(&self) -> Point3 {
        self.my_point
    }

    /// `ChangePoint()`; `*p.change_point() = q`.
    pub fn change_point(&mut self) -> &mut Point3 {
        &mut self.my_point
    }

    /// `Tolerance()`; `p.tolerance()`.
    pub fn tolerance(&self) -> f64 {
        self.my_tolerance
    }

    /// `Tolerance(Tol)`; `p.set_tolerance(1e-6)`.
    pub fn set_tolerance(&mut self, tol: f64) {
        self.my_tolerance = tol;
    }

    /// `Keep()`; `p.keep()`.
    pub fn keep(&self) -> bool {
        self.my_keep
    }

    /// `ChangeKeep(B)`; `p.change_keep(false)`.
    pub fn change_keep(&mut self, b: bool) {
        self.my_keep = b;
    }
}

/// A surface of the data structure (`TopOpeBRepDS_Surface`): not kept by default;
/// `let s = TopOpeBRepDSSurface::new(&surface, 1e-7);`.
#[derive(Clone, Default)]
pub struct TopOpeBRepDSSurface {
    my_surface: Handle<dyn GeomSurface>,
    my_tolerance: f64,
    my_keep: bool,
}

impl TopOpeBRepDSSurface {
    /// `TopOpeBRepDS_Surface(P, T)`: not kept; `TopOpeBRepDSSurface::new(&s, 1e-7)`.
    pub fn new(p: &Handle<dyn GeomSurface>, t: f64) -> Self {
        Self {
            my_surface: p.clone(),
            my_tolerance: t,
            my_keep: false,
        }
    }

    /// `Assign(Other)` (`operator=`); `s.assign(&other)`.
    pub fn assign(&mut self, other: &TopOpeBRepDSSurface) {
        *self = other.clone();
    }

    /// `Surface()`; `s.surface()`.
    pub fn surface(&self) -> &Handle<dyn GeomSurface> {
        &self.my_surface
    }

    /// `Tolerance()`; `s.tolerance()`.
    pub fn tolerance(&self) -> f64 {
        self.my_tolerance
    }

    /// `Tolerance(theTol)`; `s.set_tolerance(1e-6)`.
    pub fn set_tolerance(&mut self, the_tol: f64) {
        self.my_tolerance = the_tol;
    }

    /// `Keep()`; `s.keep()`.
    pub fn keep(&self) -> bool {
        self.my_keep
    }

    /// `ChangeKeep(theToKeep)`; `s.change_keep(true)`.
    pub fn change_keep(&mut self, the_to_keep: bool) {
        self.my_keep = the_to_keep;
    }
}
