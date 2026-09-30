// Rust translation of BRepLib::UpdateInnerTolerances (BRepLib.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepLib::UpdateInnerTolerances`: edge and vertex tolerances enlarged to cover the
//! distances between the 3d curve and the curves on the faces of each edge.
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::brep_adaptor::BRepAdaptorCurve;
use rustycad_tkbrep::topexp;
use rustycad_tkbrep::topods::{Edge, Face, Shape, Vertex};
use rustycad_tkbrep::toptools::IndexedDataMapOfShapeListOfShape;
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::standard::Failure;
use rustycad_tkernel::standard::real::epsilon;
use rustycad_tkg2d::geom2d::native::std_max;
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::gp::Point3;

/// A distance enlarged by twice its epsilon.
fn padded(distance: f64) -> f64 {
    distance + 2. * epsilon(distance)
}

/// `BRepLib::UpdateInnerTolerances(S)`: for every geometric edge of the shape, enlarges
/// the edge tolerance to the largest distance between its 3d curve and its curves on
/// its faces (23 parameters apart for a SameParameter edge, the ends otherwise) and the
/// vertex tolerances to the distances to the curve ends;
/// `brep_lib::update_inner_tolerances(&shape)?`.
pub fn update_inner_tolerances(shape: &Shape) -> Result<(), Failure> {
    let mut ef_map = IndexedDataMapOfShapeListOfShape::default();
    topexp::map_shapes_and_ancestors(shape, ShapeEnum::Edge, ShapeEnum::Face, &mut ef_map)?;
    for index in 1..=ef_map.extent() {
        let edge = Edge::unchecked(ef_map.find_key(index)?.clone());
        if !BRepTool::is_geometric_edge(&edge)? {
            continue;
        }
        update_edge(&edge, ef_map.find_from_index(index)?)?;
    }
    Ok(())
}

/// The update of one edge and its vertices.
fn update_edge(edge: &Edge, faces: &List<Shape>) -> Result<(), Failure> {
    let (v1, v2) = topexp::vertices(edge, false);
    let (fpar, lpar) = BRepTool::range(edge)?;
    let curve = BRepAdaptorCurve::with_edge(edge)?;
    let pnt1: Point3 = if v1.is_null() {
        Point3::default()
    } else {
        BRepTool::pnt(&v1)?
    };
    let pnt2: Point3 = if v2.is_null() {
        Point3::default()
    } else {
        BRepTool::pnt(&v2)?
    };
    let builder = BRepBuilder::default();
    if !BRepTool::degenerated(edge)? && faces.extent() > 0 {
        let max_dist: f64 =
            sample_curves(edge, faces, &curve, (fpar, lpar), (&v1, pnt1), (&v2, pnt2))?;
        builder.update_edge_tolerance(edge, max_dist)?;
    }
    let tol_edge: f64 = BRepTool::edge_tolerance(edge)?;
    if !v1.is_null() {
        let dist1: f64 = padded(pnt1.distance(&curve.value(fpar)?));
        builder.update_vertex_tolerance(&v1, std_max(dist1, tol_edge))?;
    }
    if !v2.is_null() {
        let dist2: f64 = padded(pnt2.distance(&curve.value(lpar)?));
        builder.update_vertex_tolerance(&v2, std_max(dist2, tol_edge))?;
    }
    Ok(())
}

/// The largest padded distance between the 3d curve and the curves on the faces, the
/// vertex tolerances enlarged on the way to the first and last points of each curve.
fn sample_curves(
    edge: &Edge,
    faces: &List<Shape>,
    curve: &BRepAdaptorCurve,
    (fpar, lpar): (f64, f64),
    (v1, pnt1): (&Vertex, Point3),
    (v2, pnt2): (&Vertex, Point3),
) -> Result<f64, Failure> {
    let same_parameter: bool = BRepTool::same_parameter(edge)?;
    let nb_samples: i32 = if same_parameter { 23 } else { 2 };
    let delta: f64 = (lpar - fpar) / f64::from(nb_samples - 1);
    let builder = BRepBuilder::default();
    // theRep: the curves on the faces, all adapted before the sampling
    let mut reps: Vec<BRepAdaptorCurve> = Vec::new();
    for face in faces.iter() {
        reps.push(BRepAdaptorCurve::with_edge_on_face(
            edge,
            &Face::unchecked(face.clone()),
        )?);
    }
    let mut max_dist: f64 = 0.;
    for on_surface in &reps {
        for k in 0..=nb_samples {
            let param_on_center: f64 = if k == nb_samples {
                lpar
            } else {
                fpar + f64::from(k) * delta
            };
            let center: Point3 = curve.value(param_on_center)?;
            let param_on_curve: f64 = match (same_parameter, k) {
                (true, _) => param_on_center,
                (false, 0) => on_surface.first_parameter()?,
                (false, _) => on_surface.last_parameter()?,
            };
            let point: Point3 = on_surface.value(param_on_curve)?;
            // aDist *= 1.1;
            max_dist = std_max(max_dist, padded(center.distance(&point)));
            // Update tolerances of vertices
            if k == 0 && !v1.is_null() {
                builder.update_vertex_tolerance(v1, padded(pnt1.distance(&point)))?;
            }
            if k == nb_samples && !v2.is_null() {
                builder.update_vertex_tolerance(v2, padded(pnt2.distance(&point)))?;
            }
        }
    }
    Ok(max_dist)
}
