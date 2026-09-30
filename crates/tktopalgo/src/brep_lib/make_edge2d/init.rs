// Rust translation of the Init members of BRepLib_MakeEdge2d.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The `Init` members of `BRepLib_MakeEdge2d`.
use super::BRepLibMakeEdge2d;
use super::project::{plane_point, project_on_curve2d};
use crate::brep_lib::errors::{BRepLibEdgeError, null_object};
use crate::brep_lib::globals;
use crate::brep_lib::make_edge::ends::{EdgeRange, Step, ordered_range};
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::topods::{Edge, Vertex};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dTrimmedCurve};
use rustycad_tkg3d::geom::GeomSurface;
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkmath::gp::Point2;
use rustycad_tkmath::toploc::Location;

/// The 2d curve of an Init; natively a null handle is dereferenced.
fn checked_curve(curve: &Handle<dyn Geom2dCurve>) -> Result<&dyn Geom2dCurve, Failure> {
    curve
        .get()
        .ok_or_else(|| null_object("BRepLib_MakeEdge2d::Init: null Geom2d_Curve; expected a curve"))
}

/// "kill trimmed curves": the basis of nested 2d trimmed curves.
fn untrimmed(curve: &Handle<dyn Geom2dCurve>) -> Handle<dyn Geom2dCurve> {
    let mut basis: Handle<dyn Geom2dCurve> = curve.clone();
    while let Some(trimmed) = basis.cast::<Geom2dTrimmedCurve>().get() {
        basis = trimmed.basis_curve();
    }
    basis
}

/// The vertices of the point Init members: at the plane points of `p1` and `p2`, one
/// vertex when the 2d points are closer than Precision::Confusion().
fn plane_vertex_pair(p1: Point2, p2: Point2) -> Result<(Vertex, Vertex), Failure> {
    let builder = BRepBuilder::default();
    let confusion: f64 = precision::confusion();
    let mut v1 = Vertex::default();
    builder.make_vertex_at(&mut v1, plane_point(p1)?, confusion)?;
    if p1.distance(p2) < confusion {
        return Ok((v1.clone(), v1));
    }
    let mut v2 = Vertex::default();
    builder.make_vertex_at(&mut v2, plane_point(p2)?, confusion)?;
    Ok((v1, v2))
}

impl BRepLibMakeEdge2d {
    /// `Init(C)`: the whole curve; `maker.init_curve(&c)?`.
    pub fn init_curve(&mut self, curve: &Handle<dyn Geom2dCurve>) -> Result<(), Failure> {
        let c: &dyn Geom2dCurve = checked_curve(curve)?;
        let (first, last): (f64, f64) = (c.first_parameter(), c.last_parameter());
        self.init_curve_params(curve, first, last)
    }

    /// `Init(C, p1, p2)`: the curve between two parameters;
    /// `maker.init_curve_params(&c, 0.0, 1.0)?`.
    pub fn init_curve_params(
        &mut self,
        curve: &Handle<dyn Geom2dCurve>,
        p1: f64,
        p2: f64,
    ) -> Result<(), Failure> {
        let null = Vertex::default();
        self.init_curve_vertices_params(curve, &null, &null, p1, p2)
    }

    /// `Init(C, P1, P2)`: the curve between the projections of two 2d points, on
    /// vertices at their plane points; `maker.init_curve_points(&c, a, b)?`.
    pub fn init_curve_points(
        &mut self,
        curve: &Handle<dyn Geom2dCurve>,
        p1: Point2,
        p2: Point2,
    ) -> Result<(), Failure> {
        let (v1, v2) = plane_vertex_pair(p1, p2)?;
        self.init_curve_vertices(curve, &v1, &v2)
    }

    /// `Init(C, V1, V2)`: the curve between the projections of the vertices (their
    /// plane parameters); `BRepLib_PointProjectionFailed` when Extrema fails;
    /// `maker.init_curve_vertices(&c, &v1, &v2)?`.
    pub fn init_curve_vertices(
        &mut self,
        curve: &Handle<dyn Geom2dCurve>,
        v1: &Vertex,
        v2: &Vertex,
    ) -> Result<(), Failure> {
        let Some(p1) = self.projected(curve, v1, true)? else {
            return Ok(());
        };
        let Some(p2) = self.projected(curve, v2, false)? else {
            return Ok(());
        };
        self.init_curve_vertices_params(curve, v1, v2, p1, p2)
    }

    /// The parameter of `vertex`, the curve end for a null one; None after recording
    /// `BRepLib_PointProjectionFailed`.
    fn projected(
        &mut self,
        curve: &Handle<dyn Geom2dCurve>,
        vertex: &Vertex,
        first: bool,
    ) -> Result<Option<f64>, Failure> {
        if vertex.is_null() {
            let c: &dyn Geom2dCurve = checked_curve(curve)?;
            return Ok(Some(if first {
                c.first_parameter()
            } else {
                c.last_parameter()
            }));
        }
        let parameter: Option<f64> = project_on_curve2d(curve, vertex)?;
        if parameter.is_none() {
            self.my_error = BRepLibEdgeError::PointProjectionFailed;
        }
        Ok(parameter)
    }

    /// `Init(C, P1, P2, p1, p2)`: the curve between two parameters, on vertices at the
    /// plane points of two 2d points; `maker.init_curve_points_params(&c, a, b, 0.0, 1.0)?`.
    pub fn init_curve_points_params(
        &mut self,
        curve: &Handle<dyn Geom2dCurve>,
        point1: Point2,
        point2: Point2,
        p1: f64,
        p2: f64,
    ) -> Result<(), Failure> {
        let (v1, v2) = plane_vertex_pair(point1, point2)?;
        self.init_curve_vertices_params(curve, &v1, &v2, p1, p2)
    }

    /// `Init(C, V1, V2, p1, p2)`, the member that builds the edge: the curve (trimmed
    /// curves replaced by their basis) on `BRepLib::Plane()`, with the range and
    /// vertex checks of `BRepLib_MakeEdge` at Precision::Confusion(). A vertex of an
    /// open curve is not checked against its end; a success does not set `Error()`;
    /// `maker.init_curve_vertices_params(&c, &v1, &v2, 0.0, 1.0)?`.
    pub fn init_curve_vertices_params(
        &mut self,
        curve: &Handle<dyn Geom2dCurve>,
        vv1: &Vertex,
        vv2: &Vertex,
        pp1: f64,
        pp2: f64,
    ) -> Result<(), Failure> {
        let c: Handle<dyn Geom2dCurve> = untrimmed(curve);
        let basis: &dyn Geom2dCurve = checked_curve(&c)?;
        let bounds: (f64, f64) = (basis.first_parameter(), basis.last_parameter());
        let (periodic, confusion) = (basis.is_periodic(), precision::confusion());
        let ordered = ordered_range(periodic, bounds, (pp1, pp2), (vv1, vv2), confusion);
        let mut range: EdgeRange = match ordered {
            Ok(range) => range,
            Err(error) => return self.record(error),
        };
        match plane_ends(basis, &mut range)? {
            Ok(()) => self.build_edge(&c, &range),
            Err(error) => self.record(error),
        }
    }

    /// The edge whose only representation is `c` on `BRepLib::Plane()`, with the
    /// vertices of `range`; the maker is done.
    fn build_edge(
        &mut self,
        c: &Handle<dyn Geom2dCurve>,
        range: &EdgeRange,
    ) -> Result<(), Failure> {
        let builder = BRepBuilder::default();
        self.my_vertex1 = Vertex::unchecked(range.v1.oriented(Orientation::Forward));
        self.my_vertex2 = Vertex::unchecked(range.v2.oriented(Orientation::Reversed));
        let mut edge = Edge::unchecked(self.my_shape.clone());
        builder.make_edge(&mut edge)?;
        self.my_shape = edge.clone().into_shape();
        let plane: Handle<dyn GeomSurface> = globals::plane().cast();
        let location = Location::default();
        builder.update_edge_pcurve(&edge, c, &plane, &location, precision::confusion(), None)?;
        for vertex in [&self.my_vertex1, &self.my_vertex2] {
            if !vertex.is_null() {
                builder.add(&mut edge, vertex)?;
            }
        }
        builder.range(&edge, range.p1, range.p2, false)?;
        self.set_done();
        Ok(())
    }
}

/// The plane points of the curve ends, the closed flag (2d distance within
/// Precision::Confusion()) and the vertex checks.
fn plane_ends(basis: &dyn Geom2dCurve, range: &mut EdgeRange) -> Step<()> {
    let p1inf: bool = precision::is_negative_infinite(range.p1);
    let p2inf: bool = precision::is_positive_infinite(range.p2);
    let point1: Option<Point2> = if p1inf {
        None
    } else {
        Some(basis.value(range.p1)?)
    };
    let point2: Option<Point2> = if p2inf {
        None
    } else {
        Some(basis.value(range.p2)?)
    };
    let preci: f64 = precision::confusion();
    if let (Some(a), Some(b)) = (point1, point2)
        && a.distance(b) <= preci
    {
        return closed_plane_vertices(range, a, preci);
    }
    if let Err(error) = open_plane_end(&mut range.v1, point1, preci)? {
        return Ok(Err(error));
    }
    open_plane_end(&mut range.v2, point2, preci)
}

/// The vertices of a closed curve: one new vertex for none, the given one for one;
/// two must be the same vertex within `preci` of the plane point of `p1`.
fn closed_plane_vertices(range: &mut EdgeRange, p1: Point2, preci: f64) -> Step<()> {
    if range.v1.is_null() && range.v2.is_null() {
        BRepBuilder::default().make_vertex_at(&mut range.v1, plane_point(p1)?, preci)?;
        range.v2 = range.v1.clone();
    } else if range.v1.is_null() {
        range.v1 = range.v2.clone();
    } else if range.v2.is_null() {
        range.v2 = range.v1.clone();
    } else if !range.v1.is_same(&range.v2)
        || plane_point(p1)?.distance(&BRepTool::pnt(&range.v1)?) > preci
    {
        return Ok(Err(BRepLibEdgeError::DifferentPointsOnClosedCurve));
    }
    Ok(Ok(()))
}

/// One end of an open curve: a vertex at an infinite end is an error, a finite end
/// without vertex gets one at its plane point (the point is computed in any case).
fn open_plane_end(vertex: &mut Vertex, end: Option<Point2>, preci: f64) -> Step<()> {
    let Some(point) = end else {
        if !vertex.is_null() {
            return Ok(Err(BRepLibEdgeError::PointWithInfiniteParameter));
        }
        return Ok(Ok(()));
    };
    let on_plane = plane_point(point)?;
    if vertex.is_null() {
        BRepBuilder::default().make_vertex_at(vertex, on_plane, preci)?;
    }
    Ok(Ok(()))
}
