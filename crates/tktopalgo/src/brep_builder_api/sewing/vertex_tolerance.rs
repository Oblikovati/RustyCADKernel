// Rust translation of the static findNMVertices and both ComputeToleranceVertex
// (BRepBuilderAPI_Sewing.cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The non-manifold vertices of an edge with their parameters, and the vertex that
//! covers two or three merged vertices.
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::topods::{self, Edge, Shape, ShapeIterator, Vertex};
use rustycad_tkernel::ncollection::Sequence;
use rustycad_tkernel::standard::Failure;
use rustycad_tkernel::standard::real::real_epsilon;
use rustycad_tkg2d::geom2d::native::std_min;
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkg3d::geom_adaptor::GeomAdaptorCurve;
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkgeombase::extrema::ExtPC;
use rustycad_tkmath::gp::{Point3, Vector3, Xyz};

/// The native default `TolF` of `Extrema_ExtPC::Initialize`.
const EXTREMA_TOL_F: f64 = 1.0e-10;

/// The parameter of the projection of `point` on the curve: the end nearer to it,
/// unless an extremum is nearer still; None when Extrema fails or finds nothing.
fn nm_parameter(
    projector: &mut ExtPC<'_>,
    ends: ((f64, Point3), (f64, Point3)),
    point: Point3,
) -> Result<Option<f64>, Failure> {
    let ((first, pfirst), (last, plast)) = ends;
    let dist_f2: f64 = pfirst.square_distance(&point);
    let dist_l2: f64 = plast.square_distance(&point);
    let mut par: f64 = if dist_f2 > dist_l2 { last } else { first };
    projector.perform(point)?;
    if !projector.is_done() || projector.nb_ext()? <= 0 {
        return Ok(None);
    }
    let mut dist2_min: f64 = std_min(dist_f2, dist_l2);
    let mut ind_min: i32 = 0;
    for ind in 1..=projector.nb_ext()? {
        let d_proj2: f64 = projector.square_distance(ind)?;
        if d_proj2 < dist2_min {
            ind_min = ind;
            dist2_min = d_proj2;
        }
    }
    if ind_min != 0 {
        par = projector.point(ind_min)?.parameter();
    }
    Ok(Some(par))
}

/// The static `findNMVertices(theEdge, theSeqNMVert, theSeqPars)`: appends the INTERNAL
/// and EXTERNAL vertices of the edge, then the parameter of each on the 3d curve (only
/// for the vertices whose projection is done, so the two sequences may differ in
/// length, as natively); false when the edge has none. Standard_NullObject when the
/// edge has such vertices but no 3d curve.
pub(super) fn find_nm_vertices(
    edge: &Edge,
    seq_nm_vert: &mut Sequence<Shape>,
    seq_pars: &mut Sequence<f64>,
) -> Result<bool, Failure> {
    let mut it = ShapeIterator::new(edge, false, true);
    while let Ok(child) = it.value() {
        if matches!(
            child.orientation(),
            Orientation::Internal | Orientation::External
        ) {
            seq_nm_vert.append(child.clone());
        }
        it.next();
    }
    let nb_v = seq_nm_vert.length();
    if nb_v == 0 {
        return Ok(false);
    }
    let (c3d, first, last) = BRepTool::curve(edge)?;
    let adaptor = GeomAdaptorCurve::with_curve(&c3d)?;
    let mut projector = ExtPC::new();
    projector.initialize(&adaptor, first, last, EXTREMA_TOL_F)?;
    let ends = ((first, adaptor.value(first)?), (last, adaptor.value(last)?));
    for i in 1..=nb_v as usize {
        let vertex = topods::vertex(seq_nm_vert.value(i)?)?;
        let point = BRepTool::pnt(&vertex)?;
        if let Some(par) = nm_parameter(&mut projector, ends, point)? {
            seq_pars.append(par);
        }
    }
    Ok(true)
}

/// The static `ComputeToleranceVertex(theV1, theV2, theNewV)`: a new vertex whose
/// tolerance sphere covers both vertices' spheres (the larger one when it already
/// covers the other).
pub(super) fn compute_tolerance_vertex2(v1: &Vertex, v2: &Vertex) -> Result<Vertex, Failure> {
    let points = [BRepTool::pnt(v1)?, BRepTool::pnt(v2)?];
    let radii = [
        BRepTool::vertex_tolerance(v1)?,
        BRepTool::vertex_tolerance(v2)?,
    ];
    let (m, n) = if radii[0] < radii[1] { (1, 0) } else { (0, 1) };
    let d_r: f64 = radii[m] - radii[n];
    let vd = Vector3::between(points[m], points[n]);
    let d: f64 = vd.magnitude();
    let builder = BRepBuilder::default();
    let mut new_vertex = Vertex::default();
    if d <= d_r || d < real_epsilon() {
        builder.make_vertex_at(&mut new_vertex, points[m], radii[m])?;
        return Ok(new_vertex);
    }
    let rr: f64 = 0.5 * (radii[m] + radii[n] + d);
    let xyz_r: Xyz = 0.5 * (points[m].xyz() + points[n].xyz() - *vd.xyz() * (d_r / d));
    let pr = Point3::new(xyz_r.x(), xyz_r.y(), xyz_r.z());
    builder.make_vertex_at(&mut new_vertex, pr, rr)?;
    Ok(new_vertex)
}

/// The static `ComputeToleranceVertex(theV1, theV2, theV3, theNewV)`: a new vertex at
/// the centroid of the three, whose tolerance reaches the farthest tolerance sphere.
pub(super) fn compute_tolerance_vertex3(vertices: [&Vertex; 3]) -> Result<Vertex, Failure> {
    let mut points = [Point3::default(); 3];
    let mut radii = [0.0; 3];
    let mut xyz = Xyz::new(0.0, 0.0, 0.0);
    for i in 0..3 {
        points[i] = BRepTool::pnt(vertices[i])?;
        radii[i] = BRepTool::vertex_tolerance(vertices[i])?;
        xyz += points[i].xyz();
    }
    xyz /= 3.0;
    let center = Point3::new(xyz.x(), xyz.y(), xyz.z());
    let mut d_max: f64 = -1.0;
    for i in 0..3 {
        let mut di: f64 = center.distance(&points[i]);
        di += radii[i];
        if di > d_max {
            d_max = di;
        }
    }
    let mut new_vertex = Vertex::default();
    BRepBuilder::default().make_vertex_at(&mut new_vertex, center, d_max)?;
    Ok(new_vertex)
}
