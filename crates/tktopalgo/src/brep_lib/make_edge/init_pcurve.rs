// Rust translation of the Geom2d_Curve Init members of BRepLib_MakeEdge.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The `Init` members of `BRepLib_MakeEdge` on a 2d curve of a surface.
use super::BRepLibMakeEdge;
use super::ends::{EdgeRange, Step, closed_vertices, open_vertices, ordered_range, vertex_pair};
use super::project::project_on_surface_curve;
use crate::brep_lib::errors::{BRepLibEdgeError, null_object};
use crate::brep_lib::globals;
use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::topods::{Edge, Vertex};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dTrimmedCurve};
use rustycad_tkg3d::geom::GeomSurface;
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkmath::gp::Point3;
use rustycad_tkmath::toploc::Location;

/// The 2d curve of an Init; natively a null handle is dereferenced.
fn checked_pcurve(curve: &Handle<dyn Geom2dCurve>) -> Result<&dyn Geom2dCurve, Failure> {
    curve
        .get()
        .ok_or_else(|| null_object("BRepLib_MakeEdge::Init: null Geom2d_Curve; expected a curve"))
}

/// The surface of an Init; natively a null handle is dereferenced.
fn checked_surface(surface: &Handle<dyn GeomSurface>) -> Result<&dyn GeomSurface, Failure> {
    surface
        .get()
        .ok_or_else(|| null_object("BRepLib_MakeEdge::Init: null Geom_Surface; expected a surface"))
}

/// "kill trimmed curves": the basis of nested 2d trimmed curves.
fn untrimmed(curve: &Handle<dyn Geom2dCurve>) -> Handle<dyn Geom2dCurve> {
    let mut basis: Handle<dyn Geom2dCurve> = curve.clone();
    while let Some(trimmed) = basis.cast::<Geom2dTrimmedCurve>().get() {
        basis = trimmed.basis_curve();
    }
    basis
}

impl BRepLibMakeEdge {
    /// `Init(C, S)`: the whole 2d curve on the surface; `maker.init_pcurve(&c, &s)?`.
    pub fn init_pcurve(
        &mut self,
        curve: &Handle<dyn Geom2dCurve>,
        surface: &Handle<dyn GeomSurface>,
    ) -> Result<(), Failure> {
        let c: &dyn Geom2dCurve = checked_pcurve(curve)?;
        let (first, last): (f64, f64) = (c.first_parameter(), c.last_parameter());
        self.init_pcurve_params(curve, surface, first, last)
    }

    /// `Init(C, S, p1, p2)`: the 2d curve between two parameters;
    /// `maker.init_pcurve_params(&c, &s, 0.0, 1.0)?`.
    pub fn init_pcurve_params(
        &mut self,
        curve: &Handle<dyn Geom2dCurve>,
        surface: &Handle<dyn GeomSurface>,
        p1: f64,
        p2: f64,
    ) -> Result<(), Failure> {
        let null = Vertex::default();
        self.init_pcurve_vertices_params(curve, surface, &null, &null, p1, p2)
    }

    /// `Init(C, S, P1, P2)`: the 2d curve between the projections of two points, on
    /// vertices of tolerance `BRepLib::Precision()`;
    /// `maker.init_pcurve_points(&c, &s, a, b)?`.
    pub fn init_pcurve_points(
        &mut self,
        curve: &Handle<dyn Geom2dCurve>,
        surface: &Handle<dyn GeomSurface>,
        p1: Point3,
        p2: Point3,
    ) -> Result<(), Failure> {
        let tol: f64 = globals::precision();
        let (v1, v2) = vertex_pair(p1, p2, p1.distance(&p2) < tol, tol)?;
        self.init_pcurve_vertices(curve, surface, &v1, &v2)
    }

    /// `Init(C, S, V1, V2)`: the 2d curve between the projections of two vertices on the
    /// curve on surface (an extremum within Precision::Confusion() of the vertex);
    /// `maker.init_pcurve_vertices(&c, &s, &v1, &v2)?`.
    pub fn init_pcurve_vertices(
        &mut self,
        curve: &Handle<dyn Geom2dCurve>,
        surface: &Handle<dyn GeomSurface>,
        v1: &Vertex,
        v2: &Vertex,
    ) -> Result<(), Failure> {
        let Some(p1) = self.projected_on_surface(curve, surface, v1, true)? else {
            return Ok(());
        };
        let Some(p2) = self.projected_on_surface(curve, surface, v2, false)? else {
            return Ok(());
        };
        self.init_pcurve_vertices_params(curve, surface, v1, v2, p1, p2)
    }

    /// The parameter of `vertex` on the curve on surface, the 2d curve end for a null
    /// one; None after recording `BRepLib_PointProjectionFailed`.
    fn projected_on_surface(
        &mut self,
        curve: &Handle<dyn Geom2dCurve>,
        surface: &Handle<dyn GeomSurface>,
        vertex: &Vertex,
        first: bool,
    ) -> Result<Option<f64>, Failure> {
        if vertex.is_null() {
            let c: &dyn Geom2dCurve = checked_pcurve(curve)?;
            return Ok(Some(if first {
                c.first_parameter()
            } else {
                c.last_parameter()
            }));
        }
        let parameter: Option<f64> = project_on_surface_curve(curve, surface, vertex)?;
        if parameter.is_none() {
            self.my_error = BRepLibEdgeError::PointProjectionFailed;
        }
        Ok(parameter)
    }

    /// `Init(C, S, P1, P2, p1, p2)`: the 2d curve between two parameters, on vertices
    /// at two points; `maker.init_pcurve_points_params(&c, &s, a, b, 0.0, 1.0)?`.
    pub fn init_pcurve_points_params(
        &mut self,
        curve: &Handle<dyn Geom2dCurve>,
        surface: &Handle<dyn GeomSurface>,
        point1: Point3,
        point2: Point3,
        p1: f64,
        p2: f64,
    ) -> Result<(), Failure> {
        let tol: f64 = globals::precision();
        let (v1, v2) = vertex_pair(point1, point2, point1.distance(&point2) < tol, tol)?;
        self.init_pcurve_vertices_params(curve, surface, &v1, &v2, p1, p2)
    }

    /// `Init(C, S, V1, V2, p1, p2)`, the member that builds the edge on the surface:
    /// the 2d curve (trimmed curves replaced by their basis) is the only representation
    /// of the edge; when the non-periodic parameters are swapped the edge is REVERSED.
    /// There is no check of identical parameters, unlike the 3d member;
    /// `maker.init_pcurve_vertices_params(&c, &s, &v1, &v2, 0.0, 1.0)?`.
    pub fn init_pcurve_vertices_params(
        &mut self,
        curve: &Handle<dyn Geom2dCurve>,
        surface: &Handle<dyn GeomSurface>,
        vv1: &Vertex,
        vv2: &Vertex,
        pp1: f64,
        pp2: f64,
    ) -> Result<(), Failure> {
        let c: Handle<dyn Geom2dCurve> = untrimmed(curve);
        let basis: &dyn Geom2dCurve = checked_pcurve(&c)?;
        let bounds: (f64, f64) = (basis.first_parameter(), basis.last_parameter());
        let periodic: bool = basis.is_periodic();
        let mut range: EdgeRange = match ordered_range(
            periodic,
            bounds,
            (pp1, pp2),
            (vv1, vv2),
            precision::p_confusion(),
        ) {
            Ok(range) => range,
            Err(error) => return self.record(error),
        };
        match surface_ends(basis, surface, &mut range)? {
            Ok(()) => self.build_pcurve_edge(&c, surface, &range),
            Err(error) => self.record(error),
        }
    }

    /// The edge whose only representation is `c` on `surface` (tolerance
    /// `BRepLib::Precision()`), with the vertices of `range`; the maker is done.
    fn build_pcurve_edge(
        &mut self,
        c: &Handle<dyn Geom2dCurve>,
        surface: &Handle<dyn GeomSurface>,
        range: &EdgeRange,
    ) -> Result<(), Failure> {
        let preci: f64 = globals::precision();
        let builder = BRepBuilder::default();
        let mut edge: Edge = self.oriented_vertices(range);
        builder.make_edge(&mut edge)?;
        self.my_shape = edge.clone().into_shape();
        builder.update_edge_pcurve(&edge, c, surface, &Location::default(), preci, None)?;
        self.add_vertices(&mut edge)?;
        builder.range(&edge, range.p1, range.p2, false)?;
        if range.reversed {
            self.my_shape.set_orientation(Orientation::Reversed);
        }
        self.my_error = BRepLibEdgeError::EdgeDone;
        self.set_done();
        Ok(())
    }
}

/// The surface points of the 2d curve ends, the closed flag and the vertex checks.
fn surface_ends(
    basis: &dyn Geom2dCurve,
    surface: &Handle<dyn GeomSurface>,
    range: &mut EdgeRange,
) -> Step<()> {
    let point = |parameter: f64| -> Result<Point3, Failure> {
        let uv = basis.value(parameter)?;
        checked_surface(surface)?.value(uv.x(), uv.y())
    };
    let p1inf: bool = precision::is_negative_infinite(range.p1);
    let p2inf: bool = precision::is_positive_infinite(range.p2);
    let point1: Option<Point3> = if p1inf { None } else { Some(point(range.p1)?) };
    let point2: Option<Point3> = if p2inf { None } else { Some(point(range.p2)?) };
    let preci: f64 = globals::precision();
    let closed: bool = match (point1, point2) {
        (Some(a), Some(b)) => a.distance(&b) <= preci,
        _ => false,
    };
    if closed {
        let p1: Point3 = point1.unwrap_or_default();
        return Ok(closed_vertices(range, p1, preci)?.map(|_| ()));
    }
    open_vertices(range, (point1, point2), preci)
}
