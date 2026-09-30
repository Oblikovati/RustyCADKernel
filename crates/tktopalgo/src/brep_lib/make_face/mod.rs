// Rust translation of BRepLib_MakeFace.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepLib_MakeFace`: a face on a surface, bounded by its natural restriction or by
//! the given wires.
mod bounded;
mod constructors;
mod edges;
mod sides;

use super::degenerated_curve::degenerated_tolerance;
use super::errors::BRepLibFaceError;
use super::make_shape::BRepLibMakeShape;
use crate::brep_top_adaptor::BRepTopAdaptorFClass2d;
use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::topods::{self, Face, Shape, ShapeIterator, Wire};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::{GeomCurve, GeomSurface};
use rustycad_tkg3d::top_abs::State;
use std::ops::{Deref, DerefMut};

/// `BRepLib_MakeFace` (docs/topalgo-design.md section 7): a face on a surface, with the
/// edges of its natural bounds or of the given bounds (`init_surface_bounds`), or with
/// the wires added to it. The constructors on gp surfaces build the Geom surface
/// first. A failing Init records its error and keeps an earlier face, as natively.
///
/// ```
/// use rustycad_tkmath::gp::Plane3;
/// use rustycad_tktopalgo::brep_lib::{BRepLibFaceError, BRepLibMakeFace};
/// let maker = BRepLibMakeFace::from_plane_bounds(Plane3::default(), (0.0, 1.0, 0.0, 2.0))?;
/// assert_eq!(maker.error(), BRepLibFaceError::FaceDone);
/// assert!(maker.is_done());
/// # Ok::<(), rustycad_tkernel::standard::Failure>(())
/// ```
#[derive(Clone)]
pub struct BRepLibMakeFace {
    base: BRepLibMakeShape,
    my_error: BRepLibFaceError,
}

impl Default for BRepLibMakeFace {
    fn default() -> Self {
        Self::new()
    }
}

impl Deref for BRepLibMakeFace {
    type Target = BRepLibMakeShape;
    fn deref(&self) -> &BRepLibMakeShape {
        &self.base
    }
}

impl DerefMut for BRepLibMakeFace {
    fn deref_mut(&mut self) -> &mut BRepLibMakeShape {
        &mut self.base
    }
}

impl BRepLibMakeFace {
    /// `BRepLib_MakeFace()`: not done, `BRepLib_NoFace`; `BRepLibMakeFace::new()`.
    pub fn new() -> Self {
        Self {
            base: BRepLibMakeShape::new(),
            my_error: BRepLibFaceError::NoFace,
        }
    }

    /// `Init(F)`: an empty copy of the face with its wires; `BRepLib_FaceDone`, but the
    /// maker is not marked done (natively too); `maker.init_face(&face)?`.
    pub fn init_face(&mut self, face: &Face) -> Result<(), Failure> {
        // copy the face
        self.base.my_shape = face.empty_copied()?;
        self.my_error = BRepLibFaceError::FaceDone;
        let builder = BRepBuilder::default();
        let mut children = ShapeIterator::new(face, true, true);
        while let Ok(child) = children.value() {
            builder.add(&mut self.base.my_shape, child)?;
            children.next();
        }
        Ok(())
    }

    /// `Init(S, Bound, TolDegen)`: with `bound`, the face over the natural bounds of the
    /// surface ([`Self::init_surface_bounds`]), else a face on the surface without
    /// edges (not marked done); both flagged with the natural restriction;
    /// `maker.init_surface(&s, true, 1e-7)?`.
    pub fn init_surface(
        &mut self,
        surface: &Handle<dyn GeomSurface>,
        bound: bool,
        tol_degen: f64,
    ) -> Result<(), Failure> {
        self.my_error = BRepLibFaceError::FaceDone;
        let builder = BRepBuilder::default();
        if bound {
            let s = surface
                .get()
                .ok_or_else(|| super::errors::null_object("BRepLib_MakeFace: null surface"))?;
            let (umin, umax, vmin, vmax) = s.bounds();
            self.init_surface_bounds(surface, (umin, umax, vmin, vmax), tol_degen)?;
        } else {
            let mut face = Face::default();
            builder.make_face_on_surface(&mut face, surface, precision::confusion())?;
            self.base.my_shape = face.into_shape();
        }
        builder.natural_restriction(&topods::face(&self.base.my_shape)?, true)
    }

    /// `Add(W)`: adds the wire to the face, clears the natural restriction flag and marks
    /// the maker done; `maker.add(&wire)?`.
    pub fn add(&mut self, wire: &Wire) -> Result<(), Failure> {
        let builder = BRepBuilder::default();
        builder.add(&mut self.base.my_shape, wire)?;
        builder.natural_restriction(&topods::face(&self.base.my_shape)?, false)?;
        self.set_done();
        Ok(())
    }

    /// `Error()`; `maker.error()`.
    pub fn error(&self) -> BRepLibFaceError {
        self.my_error
    }

    /// `Face()` and `operator TopoDS_Face()`: the face under construction (not checked,
    /// as natively); `maker.face()?`.
    pub fn face(&self) -> Result<Face, Failure> {
        topods::face(&self.base.my_shape)
    }

    /// `IsDegenerated(theCurve, theMaxTol, theActTol)`: whether the curve is a point
    /// within `max_tol` (a small circle, or a Bezier or B-spline curve with close poles),
    /// with the actual tolerance (Precision::Confusion() when it is not);
    /// `BRepLibMakeFace::is_degenerated(&curve, 1e-3)?`.
    pub fn is_degenerated(
        curve: &Handle<dyn GeomCurve>,
        max_tol: f64,
    ) -> Result<(bool, f64), Failure> {
        Ok(match degenerated_tolerance(curve, max_tol)? {
            Some(tolerance) => (true, tolerance),
            None => (false, precision::confusion()),
        })
    }

    /// `CheckInside()`: reverses the wires of the face when the infinite point is inside
    /// (the face is then rebuilt as an empty copy with the reversed wires).
    fn check_inside(&mut self) -> Result<(), Failure> {
        // compute the area and return the face if the area is negative
        let classifier = BRepTopAdaptorFClass2d::new(&self.face()?, 0.)?;
        if classifier.perform_infinite_point()? != State::In {
            return Ok(());
        }
        let builder = BRepBuilder::default();
        let mut reversed: Shape = self.base.my_shape.empty_copied()?;
        let mut children = ShapeIterator::new(&self.base.my_shape, true, true);
        while let Ok(child) = children.value() {
            builder.add(&mut reversed, &child.reversed())?;
            children.next();
        }
        self.base.my_shape = reversed;
        Ok(())
    }

    /// The maker after `init`, as the native constructors run it.
    fn initialized(init: impl FnOnce(&mut Self) -> Result<(), Failure>) -> Result<Self, Failure> {
        let mut maker = Self::new();
        init(&mut maker)?;
        Ok(maker)
    }
}
