//! `MergeSubSeq` of ShapeUpgrade_UnifySameDomain.cxx: a chain of edges replaced by one
//! edge: a degenerated edge, a line segment, a circular arc or (with `ConcatBSplines`)
//! the concatenation of B-spline and Bezier curves.
use super::geometry::is_linear;
use super::glue_edges::glue_edges_with_3d_curves;
use super::seams::find_closest_points;
use super::{ShapeUpgradeUnifySameDomain, UnifyState};
use crate::shape_analysis::ShapeAnalysisEdge;
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::brep_adaptor::BRepAdaptorCurve;
use rustycad_tkbrep::brep_tools::ReShape;
use rustycad_tkbrep::topexp;
use rustycad_tkbrep::topods::{self, Edge, Shape, Vertex};
use rustycad_tkbrep::toptools::IndexedDataMapOfShapeListOfShape;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkg3d::geom::{
    GeomBSplineCurve, GeomBezierCurve, GeomCircle, GeomConic, GeomCurve, GeomLine, GeomTrimmedCurve,
};
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkgeombase::gc::{MakeCircle, MakeLine2d, Root};
use rustycad_tkmath::gp::{Axis1, Axis2, Direction3, Vector3};
use std::f64::consts::PI;

/// The basis of a trimmed curve.
fn untrimmed(curve: Handle<dyn GeomCurve>) -> Handle<dyn GeomCurve> {
    match curve.cast::<GeomTrimmedCurve>().get() {
        Some(trimmed) => trimmed.basis_curve(),
        None => curve,
    }
}

/// What the pairs of the chain allow.
struct ChainKinds {
    lines: bool,
    circles: bool,
}

impl UnifyState {
    /// `MergeSubSeq(theChain, theVFmap, OutEdge)`: the merged edge, None when the chain
    /// cannot be merged.
    pub(super) fn merge_sub_seq(
        &self,
        chain: &[Shape],
        vf_map: &IndexedDataMapOfShapeListOfShape,
    ) -> Result<Option<Edge>, Failure> {
        let mut kinds = ChainKinds {
            lines: true,
            circles: true,
        };
        for pair in chain.windows(2) {
            let (edge1, edge2) = (topods::edge(&pair[0])?, topods::edge(&pair[1])?);
            if BRepTool::degenerated(&edge1)? && BRepTool::degenerated(&edge2)? {
                return degenerated_union(chain, vf_map);
            }
            if !pair_kinds(&edge1, &edge2, self.ang_tol, &mut kinds)? {
                return Ok(None);
            }
        }
        if kinds.lines && kinds.circles {
            return Ok(None);
        }
        if kinds.lines {
            return self.union_of_lines(chain).map(Some);
        }
        if kinds.circles {
            return self.union_of_circles(chain);
        }
        if chain.len() > 1 && self.concat_bsplines && all_splines(chain)? {
            // "skl for bug 0020052 from Mantis: perform such unions only if curves are
            // bspline or bezier".
            let analysis = ShapeAnalysisEdge::new();
            let first = analysis.first_vertex(&topods::edge(&chain[0])?);
            let last = analysis.last_vertex(&topods::edge(&chain[chain.len() - 1])?);
            return glue_edges_with_3d_curves(chain, &first, &last).map(Some);
        }
        Ok(None)
    }

    /// The end vertices of the chain, copied into the context in safe input mode (or
    /// their recorded images).
    fn safe_vertices(&self, mut vertices: [Vertex; 2]) -> Result<[Vertex; 2], Failure> {
        if !self.safe_input_mode {
            return Ok(vertices);
        }
        let context = self.context()?;
        for vertex in vertices.iter_mut() {
            if !context.is_recorded(vertex.as_ref()) {
                let copy = topods::vertex(&vertex.as_ref().empty_copied()?)?;
                context.replace(vertex.as_ref(), copy.as_ref())?;
                *vertex = copy;
            } else {
                *vertex = topods::vertex(&context.apply(vertex.as_ref(), ShapeEnum::Shape)?)?;
            }
        }
        Ok(vertices)
    }

    /// "union of lines is possible": the segment between the end vertices.
    fn union_of_lines(&self, chain: &[Shape]) -> Result<Edge, Failure> {
        let analysis = ShapeAnalysisEdge::new();
        let v0 = analysis.first_vertex(&topods::edge(&chain[0])?);
        let p1 = BRepTool::pnt(&v0)?;
        let v1 = analysis.last_vertex(&topods::edge(&chain[chain.len() - 1])?);
        let p2 = BRepTool::pnt(&v1)?;
        let vector = Vector3::between(p1, p2);
        let [v0, v1] = self.safe_vertices([v0, v1])?;
        let line: Handle<dyn GeomCurve> = Handle::new(GeomLine::new(Axis1::new(
            p1,
            Direction3::from_vector(vector)?,
        )))
        .cast();
        let dist = p1.distance(&p2);
        let trimmed: Handle<dyn GeomCurve> =
            Handle::new(GeomTrimmedCurve::new(&line, 0.0, dist, true, true)?).cast();
        let edge = edge_on_curve(&trimmed, [(&v0, 0.), (&v1, dist)], true)?;
        self.union_pcurves(chain, &edge)?;
        Ok(edge)
    }

    /// "union of circles": the whole circle of a closed chain (or the circle through
    /// three points of the first edge), else the arc between the end vertices.
    fn union_of_circles(&self, chain: &[Shape]) -> Result<Option<Edge>, Failure> {
        let first_edge = topods::edge(&chain[0])?;
        let (c3d, _, _) = BRepTool::curve(&first_edge)?;
        let circle = untrimmed(c3d).cast::<GeomCircle>();
        let analysis = ShapeAnalysisEdge::new();
        let mut v0 = analysis.first_vertex(&first_edge);
        let mut v1 = analysis.last_vertex(&topods::edge(&chain[chain.len() - 1])?);
        let mut closed = v0.is_same(&v1);
        if !closed {
            // "additionally check the points for equality to make a final decision about
            // closedness of the result curve".
            let (p0, p1) = (BRepTool::pnt(&v0)?, BRepTool::pnt(&v1)?);
            let tol = BRepTool::vertex_tolerance(&v0)?.max(BRepTool::vertex_tolerance(&v1)?);
            if p0.square_distance(&p1) < tol * tol {
                closed = true;
                v1 = v0.clone();
                v1.reverse();
            }
        }
        let edge = if closed {
            match closed_circle(&first_edge, &circle, [&v0, &v1])? {
                Some(edge) => edge,
                None => return Ok(None),
            }
        } else {
            let first_parameter = BRepTool::parameter(&v0, &first_edge)?;
            let last_on_first = analysis.last_vertex(&first_edge);
            let last_parameter = BRepTool::parameter(&last_on_first, &first_edge)?;
            [v0, v1] = self.safe_vertices([v0, v1])?;
            open_arc(
                &first_edge,
                &circle,
                [&v0, &v1],
                (first_parameter, last_parameter),
            )?
        };
        self.union_pcurves(chain, &edge)?;
        Ok(Some(edge))
    }
}

/// The lines and circles flags of one pair; false when a 3D curve is missing.
fn pair_kinds(
    edge1: &Edge,
    edge2: &Edge,
    ang_tol: f64,
    kinds: &mut ChainKinds,
) -> Result<bool, Failure> {
    let (c3d1, _, _) = BRepTool::curve(edge1)?;
    let (c3d2, _, _) = BRepTool::curve(edge2)?;
    let curve1 = BRepAdaptorCurve::with_edge(edge1)?;
    let curve2 = BRepAdaptorCurve::with_edge(edge2)?;
    if c3d1.is_null() || c3d2.is_null() {
        return Ok(false);
    }
    let (c3d1, c3d2) = (untrimmed(c3d1), untrimmed(c3d2));
    match is_linear(&curve1)? {
        Some(d1) => match is_linear(&curve2)? {
            Some(d2) if d1.is_parallel(d2, ang_tol) => {}
            _ => kinds.lines = false,
        },
        None => kinds.lines = false,
    }
    let (circle1, circle2) = (c3d1.cast::<GeomCircle>(), c3d2.cast::<GeomCircle>());
    match (circle1.get(), circle2.get()) {
        (Some(a), Some(b)) => {
            if a.location().distance(&b.location()) > precision::confusion() {
                kinds.circles = false;
            }
        }
        _ => kinds.circles = false,
    }
    Ok(true)
}

/// Whether the 3D curve of every edge of the chain having one is a B-spline or a Bezier
/// curve (`NeedUnion`).
fn all_splines(chain: &[Shape]) -> Result<bool, Failure> {
    for shape in chain {
        let found = BRepTool::curve_with_location(&topods::edge(shape)?)?;
        if found.curve.is_null() {
            continue;
        }
        let curve = untrimmed(found.curve);
        if curve.cast::<GeomBSplineCurve>().is_null() && curve.cast::<GeomBezierCurve>().is_null() {
            return Ok(false);
        }
    }
    Ok(true)
}

/// A new edge on the curve with the two vertices, at the given parameters when `update`.
fn edge_on_curve(
    curve: &Handle<dyn GeomCurve>,
    vertices: [(&Vertex, f64); 2],
    update: bool,
) -> Result<Edge, Failure> {
    let builder = BRepBuilder::default();
    let mut edge = Edge::default();
    builder.make_edge_with_curve(&mut edge, curve, precision::confusion())?;
    for (vertex, _) in vertices {
        builder.add(&mut edge, vertex)?;
    }
    if update {
        for (vertex, parameter) in vertices {
            builder.update_vertex_on_edge(vertex, parameter, &edge, 0.)?;
        }
    }
    Ok(edge)
}

/// The closed chain: the circle itself when the first edge starts at parameter 0, else
/// the circle through 3 points of the first edge; None when that circle fails.
fn closed_circle(
    first_edge: &Edge,
    circle: &Handle<GeomCircle>,
    [v0, v1]: [&Vertex; 2],
) -> Result<Option<Edge>, Failure> {
    let adaptor = BRepAdaptorCurve::with_edge(first_edge)?;
    let (mut fp, mut lp) = (adaptor.first_parameter()?, adaptor.last_parameter()?);
    if first_edge.orientation() != Orientation::Forward {
        std::mem::swap(&mut fp, &mut lp);
    }
    if fp.abs() < precision::p_confusion() {
        let mut edge = edge_on_curve(&circle.clone().cast(), [(v0, 0.), (v1, 0.)], false)?;
        edge.set_orientation(first_edge.orientation());
        return Ok(Some(edge));
    }
    let points = (
        adaptor.value(fp)?,
        adaptor.value((fp + lp) * 0.5)?,
        adaptor.value(lp)?,
    );
    let maker = MakeCircle::through_points(points.0, points.1, points.2)?;
    if !maker.is_done() {
        return Ok(None);
    }
    Ok(Some(edge_on_curve(
        &maker.value()?.cast(),
        [(v0, 0.), (v1, 0.)],
        false,
    )?))
}

/// The open chain: the arc of the circle from the first vertex to the last one, in the
/// plane of the first vertex and a point of the first edge less than 7/8 pi away.
fn open_arc(
    first_edge: &Edge,
    circle: &Handle<GeomCircle>,
    [v0, v1]: [&Vertex; 2],
    (first, mut last): (f64, f64),
) -> Result<Edge, Failure> {
    let circle = circle.get().ok_or_else(super::geometry::null_surface)?;
    let point_first = BRepTool::pnt(v0)?;
    while (last - first).abs() > 7. * PI / 8. {
        last = (first + last) / 2.;
    }
    let point_last = BRepAdaptorCurve::with_edge(first_edge)?.value(last)?;
    let origin = circle.circ()?.location();
    let dir1 = Direction3::from_vector(Vector3::between(origin, point_first))?;
    let dir2 = Direction3::from_vector(Vector3::between(origin, point_last))?;
    let normal = dir1.crossed(dir2)?;
    let new_circle = GeomCircle::new(
        Axis2::with_x_direction(origin, normal, dir1)?,
        circle.radius(),
    )?;
    let last_in_chain = Direction3::from_vector(Vector3::between(origin, BRepTool::pnt(v1)?))?;
    let mut lpar = dir1.angle_with_ref(last_in_chain, normal);
    if lpar < 0. {
        lpar += 2. * PI;
    }
    let new_circle: Handle<dyn GeomCurve> = Handle::new(new_circle).cast();
    let trimmed: Handle<dyn GeomCurve> =
        Handle::new(GeomTrimmedCurve::new(&new_circle, 0., lpar, true, true)?).cast();
    edge_on_curve(&trimmed, [(v0, 0.), (v1, lpar)], true)
}

/// Two degenerated edges: the degenerated edge between the far ends of the chain on the
/// common face; None when the ends have no common face.
fn degenerated_union(
    chain: &[Shape],
    vf_map: &IndexedDataMapOfShapeListOfShape,
) -> Result<Option<Edge>, Failure> {
    let first = topods::edge(&chain[0])?;
    let last = topods::edge(&chain[chain.len() - 1])?;
    let Some(c) = find_closest_points(&first, &last, vf_map)? else {
        return Ok(None);
    };
    // "Define indices corresponding to extremities of future edge".
    let (ind1, ind2) = (1 - c.ind1, 1 - c.ind2);
    let (mut start, mut end) = (c.points1[ind1], c.points2[ind2]);
    if (c.or1 == Orientation::Forward && ind1 == 1) || (c.or1 == Orientation::Reversed && ind1 == 0)
    {
        std::mem::swap(&mut start, &mut end);
    }
    let line: Handle<dyn Geom2dCurve> = MakeLine2d::from_points(start, end)?.value()?.cast();
    let vertex = topexp::first_vertex(&first, false);
    let (mut start_vertex, mut end_vertex) = (vertex.clone(), vertex);
    start_vertex.set_orientation(Orientation::Forward);
    end_vertex.set_orientation(Orientation::Reversed);
    let builder = BRepBuilder::default();
    let mut edge = Edge::default();
    builder.make_edge(&mut edge)?;
    builder.update_edge_pcurve_on_face(&edge, &line, &c.face, precision::confusion())?;
    builder.range(&edge, 0., start.distance(end), false)?;
    builder.add(&mut edge, &start_vertex)?;
    builder.add(&mut edge, &end_vertex)?;
    builder.degenerated(&edge, true)?;
    Ok(Some(edge))
}

impl ShapeUpgradeUnifySameDomain {
    /// `MergeSubSeq(theChain, theVFmap, OutEdge)` (protected): the edge replacing the
    /// chain, None when it cannot be merged;
    /// `let merged = unifier.merge_sub_seq(&chain, &vf_map)?;`.
    pub fn merge_sub_seq(
        &self,
        chain: &[Shape],
        vf_map: &IndexedDataMapOfShapeListOfShape,
    ) -> Result<Option<Edge>, Failure> {
        self.state.read().merge_sub_seq(chain, vf_map)
    }
}
