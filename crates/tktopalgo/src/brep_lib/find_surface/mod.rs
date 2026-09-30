// Rust translation of BRepLib_FindSurface.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepLib_FindSurface`: a surface through the edges of a shape, an existing one
//! shared by all edges or a plane fitted to points sampled on them.
mod closed;
mod plane;

use rustycad_tkbrep::brep::{BRepTool, EdgeCurveOnSurface};
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{Edge, Shape};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::{GeomPlane, GeomRectangularTrimmedSurface, GeomSurface};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::toploc::Location;

/// `BRepLib_FindSurface` (docs/topalgo-design.md section 7): the surface of the first
/// edge's curves on surface shared by every edge (a plane only with `only_plane`, one
/// on which the wire is closed in 2d with `only_closed`), else the plane that fits
/// points sampled on the edges within the tolerance (the largest edge tolerance, or
/// the given one).
///
/// ```
/// use rustycad_tkbrep::topods::Shape;
/// use rustycad_tktopalgo::brep_lib::BRepLibFindSurface;
/// let finder = BRepLibFindSurface::new(&Shape::default(), -1.0, false, false)?;
/// assert!(!finder.found());
/// # Ok::<(), rustycad_tkernel::standard::Failure>(())
/// ```
#[derive(Clone, Default)]
pub struct BRepLibFindSurface {
    my_surface: Handle<dyn GeomSurface>,
    my_tolerance: f64,
    my_tol_reached: f64,
    is_existed: bool,
    my_location: Location,
}

impl BRepLibFindSurface {
    /// `BRepLib_FindSurface(S, Tol, OnlyPlane, OnlyClosed)`: [`Self::init`];
    /// `BRepLibFindSurface::new(&wire, -1.0, false, false)?`.
    pub fn new(
        shape: &Shape,
        tol: f64,
        only_plane: bool,
        only_closed: bool,
    ) -> Result<Self, Failure> {
        let mut finder = Self::default();
        finder.init(shape, tol, only_plane, only_closed)?;
        Ok(finder)
    }

    /// `Init(S, Tol, OnlyPlane, OnlyClosed)`: the tolerance is the largest of `tol` and
    /// the edge tolerances; an existing surface is searched first, then a plane (none
    /// for fewer than three sample points, or when the fit misses the tolerance, 5
    /// times the tolerance when `tol` is negative); `finder.init(&wire, -1.0, false, false)?`.
    pub fn init(
        &mut self,
        shape: &Shape,
        tol: f64,
        only_plane: bool,
        only_closed: bool,
    ) -> Result<(), Failure> {
        *self = Self {
            my_tolerance: tol,
            ..Self::default()
        };
        // compute the tolerance
        let edges: Vec<Edge> = Explorer::new(shape, ShapeEnum::Edge, ShapeEnum::Shape)?
            .map(Edge::unchecked)
            .collect();
        for edge in &edges {
            let t: f64 = BRepTool::edge_tolerance(edge)?;
            if t > self.my_tolerance {
                self.my_tolerance = t;
            }
        }
        // search an existing surface
        let Some(first) = edges.first() else {
            return Ok(()); // no edges ....
        };
        self.existing_surface(shape, first, &edges, only_plane, only_closed)?;
        if self.my_surface.is_some() {
            self.is_existed = true;
            return Ok(());
        }
        plane::fit_plane(self, shape, &edges, tol)
    }

    /// "iterate on the surfaces of the first edge": the first surface of its curves on
    /// surface that every other edge has at the same location (and that passes the
    /// plane and closure filters); the surface and location of the last search stay.
    fn existing_surface(
        &mut self,
        shape: &Shape,
        first: &Edge,
        edges: &[Edge],
        only_plane: bool,
        only_closed: bool,
    ) -> Result<(), Failure> {
        let mut index: i32 = 0;
        loop {
            index += 1;
            let found = BRepTool::curve_on_surface_by_index(first, index)?;
            (self.my_surface, self.my_location) = (found.surface, found.location);
            if self.my_surface.is_null() {
                return Ok(());
            }
            if !self.shared_by_all(first, edges)? {
                self.my_surface = Handle::default();
            }
            // if OnlyPlane, eval if mySurface is a plane.
            if only_plane && self.my_surface.is_some() {
                self.my_surface = plane_of(&self.my_surface);
            }
            // if S is e.g. the bottom face of a cylinder, mySurface can be the
            // lateral (cylindrical) face of the cylinder; reject an improper mySurface
            if self.my_surface.is_some()
                && (!only_closed
                    || closed::is_2d_closed(shape, &self.my_surface, &self.my_location))
            {
                return Ok(());
            }
        }
    }

    /// Whether every edge other than the first has a curve on the current surface
    /// (the same handle) at the same location.
    fn shared_by_all(&self, first: &Edge, edges: &[Edge]) -> Result<bool, Failure> {
        for edge in edges.iter().filter(|e| !first.is_same(e)) {
            let mut j: i32 = 0;
            let found: bool = loop {
                j += 1;
                let other: EdgeCurveOnSurface = BRepTool::curve_on_surface_by_index(edge, j)?;
                if other.surface.is_null() {
                    break false;
                }
                if other.surface == self.my_surface && other.location.is_equal(&self.my_location) {
                    break true;
                }
            };
            if !found {
                return Ok(false);
            }
        }
        Ok(true)
    }

    /// `Found()`; `finder.found()`.
    pub fn found(&self) -> bool {
        self.my_surface.is_some()
    }

    /// `Surface()`: the surface found (null when none); `finder.surface()`.
    pub fn surface(&self) -> Handle<dyn GeomSurface> {
        self.my_surface.clone()
    }

    /// `Tolerance()`: the tolerance of the search; `finder.tolerance()`.
    pub fn tolerance(&self) -> f64 {
        self.my_tolerance
    }

    /// `ToleranceReached()`: the largest distance from the samples to the fitted plane
    /// (0 for an existing surface); `finder.tolerance_reached()`.
    pub fn tolerance_reached(&self) -> f64 {
        self.my_tol_reached
    }

    /// `Existed()`: whether the surface is one of the edges' surfaces; `finder.existed()`.
    pub fn existed(&self) -> bool {
        self.is_existed
    }

    /// `Location()`: the location of the surface; `finder.location()`.
    pub fn location(&self) -> Location {
        self.my_location.clone()
    }
}

/// The plane of a surface (the basis of a rectangular trimmed surface first), null for
/// another surface.
fn plane_of(surface: &Handle<dyn GeomSurface>) -> Handle<dyn GeomSurface> {
    let basis = match surface.cast::<GeomRectangularTrimmedSurface>().get() {
        Some(trimmed) => trimmed.basis_surface(),
        None => surface.clone(),
    };
    match basis.cast::<GeomPlane>().get() {
        Some(_) => basis,
        None => Handle::default(),
    }
}
