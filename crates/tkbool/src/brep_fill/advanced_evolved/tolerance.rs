// Rust translation of the static ContainsInList, FindInternals, RemoveInternalWires,
// ProcessVertex and ReduceVertexTolerance
// (ModelingAlgorithms/TKBool/BRepFill/BRepFill_AdvancedEvolved.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The shape helpers of the advanced evolved: list membership, the internal wires
//! removed from the faces and the vertex tolerances reduced to their real gaps.
use rustycad_tkbrep::brep::access::{edge_tshape, vertex_tshape};
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::topexp::{self, Explorer};
use rustycad_tkbrep::topods::{self, Edge, Shape, ShapeIterator, Vertex};
use rustycad_tkbrep::toptools::IndexedDataMapOfShapeListOfShape;
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkmath::gp::Point3;

/// `ContainsInList(theL, theObject)`: whether the list holds the same shape.
pub(super) fn contains_in_list(the_l: &List<Shape>, the_object: &Shape) -> bool {
    the_l.iter().any(|s| s.is_same(the_object))
}

/// `FindInternals(theS, theLInt)`: the internal sub-shapes of the shape, and those
/// holding an internal sub-shape.
fn find_internals(the_s: &Shape) -> Result<Vec<Shape>, Failure> {
    let mut the_l_int: Vec<Shape> = Vec::new();
    let mut it_s = ShapeIterator::new(the_s, true, true);
    while it_s.more() {
        let a_ss: Shape = it_s.value()?.clone();
        it_s.next();
        if a_ss.orientation() == Orientation::Internal {
            the_l_int.push(a_ss);
            continue;
        }
        let mut it_ss = ShapeIterator::new(&a_ss, true, true);
        while it_ss.more() {
            if it_ss.value()?.orientation() == Orientation::Internal {
                the_l_int.push(a_ss.clone());
                break;
            }
            it_ss.next();
        }
    }
    Ok(the_l_int)
}

/// `RemoveInternalWires(theShape)`: the internal wires (and the wires holding an
/// internal edge) removed from every face, in place.
pub(super) fn remove_internal_wires(the_shape: &Shape) -> Result<(), Failure> {
    let mut an_exp_f = Explorer::new(the_shape, ShapeEnum::Face, ShapeEnum::Shape)?;
    while an_exp_f.more() {
        let mut a_f: Shape = an_exp_f.current().clone();
        an_exp_f.next()?;
        let a_lw_to_remove = find_internals(&a_f)?;
        if a_lw_to_remove.is_empty() {
            continue;
        }
        a_f.set_free(true)?;
        for w in &a_lw_to_remove {
            BRepBuilder::default().remove(&mut a_f, w)?;
        }
        a_f.set_free(false)?;
    }
    Ok(())
}

/// The greatest square distance from the vertex point to the points of its edges at
/// it (their 3d curves and p-curves); -1e6 without any.
fn max_square_gap(a_v: &Vertex, a_le: &List<Shape>, a_pv3d: Point3) -> Result<f64, Failure> {
    let mut a_tol_max2: f64 = -1.0e6;
    for e in a_le.iter() {
        let a_e: Edge = topods::edge(e)?;
        let te = edge_tshape(&a_e, "BRepFill_AdvancedEvolved::ProcessVertex")?;
        let mut a_v_exp = Explorer::new(a_e.as_ref(), ShapeEnum::Vertex, ShapeEnum::Shape)?;
        while a_v_exp.more() {
            let a_vx: Vertex = topods::vertex(a_v_exp.current())?;
            a_v_exp.next()?;
            let an_or_v = a_vx.orientation();
            if !a_vx.is_same(a_v)
                || (an_or_v != Orientation::Forward && an_or_v != Orientation::Reversed)
            {
                continue;
            }
            for cr in te.curves() {
                if let Some(d2) = representation_gap(&cr, (&a_e, &a_vx, a_v), a_pv3d)? {
                    a_tol_max2 = a_tol_max2.max(d2);
                }
            }
        }
    }
    Ok(a_tol_max2)
}

/// The square distance to the point of one curve representation of the edge at the
/// vertex (3d curve or p-curve), `None` for another representation or a null curve.
fn representation_gap(
    cr: &rustycad_tkernel::standard::Handle<dyn rustycad_tkbrep::brep::BRepCurveRepresentation>,
    (a_e, a_vx, a_v): (&Edge, &Vertex, &Vertex),
    a_pv3d: Point3,
) -> Result<Option<f64>, Failure> {
    let Some(rep) = cr.get() else { return Ok(None) };
    let l = a_e
        .location()
        .multiplied(&rep.location())?
        .predivided(a_v.location())?;
    if rep.is_curve3d() {
        let a_c3d = rep.curve3d()?;
        let Some(c) = a_c3d.get() else {
            return Ok(None);
        };
        let a_param: f64 = BRepTool::parameter(a_vx, a_e)?;
        let a_pc3d = c.value(a_param)?.transformed(l.transformation());
        return Ok(Some(a_pv3d.square_distance(&a_pc3d)));
    }
    if !rep.is_curve_on_surface() {
        return Ok(None);
    }
    let a_c2d = rep.pcurve()?;
    let Some(c2d) = a_c2d.get() else {
        return Ok(None);
    };
    let a_s = rep.surface()?;
    let a_param: f64 = BRepTool::parameter_on_surface(a_vx, a_e, &a_s, &l)?;
    let a_pc2d = c2d.value(a_param)?;
    let surface = a_s.get().ok_or_else(null_surface)?;
    let a_pc3d = surface
        .value(a_pc2d.x(), a_pc2d.y())?
        .transformed(l.transformation());
    Ok(Some(a_pv3d.square_distance(&a_pc3d)))
}

/// `ProcessVertex(aV, aLE, aLF)`: the vertex tolerance reduced to its real gap (at
/// least the tolerances of its edges and faces, with a margin of 1e-4) when that is
/// smaller.
fn process_vertex(a_v: &Vertex, a_le: &List<Shape>, a_lf: &List<Shape>) -> Result<(), Failure> {
    let tv = vertex_tshape(a_v.as_ref(), "BRepFill_AdvancedEvolved::ProcessVertex")?;
    let a_pv3d: Point3 = tv.pnt();
    let a_tol: f64 = BRepTool::vertex_tolerance(a_v)?;
    let mut a_tol_max2: f64 = max_square_gap(a_v, a_le, a_pv3d)?;
    // Reducing.
    if a_tol_max2 < 0. {
        return Ok(());
    }
    a_tol_max2 = a_tol_max2.sqrt();
    if a_tol_max2 > a_tol {
        return Ok(());
    }
    for e in a_le.iter() {
        a_tol_max2 = a_tol_max2.max(BRepTool::edge_tolerance(&topods::edge(e)?)?);
    }
    for f in a_lf.iter() {
        a_tol_max2 = a_tol_max2.max(BRepTool::face_tolerance(&topods::face(f)?)?);
    }
    if a_tol_max2 > a_tol {
        return Ok(());
    }
    // Update the tolerance with a small margin.
    tv.set_tolerance(a_tol_max2 + a_tol_max2 * 0.0001);
    Ok(())
}

/// `ReduceVertexTolerance(aS)`: the tolerance of every vertex of the shape reduced.
pub(super) fn reduce_vertex_tolerance(a_s: &Shape) -> Result<(), Failure> {
    let (mut a_ve_map, mut a_vf_map) = (
        IndexedDataMapOfShapeListOfShape::default(),
        IndexedDataMapOfShapeListOfShape::default(),
    );
    topexp::map_shapes_and_unique_ancestors(
        a_s,
        ShapeEnum::Vertex,
        ShapeEnum::Edge,
        &mut a_ve_map,
        false,
    )?;
    topexp::map_shapes_and_unique_ancestors(
        a_s,
        ShapeEnum::Vertex,
        ShapeEnum::Face,
        &mut a_vf_map,
        false,
    )?;
    for i in 1..=a_ve_map.extent() {
        let a_v: Vertex = topods::vertex(a_ve_map.find_key(i)?)?;
        let a_le = a_ve_map.find_from_index(i)?;
        let a_lf = a_vf_map.find_from_key(a_v.as_ref())?;
        process_vertex(&a_v, a_le, a_lf)?;
    }
    Ok(())
}

/// Standard_NullObject of a p-curve without surface (natively a crash).
fn null_surface() -> Failure {
    Failure::with_type(
        "Standard_NullObject",
        "BRepFill_AdvancedEvolved: p-curve without surface; expected a surface",
        "BRep_CurveRepresentation",
    )
}
