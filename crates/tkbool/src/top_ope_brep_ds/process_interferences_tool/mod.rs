// Rust translation of
// ModelingAlgorithms/TKBool/TopOpeBRepDS/TopOpeBRepDS_ProcessInterferencesTool.hxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `TopOpeBRepDS_ProcessInterferencesTool`: making point/vertex interferences, and
//! selecting interferences of a list by their transition, geometry or support (the
//! selected interferences are moved to the output list).
pub(crate) mod select;
mod select3d;

pub use select::{
    fun_reducedoublons, fun_select1d_i, fun_select2d_i, fun_select3d_interference,
    fun_selectpure2d_i, fun_unkeep_unknown,
};

use crate::top_ope_brep_ds::export::iref;
use crate::top_ope_brep_ds::{
    ListOfInterference, TopOpeBRepDSConfig, TopOpeBRepDSDataStructure, TopOpeBRepDSInterference,
    TopOpeBRepDSInterferenceTool, TopOpeBRepDSKind, TopOpeBRepDSTransition,
};
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum, State};

/// `MakeCPVInterference(T, SI, GI, P, GK)`: a curve/point interference on the CURVE
/// `si`; `make_cpv_interference(&t, 1, 2, 0.5, TopOpeBRepDSKind::Point)`.
pub fn make_cpv_interference(
    t: &TopOpeBRepDSTransition,
    si: i32,
    gi: i32,
    p: f64,
    gk: TopOpeBRepDSKind,
) -> Handle<TopOpeBRepDSInterference> {
    TopOpeBRepDSInterferenceTool::make_curve_interference(t, TopOpeBRepDSKind::Curve, si, gk, gi, p)
}

/// `MakeEPVInterference(T, SI, GI, P, GK, B)`: on the EDGE `si`, a curve/point
/// interference for a POINT, an edge/vertex one (UNSHGEOMETRY, `b` its GBound) for a
/// VERTEX, a null handle otherwise;
/// `make_epv_interference(&t, 1, 2, 0.5, TopOpeBRepDSKind::Vertex, false)`.
pub fn make_epv_interference(
    t: &TopOpeBRepDSTransition,
    si: i32,
    gi: i32,
    p: f64,
    gk: TopOpeBRepDSKind,
    b: bool,
) -> Handle<TopOpeBRepDSInterference> {
    let sk = TopOpeBRepDSKind::Edge;
    match gk {
        TopOpeBRepDSKind::Point => {
            TopOpeBRepDSInterferenceTool::make_edge_interference(t, sk, si, gk, gi, p)
        }
        TopOpeBRepDSKind::Vertex => {
            let i = TopOpeBRepDSInterferenceTool::make_edge_vertex_interference(
                t,
                si,
                gi,
                b,
                TopOpeBRepDSConfig::UnshGeometry,
                p,
            );
            if let Some(ii) = i.get() {
                ii.set_geometry_type(gk);
            }
            i
        }
        _ => Handle::default(),
    }
}

/// `MakeEPVInterference(T, S, G, P, GK, SK, B)`: the same with the support kind `sk`;
/// Standard_NullObject for a kind neither POINT nor VERTEX (a null dereference
/// natively); `make_epv_interference_sk(&t, 1, 2, 0.5, TopOpeBRepDSKind::Point, TopOpeBRepDSKind::Face, false)?`.
pub fn make_epv_interference_sk(
    t: &TopOpeBRepDSTransition,
    s: i32,
    g: i32,
    p: f64,
    gk: TopOpeBRepDSKind,
    sk: TopOpeBRepDSKind,
    b: bool,
) -> Result<Handle<TopOpeBRepDSInterference>, Failure> {
    let i = make_epv_interference(t, s, g, p, gk, b);
    iref(&i)?.set_support_type(sk);
    Ok(i)
}

/// `FUN_hasStateShape(T, state, shape)`: the state and shape before or after;
/// `fun_has_state_shape(&t, State::In, ShapeEnum::Face)`.
pub fn fun_has_state_shape(t: &TopOpeBRepDSTransition, state: State, shape: ShapeEnum) -> bool {
    (t.before() == state && t.shape_before() == shape)
        || (t.after() == state && t.shape_after() == shape)
}

/// The interferences of `l1` passing `sel` moved to `l2` (in order); the size of `l2`.
pub(crate) fn select_by(
    l1: &mut ListOfInterference,
    l2: &mut ListOfInterference,
    mut sel: impl FnMut(&TopOpeBRepDSInterference) -> Result<bool, Failure>,
) -> Result<i32, Failure> {
    let mut err = None;
    l1.retain(|i| {
        if err.is_some() {
            return true;
        }
        match iref(i).and_then(&mut sel) {
            Ok(true) => {
                l2.append(i.clone());
                false
            }
            Ok(false) => true,
            Err(e) => {
                err = Some(e);
                true
            }
        }
    });
    match err {
        Some(e) => Err(e),
        None => Ok(l2.extent()),
    }
}

/// `FUN_selectTRASHAinterference(L1, sha2, L2)`: the transitions with a shape before
/// or after of type `sha2`; `fun_select_trasha_interference(&mut l1, ShapeEnum::Face, &mut l2)?`.
pub fn fun_select_trasha_interference(
    l1: &mut ListOfInterference,
    sha2: ShapeEnum,
    l2: &mut ListOfInterference,
) -> Result<i32, Failure> {
    select_by(l1, l2, |i| {
        let t = i.transition();
        Ok(t.shape_before() == sha2 || t.shape_after() == sha2)
    })
}

/// `FUN_selectITRASHAinterference(L1, Index, L2)`: the transitions on the shape
/// `index` (none for 0); Standard_Failure for a transition between different shapes;
/// `fun_select_itrasha_interference(&mut l1, 3, &mut l2)?`.
pub fn fun_select_itrasha_interference(
    l1: &mut ListOfInterference,
    index: i32,
    l2: &mut ListOfInterference,
) -> Result<i32, Failure> {
    if index == 0 {
        return Ok(0);
    }
    select_by(l1, l2, |i| Ok(i.transition().index()? == index))
}

/// `FUN_selectTRAUNKinterference(L1, L2)`: the unknown transitions;
/// `fun_select_traunk_interference(&mut l1, &mut l2)?`.
pub fn fun_select_traunk_interference(
    l1: &mut ListOfInterference,
    l2: &mut ListOfInterference,
) -> Result<i32, Failure> {
    select_by(l1, l2, |i| Ok(i.transition().is_unknown()))
}

/// `FUN_selectTRAORIinterference(L1, O, L2)`: the transitions of orientation `o` seen
/// from IN; `fun_select_traori_interference(&mut l1, Orientation::Forward, &mut l2)?`.
pub fn fun_select_traori_interference(
    l1: &mut ListOfInterference,
    o: Orientation,
    l2: &mut ListOfInterference,
) -> Result<i32, Failure> {
    select_by(l1, l2, |i| {
        Ok(i.transition().orientation(State::In, ShapeEnum::Face) == o)
    })
}

/// `FUN_selectGKinterference(L1, GK, L2)`: the geometries of kind `gk`;
/// `fun_select_gk_interference(&mut l1, TopOpeBRepDSKind::Vertex, &mut l2)?`.
pub fn fun_select_gk_interference(
    l1: &mut ListOfInterference,
    gk: TopOpeBRepDSKind,
    l2: &mut ListOfInterference,
) -> Result<i32, Failure> {
    select_by(l1, l2, |i| Ok(i.geometry_type() == gk))
}

/// `FUN_selectSKinterference(L1, SK, L2)`: the supports of kind `sk`;
/// `fun_select_sk_interference(&mut l1, TopOpeBRepDSKind::Face, &mut l2)?`.
pub fn fun_select_sk_interference(
    l1: &mut ListOfInterference,
    sk: TopOpeBRepDSKind,
    l2: &mut ListOfInterference,
) -> Result<i32, Failure> {
    select_by(l1, l2, |i| Ok(i.support_type() == sk))
}

/// `FUN_selectGIinterference(L1, GI, L2)`: the geometry `gi` (none for 0);
/// `fun_select_gi_interference(&mut l1, 2, &mut l2)?`.
pub fn fun_select_gi_interference(
    l1: &mut ListOfInterference,
    gi: i32,
    l2: &mut ListOfInterference,
) -> Result<i32, Failure> {
    if gi == 0 {
        return Ok(0);
    }
    select_by(l1, l2, |i| Ok(i.geometry() == gi))
}

/// `FUN_selectSIinterference(L1, SI, L2)`: the support `si` (none for 0);
/// `fun_select_si_interference(&mut l1, 2, &mut l2)?`.
pub fn fun_select_si_interference(
    l1: &mut ListOfInterference,
    si: i32,
    l2: &mut ListOfInterference,
) -> Result<i32, Failure> {
    if si == 0 {
        return Ok(0);
    }
    select_by(l1, l2, |i| Ok(i.support() == si))
}

/// `FUN_interfhassupport(DS, I, S)`: the support of `i` is the same as `s`;
/// `fun_interfhassupport(&ds, &i, &face)?`.
pub fn fun_interfhassupport(
    ds: &TopOpeBRepDSDataStructure,
    i: &Handle<TopOpeBRepDSInterference>,
    s: &Shape,
) -> Result<bool, Failure> {
    Ok(ds.shape(iref(i)?.support(), true).is_same(s))
}

/// `FUN_transitionEQUAL(T1, T2)`: same states, shapes and indices;
/// `fun_transition_equal(&t1, &t2)`.
pub fn fun_transition_equal(t1: &TopOpeBRepDSTransition, t2: &TopOpeBRepDSTransition) -> bool {
    fun_transition_state_equal(t1, t2)
        && fun_transition_shape_equal(t1, t2)
        && fun_transition_index_equal(t1, t2)
}

/// `FUN_transitionSTATEEQUAL(T1, T2)`; `fun_transition_state_equal(&t1, &t2)`.
pub fn fun_transition_state_equal(
    t1: &TopOpeBRepDSTransition,
    t2: &TopOpeBRepDSTransition,
) -> bool {
    t1.before() == t2.before() && t1.after() == t2.after()
}

/// `FUN_transitionSHAPEEQUAL(T1, T2)`; `fun_transition_shape_equal(&t1, &t2)`.
pub fn fun_transition_shape_equal(
    t1: &TopOpeBRepDSTransition,
    t2: &TopOpeBRepDSTransition,
) -> bool {
    t1.shape_before() == t2.shape_before() && t1.shape_after() == t2.shape_after()
}

/// `FUN_transitionINDEXEQUAL(T1, T2)`; `fun_transition_index_equal(&t1, &t2)`.
pub fn fun_transition_index_equal(
    t1: &TopOpeBRepDSTransition,
    t2: &TopOpeBRepDSTransition,
) -> bool {
    t1.index_before() == t2.index_before() && t1.index_after() == t2.index_after()
}
