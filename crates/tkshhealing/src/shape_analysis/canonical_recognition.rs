// Rust translation of ModelingAlgorithms/TKShHealing/ShapeAnalysis/ShapeAnalysis_CanonicalRecognition.hxx
// and the members of ShapeAnalysis_CanonicalRecognition.cxx (the static helpers are in the
// submodules) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `ShapeAnalysis_CanonicalRecognition`: recognizes planes, cylinders, cones, spheres,
//! lines, circles and ellipses under the geometry of shells, faces, wires and edges.
mod conic;
mod least_squares;
mod params;
mod surfaces;

use params::set_surf_params;
use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::topods::{self, Shape, Wire};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::GeomSurface;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkgeombase::geom_convert::ConvType;
use rustycad_tkmath::geom_abs::SurfaceType;
use rustycad_tkmath::gp::{Axis3, Cone3, Cylinder3, Plane3, Sphere3};
use rustycad_tktopalgo::brep_lib::BRepLibFindSurface;

/// `ShapeAnalysis_CanonicalRecognition`: finds whether the geometry under a shape can be
/// replaced by an elementary surface or a conic within a tolerance.
///
/// The status is -1 before a shape is set, 0 when an operation may run, 1 after a
/// failure (then `ClearStatus` resets it). The native in/out arguments (the sample
/// surface or curve, which also seeds the least-squares fit of wires and edges) are
/// `&mut` values.
///
/// ```
/// use rustycad_tkshhealing::shape_analysis::ShapeAnalysisCanonicalRecognition;
/// let recognizer = ShapeAnalysisCanonicalRecognition::new();
/// assert_eq!(recognizer.get_status(), -1);
/// assert_eq!(recognizer.get_gap(), -1.0);
/// ```
#[derive(Clone, Debug)]
pub struct ShapeAnalysisCanonicalRecognition {
    shape: Shape,
    s_type: ShapeEnum,
    gap: f64,
    status: i32,
}

impl Default for ShapeAnalysisCanonicalRecognition {
    fn default() -> Self {
        Self {
            shape: Shape::default(),
            s_type: ShapeEnum::Shape,
            gap: -1.0,
            status: -1,
        }
    }
}

impl ShapeAnalysisCanonicalRecognition {
    /// `ShapeAnalysis_CanonicalRecognition()`: no shape, status -1, gap -1;
    /// `ShapeAnalysisCanonicalRecognition::new()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// `ShapeAnalysis_CanonicalRecognition(theShape)`: [`Self::set_shape`] on a new tool;
    /// `ShapeAnalysisCanonicalRecognition::with_shape(&face)?`.
    pub fn with_shape(shape: &Shape) -> Result<Self, Failure> {
        let mut tool = Self::default();
        tool.init(shape)?;
        Ok(tool)
    }

    /// `SetShape(theShape)`: takes a shell, face, wire or edge (status 0); any other type
    /// sets the status 1 and keeps the previous shape. A null shape is
    /// Standard_NullObject (natively its type is read through a null pointer);
    /// `recognizer.set_shape(&edge)?`.
    pub fn set_shape(&mut self, shape: &Shape) -> Result<(), Failure> {
        self.init(shape)
    }

    /// The private `Init`.
    fn init(&mut self, shape: &Shape) -> Result<(), Failure> {
        let kind = shape.shape_type()?;
        match kind {
            ShapeEnum::Shell | ShapeEnum::Face | ShapeEnum::Wire | ShapeEnum::Edge => {
                self.shape = shape.clone();
                self.s_type = kind;
                self.status = 0;
            }
            _ => self.status = 1,
        }
        Ok(())
    }

    /// `GetShape()`; `recognizer.get_shape()`.
    pub fn get_shape(&self) -> &Shape {
        &self.shape
    }

    /// `GetGap()`: the deviation of the input geometry from the last recognized entity;
    /// `recognizer.get_gap()`.
    pub fn get_gap(&self) -> f64 {
        self.gap
    }

    /// `GetStatus()`; `recognizer.get_status()`.
    pub fn get_status(&self) -> i32 {
        self.status
    }

    /// `ClearStatus()`: sets the status to 0; `recognizer.clear_status()`.
    pub fn clear_status(&mut self) {
        self.status = 0;
    }

    /// The private `IsElementarySurf(theTarget, theTol, thePos, theParams)`.
    fn is_elementary_surf(
        &mut self,
        target: SurfaceType,
        tol: f64,
        pos: &mut Axis3,
        params: &mut [f64],
    ) -> Result<bool, Failure> {
        if self.status != 0 {
            return Ok(false);
        }
        if !matches!(
            self.s_type,
            ShapeEnum::Face | ShapeEnum::Shell | ShapeEnum::Edge | ShapeEnum::Wire
        ) {
            return Ok(false);
        }
        let surface = self.elementary_geometry(target, tol, pos, params)?;
        if surface.is_null() && matches!(self.s_type, ShapeEnum::Face | ShapeEnum::Shell) {
            return Ok(false);
        }
        if !set_surf_params(target, &surface, pos, params)? {
            self.status = 1;
            return Ok(false);
        }
        Ok(true)
    }

    /// Select the native topology branch without altering its status on a null face surface.
    fn elementary_geometry(
        &mut self,
        target: SurfaceType,
        tol: f64,
        pos: &mut Axis3,
        params: &mut [f64],
    ) -> Result<Handle<dyn GeomSurface>, Failure> {
        let mut fit = surfaces::SurfFit {
            pos,
            params,
            gap: &mut self.gap,
            status: &mut self.status,
        };
        match self.s_type {
            ShapeEnum::Face => surfaces::face_surface(
                &topods::face(&self.shape)?,
                tol,
                ConvType::Target,
                target,
                fit.gap,
                fit.status,
            ),
            ShapeEnum::Shell => surfaces::shell_surface(
                &topods::shell(&self.shape)?,
                tol,
                ConvType::Target,
                target,
                fit.gap,
                fit.status,
            ),
            ShapeEnum::Edge => surfaces::edge_surface(
                &topods::edge(&self.shape)?,
                tol,
                ConvType::Target,
                target,
                &mut fit,
            ),
            ShapeEnum::Wire => surfaces::wire_surface(
                &topods::wire(&self.shape)?,
                tol,
                ConvType::Target,
                target,
                &mut fit,
            ),
            _ => Ok(Handle::default()),
        }
    }

    /// `IsPlane(theTol, thePln)`: the plane of the face or shell geometry within `tol`;
    /// for a wire or an edge, else a plane found through them (`BRepLib_FindSurface`, the
    /// gap is its reached tolerance); `recognizer.is_plane(1e-7, &mut plane)?`.
    pub fn is_plane(&mut self, tol: f64, pln: &mut Plane3) -> Result<bool, Failure> {
        let mut pos = pln.position();
        let mut params = [0.0; 1];
        if self.is_elementary_surf(SurfaceType::Plane, tol, &mut pos, &mut params)? {
            pln.set_position(pos);
            return Ok(true);
        }
        if self.s_type != ShapeEnum::Edge && self.s_type != ShapeEnum::Wire {
            return Ok(false);
        }
        self.find_plane(tol, pln)
    }

    /// The planar wire/edge fallback also recovers a prior error status, as OCCT does.
    fn find_plane(&mut self, tol: f64, pln: &mut Plane3) -> Result<bool, Failure> {
        let finder = BRepLibFindSurface::new(&self.shape, tol, true, false)?;
        if !finder.found() {
            self.status = 1;
            return Ok(false);
        }
        let plane = finder.surface().cast::<rustycad_tkg3d::geom::GeomPlane>();
        let plane = plane.get().ok_or_else(|| {
            Failure::with_message(
                "Standard_NullObject",
                "ShapeAnalysis_CanonicalRecognition::IsPlane: the found surface is not a Geom_Plane",
            )
        })?;
        *pln = plane.pln();
        self.gap = finder.tolerance_reached();
        self.status = 0;
        Ok(true)
    }

    /// The wire of the least-squares fit: the wire itself, or a new wire of the edge.
    fn ls_wire(&self) -> Result<Option<Wire>, Failure> {
        match self.shape.shape_type()? {
            ShapeEnum::Edge => {
                let builder = BRepBuilder::default();
                let mut wire = Wire::default();
                builder.make_wire(&mut wire);
                builder.add(&mut wire, &self.shape)?;
                Ok(Some(wire))
            }
            ShapeEnum::Wire => Ok(Some(topods::wire(&self.shape)?)),
            _ => Ok(None),
        }
    }

    /// The least-squares fallback of `IsCylinder`, `IsCone` and `IsSphere`, when the
    /// sample radius (parameter `radius_index`) is finite.
    fn fit_by_least_squares(
        &mut self,
        target: SurfaceType,
        tol: f64,
        pos: &mut Axis3,
        params: &mut [f64],
        radius_index: usize,
    ) -> Result<bool, Failure> {
        if params[radius_index] > precision::infinite() {
            // The sample surface does not seem to be set: least squares do not apply.
            return Ok(false);
        }
        let Some(wire) = self.ls_wire()? else {
            return Ok(false);
        };
        let mut fit = surfaces::SurfFit {
            pos,
            params,
            gap: &mut self.gap,
            status: &mut self.status,
        };
        least_squares::surface_by_ls(&wire, tol, target, &mut fit)
    }

    /// `IsCylinder(theTol, theCyl)`: the cylinder of the geometry within `tol`; for a wire
    /// or an edge, else a least-squares cylinder seeded by `cyl` (only when its radius is
    /// set); `recognizer.is_cylinder(1e-7, &mut cylinder)?`.
    pub fn is_cylinder(&mut self, tol: f64, cyl: &mut Cylinder3) -> Result<bool, Failure> {
        let mut pos = cyl.position();
        let mut params = [cyl.radius()];
        let target = SurfaceType::Cylinder;
        let found = self.is_elementary_surf(target, tol, &mut pos, &mut params)?
            || self.fit_by_least_squares(target, tol, &mut pos, &mut params, 0)?;
        if found {
            cyl.set_position(pos);
            cyl.set_radius(params[0])?;
        }
        Ok(found)
    }

    /// `IsCone(theTol, theCone)`: the cone of the geometry within `tol`; for a wire or an
    /// edge, else a least-squares cone seeded by `cone` (only when its radius is set);
    /// `recognizer.is_cone(1e-7, &mut cone)?`.
    pub fn is_cone(&mut self, tol: f64, cone: &mut Cone3) -> Result<bool, Failure> {
        let mut pos = cone.position();
        let mut params = [cone.semi_angle(), cone.ref_radius()];
        let target = SurfaceType::Cone;
        let found = self.is_elementary_surf(target, tol, &mut pos, &mut params)?
            || self.fit_by_least_squares(target, tol, &mut pos, &mut params, 1)?;
        if found {
            cone.set_position(pos);
            cone.set_semi_angle(params[0])?;
            cone.set_radius(params[1])?;
        }
        Ok(found)
    }

    /// `IsSphere(theTol, theSphere)`: the sphere of the geometry within `tol`; for a wire
    /// or an edge, else a least-squares sphere seeded by `sphere` (only when its radius is
    /// set); `recognizer.is_sphere(1e-7, &mut sphere)?`.
    pub fn is_sphere(&mut self, tol: f64, sphere: &mut Sphere3) -> Result<bool, Failure> {
        let mut pos = sphere.position();
        let mut params = [sphere.radius()];
        let target = SurfaceType::Sphere;
        let found = self.is_elementary_surf(target, tol, &mut pos, &mut params)?
            || self.fit_by_least_squares(target, tol, &mut pos, &mut params, 0)?;
        if found {
            sphere.set_position(pos);
            sphere.set_radius(params[0])?;
        }
        Ok(found)
    }
}
