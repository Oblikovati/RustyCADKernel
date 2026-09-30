// Rust translation of ComputeTangency of
// ModelingAlgorithms/TKGeomAlgo/IntPatch/IntPatch_ImpPrmIntersection.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The start points of the walking on the restrictions of the parametric surface:
//! tangent points, and oriented start points with the transition of a vertex.
use super::super::path_point::IntPatchThePathPointOfTheSOnBounds;
use super::super::{IntPatchTheSOnBounds, IntPatchTheSurfFunction};
use crate::int_start::{TheArc, TheTopolTool, TheVertex};
use crate::int_surf::IntSurfPathPoint;
use crate::toptrans::CurveTransition;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::adaptor3d::h_surface_tool::Adaptor3dSurfaceHandle;
use rustycad_tkg3d::top_abs::{Orientation, State};
use rustycad_tkmath::gp::{Direction2, Direction3, Point2, Vector2, Vector3};
use rustycad_tkmath::math::{Matrix, Vector};

/// The locals of `ComputeTangency` that keep their values from one point to the next.
struct TangencyState {
    pstart: IntPatchThePathPointOfTheSOnBounds,
    ppoint: IntSurfPathPoint,
    vtx: TheVertex,
    vectg: Vector3,
    dirtg: Direction2,
    d1u: Vector3,
    d1v: Vector3,
    v1: Vector3,
    v2: Vector3,
    x: [f64; 2],
    arcorien: Orientation,
    ispassing: bool,
    seqlength: i32,
}

/// The inputs of `ComputeTangency`.
pub(super) struct TangencyInput<'a> {
    pub(super) solrst: &'a IntPatchTheSOnBounds,
    pub(super) domain: &'a (dyn TheTopolTool + 'static),
    pub(super) psurf: &'a Adaptor3dSurfaceHandle,
}

fn is_passing(o: Orientation) -> bool {
    o == Orientation::Internal || o == Orientation::External
}

fn is_oriented(o: Orientation) -> bool {
    o == Orientation::Forward || o == Orientation::Reversed
}

/// `ComputeTangency(solrst, seqpdep, Domain, Func, PSurf, Destination)`: every point of
/// the restrictions not yet used becomes a path point: a tangent one (with the points
/// of the same vertex), or a start point oriented into the domain (new points) or whose
/// vertex transition (TopTrans_CurveTransition over the arcs of the vertex) keeps it;
/// `destination` gets the rank of the path point of each restriction point (negative
/// for a dropped vertex).
pub(super) fn compute_tangency(
    input: &TangencyInput<'_>,
    seqpdep: &mut Vec<IntSurfPathPoint>,
    func: &mut IntPatchTheSurfFunction,
    destination: &mut [i32],
) -> Result<(), Failure> {
    let mut st = TangencyState {
        pstart: IntPatchThePathPointOfTheSOnBounds::new(),
        ppoint: IntSurfPathPoint::new(),
        vtx: TheVertex::default(),
        vectg: Vector3::default(),
        dirtg: Direction2::default(),
        d1u: Vector3::default(),
        d1v: Vector3::default(),
        v1: Vector3::default(),
        v2: Vector3::default(),
        x: [0.0; 2],
        arcorien: Orientation::Forward,
        ispassing: false,
        seqlength: 0,
    };
    let nb_points: i32 = input.solrst.nb_points()?;
    for i in 1..=nb_points {
        if destination[i as usize] != 0 {
            continue;
        }
        start_point(input, &mut st, func, i)?;
        if func.is_tangent()? {
            tangent_point(input, &mut st, seqpdep, destination, i)?;
        } else {
            oriented_point(input, &mut st, seqpdep, func, destination, i)?;
        }
    }
    Ok(())
}

/// The point `i`: its arc, the function at its 2d point.
fn start_point(
    input: &TangencyInput<'_>,
    st: &mut TangencyState,
    func: &mut IntPatchTheSurfFunction,
    i: i32,
) -> Result<(), Failure> {
    st.pstart = input.solrst.point(i)?.clone();
    let thearc: TheArc = st.pstart.arc();
    st.arcorien = input.domain.arc_orientation(&thearc)?;
    st.ispassing = is_passing(st.arcorien);
    let p2d: Point2 = thearc.d0(st.pstart.parameter())?;
    st.x = [p2d.x(), p2d.y()];
    st.ppoint.set_value(st.pstart.value(), st.x[0], st.x[1]);
    let mut x = Vector::with_bounds(1, 2)?;
    (x[1], x[2]) = (st.x[0], st.x[1]);
    let mut f = Vector::with_bounds(1, 1)?;
    let mut d = Matrix::with_bounds(1, 1, 1, 2).map_err(|e| e.into_failure("math_Matrix"))?;
    func.values(&x, &mut f, &mut d)?;
    Ok(())
}

/// A tangent path point with the parameters of the other points of its vertex.
fn tangent_point(
    input: &TangencyInput<'_>,
    st: &mut TangencyState,
    seqpdep: &mut Vec<IntSurfPathPoint>,
    destination: &mut [i32],
    i: i32,
) -> Result<(), Failure> {
    st.ppoint.set_tangency(true);
    destination[i as usize] = st.seqlength + 1;
    if !st.pstart.is_new() {
        st.vtx = st.pstart.vertex()?;
        for k in i + 1..=input.solrst.nb_points()? {
            if destination[k as usize] != 0 {
                continue;
            }
            st.pstart = input.solrst.point(k)?.clone();
            if st.pstart.is_new() || !input.domain.identical(&st.vtx, &st.pstart.vertex()?)? {
                continue;
            }
            let thearc: TheArc = st.pstart.arc();
            st.arcorien = input.domain.arc_orientation(&thearc)?;
            st.ispassing = st.ispassing && is_passing(st.arcorien);
            let p2d: Point2 = thearc.d0(st.pstart.parameter())?;
            st.ppoint.add_uv(p2d.x(), p2d.y())?;
            destination[k as usize] = st.seqlength + 1;
        }
    }
    st.ppoint.set_passing(st.ispassing);
    seqpdep.push(st.ppoint.clone());
    st.seqlength += 1;
    Ok(())
}

/// The tangents of a potential start point and the orientation test against the arc.
fn oriented_point(
    input: &TangencyInput<'_>,
    st: &mut TangencyState,
    seqpdep: &mut Vec<IntSurfPathPoint>,
    func: &mut IntPatchTheSurfFunction,
    destination: &mut [i32],
    i: i32,
) -> Result<(), Failure> {
    st.vectg = func.direction3d()?;
    st.dirtg = func.direction2d()?;
    let d1 = input.psurf.d1(st.x[0], st.x[1])?;
    (st.d1u, st.d1v) = (d1.d1u, d1.d1v);
    let (_, d2d): (Point2, Vector2) = st.pstart.arc().d1(st.pstart.parameter())?;
    st.v2.set_linear_form(d2d.x(), st.d1u, d2d.y(), st.d1v);
    st.v1 = st.d1u.crossed(st.d1v);
    let test: f64 = st.vectg.dot(st.v1.crossed(st.v2));
    if !st.pstart.is_new() {
        return vertex_transition(input, st, seqpdep, destination, (i, test));
    }
    if (test < 0.0 && st.arcorien == Orientation::Forward)
        || (test > 0.0 && st.arcorien == Orientation::Reversed)
    {
        st.vectg.reverse();
        st.dirtg.reverse();
    }
    st.ppoint.set_directions(st.vectg, st.dirtg);
    st.ppoint.set_passing(st.ispassing);
    destination[i as usize] = st.seqlength + 1;
    seqpdep.push(st.ppoint.clone());
    st.seqlength += 1;
    Ok(())
}

/// The local transition of the arc at the vertex from the sign of `test`.
fn local_transition(test: f64, arcorien: Orientation, tole: f64) -> Orientation {
    if test.abs() <= tole {
        return Orientation::External; // and why not INTERNAL
    }
    if (test > 0.0 && arcorien == Orientation::Forward)
        || (test < 0.0 && arcorien == Orientation::Reversed)
    {
        return Orientation::Forward;
    }
    Orientation::Reversed
}

/// Adds the arc of the vertex at the point to the transition of the vertex.
fn compare_arc(
    input: &TangencyInput<'_>,
    st: &mut TangencyState,
    comptrans: &mut CurveTransition,
    (test, vertex): (f64, &TheVertex),
) -> Result<(), Failure> {
    let tole: f64 = 1.0e-8;
    let bidnorm = Direction3::new(1.0, 1.0, 1.0)?;
    let vtxorien: Orientation = input.domain.vertex_orientation(vertex)?;
    let loc_trans: Orientation = local_transition(test, st.arcorien, tole);
    if test.abs() > tole && st.arcorien == Orientation::Reversed {
        st.v2.reverse();
    }
    comptrans.compare(
        tole,
        Direction3::from_vector(st.v2)?,
        bidnorm,
        0.0,
        loc_trans,
        vtxorien,
    );
    Ok(())
}

/// A start point on a vertex: the transitions of every arc of the vertex decide whether
/// the point is kept and how it is oriented.
fn vertex_transition(
    input: &TangencyInput<'_>,
    st: &mut TangencyState,
    seqpdep: &mut Vec<IntSurfPathPoint>,
    destination: &mut [i32],
    (i, test): (i32, f64),
) -> Result<(), Failure> {
    let bidnorm = Direction3::new(1.0, 1.0, 1.0)?;
    let mut comptrans = CurveTransition::new();
    comptrans.reset(Direction3::from_vector(st.vectg)?, bidnorm, 0.0);
    if is_oriented(st.arcorien) {
        st.vtx = st.pstart.vertex()?;
        let vtx = st.vtx.clone();
        compare_arc(input, st, &mut comptrans, (test, &vtx))?;
    }
    destination[i as usize] = st.seqlength + 1;
    same_vertex_arcs(input, st, &mut comptrans, destination, i)?;
    let fairpt: bool = keep_start_point(st, &comptrans);
    if fairpt {
        st.ppoint.set_directions(st.vectg, st.dirtg);
        st.ppoint.set_passing(st.ispassing);
        seqpdep.push(st.ppoint.clone());
        st.seqlength += 1;
        return Ok(());
    }
    // The ranks are put back in order when the point is not kept.
    for k in i as usize..destination.len() - 1 {
        if destination[k] == st.seqlength + 1 {
            destination[k] = -destination[k];
        }
    }
    Ok(())
}

/// The other points of the vertex: their parameters and arcs join the transition.
fn same_vertex_arcs(
    input: &TangencyInput<'_>,
    st: &mut TangencyState,
    comptrans: &mut CurveTransition,
    destination: &mut [i32],
    i: i32,
) -> Result<(), Failure> {
    for k in i + 1..=input.solrst.nb_points()? {
        if destination[k as usize] != 0 {
            continue;
        }
        st.pstart = input.solrst.point(k)?.clone();
        if st.pstart.is_new() || !input.domain.identical(&st.vtx, &st.pstart.vertex()?)? {
            continue;
        }
        let thearc: TheArc = st.pstart.arc();
        st.arcorien = input.domain.arc_orientation(&thearc)?;
        st.ppoint.add_uv(st.x[0], st.x[1])?;
        let (p2d, d2d): (Point2, Vector2) = thearc.d1(st.pstart.parameter())?;
        st.ppoint.add_uv(p2d.x(), p2d.y())?;
        if is_oriented(st.arcorien) {
            st.ispassing = false;
            st.v2.set_linear_form(d2d.x(), st.d1u, d2d.y(), st.d1v);
            let test: f64 = st.vectg.dot(st.v1.crossed(st.v2));
            let vertex: TheVertex = st.pstart.vertex()?;
            compare_arc(input, st, comptrans, (test, &vertex))?;
        }
        destination[k as usize] = st.seqlength + 1;
    }
    Ok(())
}

/// Whether a not passing start point is kept (known states, into the domain after or
/// crossing it); a point going out is reversed.
fn keep_start_point(st: &mut TangencyState, comptrans: &CurveTransition) -> bool {
    if st.ispassing {
        return true;
    }
    let (before, after) = (comptrans.state_before(), comptrans.state_after());
    if before == State::Unknown || after == State::Unknown {
        return false;
    }
    if before == State::In {
        if after == State::In {
            st.ispassing = true;
        } else {
            st.vectg.reverse();
            st.dirtg.reverse();
        }
        return true;
    }
    after == State::In
}
