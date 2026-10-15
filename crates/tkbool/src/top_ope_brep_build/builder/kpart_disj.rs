//! The disjoint particular case of `TopOpeBRepBuild_Builder` (TopOpeBRepBuild_KPart.cxx):
//! the solids of disjoint shapes classified by their outer shells and combined.
use super::TopOpeBRepBuildBuilder;
use super::kpart::kpresu::*;
use crate::top_ope_brep_ds::TopOpeBRepDSBuildTool;
use crate::top_ope_brep_tool::{TopOpeBRepToolShapeClassifier, TopOpeBRepToolShapeExplorer};
use rustycad_tkbrep::topods::{self, Shape, Solid};
use rustycad_tkbrep::toptools::ShapeMapHasher;
use rustycad_tkernel::ncollection::{IndexedMap, Map};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::{ShapeEnum, State};
use rustycad_tktopalgo::brep_class3d::BRepClass3d;

/// The operations of the disjoint case (`TopOpeBRepBuild_KPart_Operation`).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum KPartOperation {
    Fuse,
    Common,
    Cut12,
    Cut21,
}

/// The solid maps of the disjoint case.
pub(super) type SolidMap = IndexedMap<Shape, ShapeMapHasher>;

/// `LocalKPisdisjanalyse(Stsol1, Stsol2, theOperation, ires, icla1, icla2)` (static):
/// the result and the shell selections of two disjoint solids by their states.
pub(super) fn local_kp_isdisjanalyse(
    st1: State,
    st2: State,
    op: KPartOperation,
) -> (i32, i32, i32) {
    use State::{In, Out};
    match (op, st1, st2) {
        (KPartOperation::Fuse, Out, Out) => (RESSHAPE12, SHEAUCU, SHEAUCU),
        (KPartOperation::Fuse, Out, In) => (RESNEWSHA1, SHECLASAUTR, SHECLASAUTR),
        (KPartOperation::Fuse, In, Out) => (RESNEWSHA2, SHECLASAUTR, SHECLASAUTR),
        (KPartOperation::Cut12, Out, Out) => (RESSHAPE1, SHEGARDTOUS, SHEAUCU),
        (KPartOperation::Cut12, Out, In) => (RESNEWSHA1, SHECLASAUTR, SHEGARDCOUR),
        (KPartOperation::Cut12, In, Out) => (RESNULL, SHEAUCU, SHEAUCU),
        (KPartOperation::Cut21, Out, Out) => (RESSHAPE2, SHEAUCU, SHEGARDTOUS),
        (KPartOperation::Cut21, Out, In) => (RESNULL, SHEAUCU, SHEAUCU),
        (KPartOperation::Cut21, In, Out) => (RESNEWSHA2, SHEGARDCOUR, SHECLASAUTR),
        (KPartOperation::Common, Out, Out) => (RESNULL, SHEAUCU, SHEAUCU),
        (KPartOperation::Common, Out, In) => (RESNEWSHA2, SHECLASAUTR, SHEGARDAUTR),
        (KPartOperation::Common, In, Out) => (RESNEWSHA1, SHEGARDAUTR, SHECLASAUTR),
        _ => (RESUNDEF, SHEUNDEF, SHEUNDEF),
    }
}

/// The shells of a solid selected by `icla` (classified against the other solid when
/// asked) added to the new solid, complemented for a solid IN.
#[allow(clippy::too_many_arguments)]
fn add_solid_shells(
    bt: &TopOpeBRepDSBuildTool,
    sc: &mut TopOpeBRepToolShapeClassifier,
    newsol: &mut Shape,
    isdisjmap: Option<&mut Map<Shape, ShapeMapHasher>>,
    sol: &Solid,
    solstate: State,
    shastatetoadd: State,
    outsha: &Shape,
    icla: i32,
    othersol: &Solid,
) -> Result<(), Failure> {
    let mut isdisjmap = isdisjmap;
    let mut exsha =
        TopOpeBRepToolShapeExplorer::new(sol.as_ref(), ShapeEnum::Shell, ShapeEnum::Shape)?;
    while exsha.more() {
        let shacur = exsha.current().clone();
        exsha.next()?;
        let isoutsha = shacur.is_equal(outsha);
        let garde = match icla {
            SHEAUCU => false,
            SHEGARDAUTR | SHECLASAUTR => !isoutsha,
            _ => true,
        };
        if !garde {
            continue;
        }
        let add = match icla {
            SHEGARDCOUR | SHEGARDAUTR | SHEGARDTOUS => true,
            SHECLASAUTR => {
                sc.state_shape_shape_avoiding(&shacur, &Shape::default(), othersol.as_ref())?
                    == shastatetoadd
            }
            _ => false,
        };
        if !add {
            continue;
        }
        let mut shaori = shacur;
        if solstate == State::In {
            shaori.complement();
        }
        match isdisjmap.as_deref_mut() {
            Some(m) => {
                if m.add(shaori.clone()) {
                    bt.add_solid_shell(newsol, &shaori)?;
                }
            }
            None => bt.add_solid_shell(newsol, &shaori)?,
        }
    }
    Ok(())
}

/// `BuildNewSolid(sol1, sol2, stsol1, stsol2, ires, icla1, icla2, theState1, theState2)`
/// (static): a solid of the outer shell of the new shape and the selected shells of
/// both solids (those of the second not checked against the added ones, as natively).
#[allow(clippy::too_many_arguments)]
pub(super) fn build_new_solid(
    sol1: &Solid,
    sol2: &Solid,
    st: (State, State),
    ires: i32,
    icla: (i32, i32),
    states: (State, State),
) -> Result<Solid, Failure> {
    let mut sc = TopOpeBRepToolShapeClassifier::new();
    let mut isdisjmap: Map<Shape, ShapeMapHasher> = Map::default();
    let bt = TopOpeBRepDSBuildTool::new();
    let outsha1 = BRepClass3d::outer_shell(sol1)?.into_shape();
    let outsha2 = BRepClass3d::outer_shell(sol2)?.into_shape();
    let mut newsol = Shape::default();
    bt.make_solid(&mut newsol)?;
    let outer = if ires == RESNEWSHA1 {
        Some(&outsha1)
    } else if ires == RESNEWSHA2 {
        Some(&outsha2)
    } else {
        None
    };
    if let Some(o) = outer
        && isdisjmap.add(o.clone())
    {
        bt.add_solid_shell(&mut newsol, o)?;
    }
    add_solid_shells(
        &bt,
        &mut sc,
        &mut newsol,
        Some(&mut isdisjmap),
        sol1,
        st.0,
        states.0,
        &outsha1,
        icla.0,
        sol2,
    )?;
    add_solid_shells(
        &bt,
        &mut sc,
        &mut newsol,
        None,
        sol2,
        st.1,
        states.1,
        &outsha2,
        icla.1,
        sol1,
    )?;
    topods::solid(&newsol)
}

/// The states of the outer shells of two solids in each other.
fn outer_states(
    sc: &mut TopOpeBRepToolShapeClassifier,
    s1: &Solid,
    s2: &Solid,
) -> Result<(State, State), Failure> {
    let o1 = BRepClass3d::outer_shell(s1)?.into_shape();
    let o2 = BRepClass3d::outer_shell(s2)?.into_shape();
    Ok((
        sc.state_shape_shape_avoiding(&o1, &Shape::default(), s2.as_ref())?,
        sc.state_shape_shape_avoiding(&o2, &Shape::default(), s1.as_ref())?,
    ))
}

/// The solid of a map entry (None when not a solid: the operation fails).
fn solid_at(m: &SolidMap, i: usize) -> Result<Option<Solid>, Failure> {
    let s = m.find_key(i)?;
    if s.shape_type()? != ShapeEnum::Solid {
        return Ok(None);
    }
    topods::solid(s).map(Some)
}

/// `disjPerformFuse(theMapOfSolid1, theMapOfSolid2, theMapOfResult)` (static): each
/// solid not used fused with the next ones it contains or is contained in; false on a
/// non solid or a programming error.
pub(super) fn disj_perform_fuse(
    m1: &SolidMap,
    m2: &SolidMap,
    result: &mut SolidMap,
) -> Result<bool, Failure> {
    result.clear(true);
    let mut all = m1.clone();
    for i in 1..=m2.extent() {
        all.add(m2.find_key(i)?.clone());
    }
    let mut sc = TopOpeBRepToolShapeClassifier::new();
    let mut used: Map<Shape, ShapeMapHasher> = Map::default();
    let n = all.extent();
    let max_iter = n * n;
    for i in 1..=n {
        let Some(sol1) = solid_at(&all, i)? else {
            return Ok(false);
        };
        let mut current = sol1.clone();
        if used.contains(sol1.as_ref()) {
            continue;
        }
        let (mut j, mut check) = (1, 0);
        while j <= n && check <= max_iter {
            check += 1;
            if j == i {
                j += 1;
                continue;
            }
            let Some(sol2) = solid_at(&all, j)? else {
                return Ok(false);
            };
            j += 1;
            if used.contains(sol2.as_ref()) {
                continue;
            }
            let (st1, st2) = outer_states(&mut sc, &current, &sol2)?;
            let (ires, icla1, icla2) = local_kp_isdisjanalyse(st1, st2, KPartOperation::Fuse);
            if ires == RESUNDEF || icla1 == SHEUNDEF || icla2 == SHEUNDEF || ires == RESNULL {
                println!("Warning: disjPerformFuse: can not determine solid's states");
                continue;
            }
            if ires == RESNEWSHA1 || ires == RESNEWSHA2 {
                let newsol = build_new_solid(
                    &current,
                    &sol2,
                    (st1, st2),
                    ires,
                    (icla1, icla2),
                    (State::Out, State::Out),
                )?;
                j = 1;
                current = newsol;
                used.add(sol2.into_shape());
                if current.is_null() {
                    return Ok(false);
                }
            }
        }
        if check > max_iter {
            println!("disjPerformFuse: programming error");
            return Ok(false);
        }
        result.add(current.into_shape());
    }
    Ok(true)
}

/// The commons of two separated solids (None to skip the couple).
fn common_of_couple(
    sc: &mut TopOpeBRepToolShapeClassifier,
    sol1: &Solid,
    sol2: &Solid,
    out: &mut SolidMap,
) -> Result<(), Failure> {
    let (st1, st2) = outer_states(sc, sol1, sol2)?;
    let (ires, icla1, icla2) = local_kp_isdisjanalyse(st1, st2, KPartOperation::Common);
    if ires == RESUNDEF || icla1 == SHEUNDEF || icla2 == SHEUNDEF {
        println!("Warning: disjPerformCommon: can not determine solid's states");
        return Ok(());
    }
    match ires {
        RESSHAPE12 => {
            out.add(sol1.clone().into_shape());
            out.add(sol2.clone().into_shape());
        }
        RESSHAPE1 => {
            out.add(sol1.clone().into_shape());
        }
        RESSHAPE2 => {
            out.add(sol2.clone().into_shape());
        }
        RESNEWSHA1 | RESNEWSHA2 => {
            let newsol = build_new_solid(
                sol1,
                sol2,
                (st1, st2),
                ires,
                (icla1, icla2),
                (State::In, State::In),
            )?;
            out.add(newsol.into_shape());
        }
        _ => {}
    }
    Ok(())
}

/// `disjPerformCommon(theMapOfSolid1, theMapOfSolid2, theMapOfResult)` (static): the
/// commons of the separated solids of the two maps, fused; false on a non solid.
pub(super) fn disj_perform_common(
    m1: &SolidMap,
    m2: &SolidMap,
    result: &mut SolidMap,
) -> Result<bool, Failure> {
    let (mut s1, mut s2, mut commons) = (
        SolidMap::default(),
        SolidMap::default(),
        SolidMap::default(),
    );
    result.clear(true);
    disj_perform_fuse(m1, m1, &mut s1)?;
    disj_perform_fuse(m2, m2, &mut s2)?;
    let mut sc = TopOpeBRepToolShapeClassifier::new();
    for i in 1..=s1.extent() {
        let Some(sol1) = solid_at(&s1, i)? else {
            return Ok(false);
        };
        for j in 1..=s2.extent() {
            let Some(sol2) = solid_at(&s2, j)? else {
                return Ok(false);
            };
            common_of_couple(&mut sc, &sol1, &sol2, &mut commons)?;
        }
    }
    disj_perform_fuse(&commons.clone(), &commons, result)?;
    Ok(true)
}

/// `disjPerformCut(theMapOfSolid1, theMapOfSolid2, theMapOfResult)` (static): each
/// separated solid of the first map cut by those of the second; false on a non solid
/// or a null result.
pub(super) fn disj_perform_cut(
    m1: &SolidMap,
    m2: &SolidMap,
    result: &mut SolidMap,
) -> Result<bool, Failure> {
    let (mut s1, mut s2) = (SolidMap::default(), SolidMap::default());
    result.clear(true);
    disj_perform_fuse(m1, m1, &mut s1)?;
    disj_perform_fuse(m2, m2, &mut s2)?;
    let mut sc = TopOpeBRepToolShapeClassifier::new();
    for i in 1..=s1.extent() {
        let Some(sol1) = solid_at(&s1, i)? else {
            return Ok(false);
        };
        let mut current = sol1;
        let mut null_result = false;
        let mut j = 1;
        while j <= s2.extent() && !null_result {
            let Some(sol2) = solid_at(&s2, j)? else {
                return Ok(false);
            };
            j += 1;
            let (st1, st2) = outer_states(&mut sc, &current, &sol2)?;
            let (ires, icla1, icla2) = local_kp_isdisjanalyse(st1, st2, KPartOperation::Cut12);
            if ires == RESUNDEF || icla1 == SHEUNDEF || icla2 == SHEUNDEF {
                println!("Warning: disjPerformCut: can not determine solid's states");
                continue;
            }
            match ires {
                RESNULL | RESSHAPE12 | RESSHAPE2 => null_result = true,
                RESSHAPE1 => null_result = false,
                RESNEWSHA1 | RESNEWSHA2 => {
                    current = build_new_solid(
                        &current,
                        &sol2,
                        (st1, st2),
                        ires,
                        (icla1, icla2),
                        (State::Out, State::In),
                    )?;
                }
                _ => {}
            }
        }
        if !null_result {
            if current.is_null() {
                return Ok(false);
            }
            result.add(current.into_shape());
        }
    }
    Ok(true)
}

impl TopOpeBRepBuildBuilder {}
