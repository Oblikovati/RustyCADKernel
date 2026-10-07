// Rust translation of ModelingAlgorithms/TKBool/TopOpeBRepTool/TopOpeBRepTool_ShapeTool.hxx/.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `TopOpeBRepTool_ShapeTool`: tolerances, points, basis geometries, UV bounds, periodic
//! adjustments, orientation comparisons and resolutions of shapes.
mod orientation;
mod periodic;

use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topods::{self, Edge, Face, Shape};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkg3d::geom::{
    GeomCurve, GeomOffsetCurve, GeomOffsetSurface, GeomRectangularTrimmedSurface, GeomSurface,
    GeomSurfaceOfLinearExtrusion, GeomSurfaceOfRevolution, GeomSweptSurface, GeomTrimmedCurve,
};
use rustycad_tkg3d::geom_adaptor::GeomAdaptorSurface;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::gp::Point3;

/// The static functions of `TopOpeBRepTool_ShapeTool`;
/// `TopOpeBRepToolShapeTool::tolerance(&shape)?`.
pub struct TopOpeBRepToolShapeTool;

/// The UV periodicity and bounds of `UVBOUNDS`: `(UPeriodic, VPeriodic, Umin, Umax,
/// Vmin, Vmax)`.
pub type UvBounds = (bool, bool, f64, f64, f64, f64);

/// Standard_ProgramError of the shape tool.
pub(crate) fn program_error(message: &str) -> Failure {
    Failure::with_type("Standard_ProgramError", message, "TopOpeBRepTool_ShapeTool")
}

/// Standard_NullObject of a null geometry (natively a null dereference).
pub(crate) fn null_geometry(member: &str) -> Failure {
    Failure::with_type(
        "Standard_NullObject",
        &format!("{member}: null geometry; expected a curve or surface"),
        "TopOpeBRepTool_ShapeTool",
    )
}

impl TopOpeBRepToolShapeTool {
    /// `Tolerance(S)`: the tolerance of a face, edge or vertex, 0 for a null shape;
    /// Standard_ProgramError for another type; `TopOpeBRepToolShapeTool::tolerance(&edge)?`.
    pub fn tolerance(s: &Shape) -> Result<f64, Failure> {
        if s.is_null() {
            return Ok(0.);
        }
        match s.shape_type()? {
            ShapeEnum::Face => BRepTool::face_tolerance(&topods::face(s)?),
            ShapeEnum::Edge => BRepTool::edge_tolerance(&topods::edge(s)?),
            ShapeEnum::Vertex => BRepTool::vertex_tolerance(&topods::vertex(s)?),
            t => Err(program_error(&format!(
                "TopOpeBRepTool_ShapeTool : Shape has no tolerance: {t:?}; expected a face, edge or vertex"
            ))),
        }
    }

    /// `Pnt(S)`: the point of a vertex; Standard_ProgramError for another shape;
    /// `TopOpeBRepToolShapeTool::pnt(&vertex)?`.
    pub fn pnt(s: &Shape) -> Result<Point3, Failure> {
        let t = s.shape_type()?;
        if t != ShapeEnum::Vertex {
            return Err(program_error(&format!(
                "TopOpeBRepTool_ShapeTool::Pnt: {t:?}; expected a vertex"
            )));
        }
        BRepTool::pnt(&topods::vertex(s)?)
    }

    /// `BASISCURVE(C)`: the basis of offset and trimmed curves, recursively;
    /// `TopOpeBRepToolShapeTool::basis_curve(&c)?`.
    pub fn basis_curve(c: &Handle<dyn GeomCurve>) -> Result<Handle<dyn GeomCurve>, Failure> {
        if c.is_null() {
            return Err(null_geometry("TopOpeBRepTool_ShapeTool::BASISCURVE"));
        }
        let offset: Handle<GeomOffsetCurve> = c.clone().cast();
        if let Some(o) = offset.get() {
            return Self::basis_curve(&o.basis_curve());
        }
        let trimmed: Handle<GeomTrimmedCurve> = c.clone().cast();
        if let Some(t) = trimmed.get() {
            return Self::basis_curve(&t.basis_curve());
        }
        Ok(c.clone())
    }

    /// `BASISCURVE(E)`: the basis of the 3d curve of the edge, null without one;
    /// `TopOpeBRepToolShapeTool::basis_curve_of_edge(&edge)?`.
    pub fn basis_curve_of_edge(e: &Edge) -> Result<Handle<dyn GeomCurve>, Failure> {
        let (c, _, _) = BRepTool::curve(e)?;
        if c.is_null() {
            return Ok(c);
        }
        Self::basis_curve(&c)
    }

    /// `BASISSURFACE(S)`: the basis of offset and rectangular trimmed surfaces,
    /// recursively; `TopOpeBRepToolShapeTool::basis_surface(&s)?`.
    pub fn basis_surface(s: &Handle<dyn GeomSurface>) -> Result<Handle<dyn GeomSurface>, Failure> {
        if s.is_null() {
            return Err(null_geometry("TopOpeBRepTool_ShapeTool::BASISSURFACE"));
        }
        let offset: Handle<GeomOffsetSurface> = s.clone().cast();
        if let Some(o) = offset.get() {
            return Self::basis_surface(&o.basis_surface());
        }
        let trimmed: Handle<GeomRectangularTrimmedSurface> = s.clone().cast();
        if let Some(t) = trimmed.get() {
            return Self::basis_surface(&t.basis_surface());
        }
        Ok(s.clone())
    }

    /// `BASISSURFACE(F)`: the basis of the surface of the face (without its location);
    /// `TopOpeBRepToolShapeTool::basis_surface_of_face(&face)?`.
    pub fn basis_surface_of_face(f: &Face) -> Result<Handle<dyn GeomSurface>, Failure> {
        let (s, _) = BRepTool::surface_with_location(f)?;
        Self::basis_surface(&s)
    }

    /// `UVBOUNDS(S, ...)`: for a surface of revolution of a periodic basis curve, V
    /// periodic on the curve range (U untouched); for an extrusion of a periodic curve,
    /// U periodic on its range (V untouched); else the periodicity and bounds of the
    /// basis surface. `out` holds the native output arguments, kept where untouched;
    /// `TopOpeBRepToolShapeTool::uv_bounds(&s, &mut out)?`.
    pub fn uv_bounds(s: &Handle<dyn GeomSurface>, out: &mut UvBounds) -> Result<(), Failure> {
        let bs = Self::basis_surface(s)?;
        let revolution: Handle<GeomSurfaceOfRevolution> = bs.clone().cast();
        if let Some(sr) = revolution.get() {
            let c = Self::basis_curve(&sr.basis_curve())?;
            let curve = c.get().ok_or_else(|| null_geometry("UVBOUNDS"))?;
            if curve.is_periodic() {
                (out.0, out.1, out.4, out.5) =
                    (false, true, curve.first_parameter(), curve.last_parameter());
            }
            return Ok(());
        }
        let extrusion: Handle<GeomSurfaceOfLinearExtrusion> = bs.clone().cast();
        if let Some(se) = extrusion.get() {
            let c = Self::basis_curve(&se.basis_curve())?;
            let curve = c.get().ok_or_else(|| null_geometry("UVBOUNDS"))?;
            if curve.is_periodic() {
                (out.0, out.2, out.3, out.1) =
                    (true, curve.first_parameter(), curve.last_parameter(), false);
            }
            return Ok(());
        }
        let surface = bs.get().ok_or_else(|| null_geometry("UVBOUNDS"))?;
        let (umin, umax, vmin, vmax) = surface.bounds();
        *out = (
            surface.is_u_periodic(),
            surface.is_v_periodic(),
            umin,
            umax,
            vmin,
            vmax,
        );
        Ok(())
    }

    /// `UVBOUNDS(F, ...)`: the UV bounds of the surface of the face;
    /// `TopOpeBRepToolShapeTool::uv_bounds_of_face(&face, &mut out)?`.
    pub fn uv_bounds_of_face(f: &Face, out: &mut UvBounds) -> Result<(), Failure> {
        let (s, _) = BRepTool::surface_with_location(f)?;
        Self::uv_bounds(&s, out)
    }

    /// `Resolution3dU(SU, Tol2d)`: the 3d length of `tol2d` along U, from the U
    /// resolution of 1e-5; `TopOpeBRepToolShapeTool::resolution3d_u(&s, 1e-9)?`.
    pub fn resolution3d_u(su: &Handle<dyn GeomSurface>, tol2d: f64) -> Result<f64, Failure> {
        let gas = GeomAdaptorSurface::with_surface(su)?;
        let r3dunit: f64 = 0.00001;
        let ru = gas.u_resolution(r3dunit)?;
        Ok(r3dunit * (tol2d / ru))
    }

    /// `Resolution3dV(SU, Tol2d)`: the 3d length of `tol2d` along V;
    /// `TopOpeBRepToolShapeTool::resolution3d_v(&s, 1e-9)?`.
    pub fn resolution3d_v(su: &Handle<dyn GeomSurface>, tol2d: f64) -> Result<f64, Failure> {
        let gas = GeomAdaptorSurface::with_surface(su)?;
        let r3dunit: f64 = 0.00001;
        let rv = gas.v_resolution(r3dunit)?;
        Ok(r3dunit * (tol2d / rv))
    }

    /// `Resolution3d(SU, Tol2d)`: the greater of both resolutions;
    /// `TopOpeBRepToolShapeTool::resolution3d(&s, 1e-9)?`.
    pub fn resolution3d(su: &Handle<dyn GeomSurface>, tol2d: f64) -> Result<f64, Failure> {
        let ru = Self::resolution3d_u(su, tol2d)?;
        let rv = Self::resolution3d_v(su, tol2d)?;
        Ok(ru.max(rv))
    }

    /// `Resolution3d(F, Tol2d)`: on the surface of the face (without its location);
    /// `TopOpeBRepToolShapeTool::resolution3d_of_face(&face, 1e-9)?`.
    pub fn resolution3d_of_face(f: &Face, tol2d: f64) -> Result<f64, Failure> {
        let (su, _) = BRepTool::surface_with_location(f)?;
        Self::resolution3d(&su, tol2d)
    }
}
