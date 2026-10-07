//! HLRBRep_Hider.cxx: the hidden and on-face parts of an edge with interferences.
use super::{EdgeLists, Hider, hide_all};
use crate::hlr_algo::Interference;
use crate::hlr_brep::{Data, EdgeBuilder, EdgeIList, EdgeInterferenceTool, VertexList};
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::standard::Failure;
use rustycad_tkernel::standard::real::real_last;
use rustycad_tkg3d::top_abs::{Orientation, State};

// How the parts of one build are hidden: classified at their middle first (the
// `IsSuspicion` test of the hidden parts), clamped to the interference range (all
// interferences INTERNAL), and the `on face` and `on boundary` flags of the hiding.
struct PartRule {
    to_build: State,
    classify: bool,
    clamp: Option<(f64, f64)>,
    on_face: bool,
    on_boundary: bool,
}

// The bounds of the current part of the builder: FORWARD and REVERSED vertices, or
// None when one is missing or the part is shorter than 1e-7.
fn part_bounds(eb: &mut EdgeBuilder) -> Result<Option<(f64, f32, f64, f32)>, Failure> {
    let (mut p1, mut p2, mut tol1, mut tol2) = (0., 0., 0., 0.);
    let mut mask = 0;
    while eb.more_vertices() {
        match eb.orientation()? {
            Orientation::Forward => {
                let c = eb.current()?;
                (p1, tol1) = (c.parameter(), c.tolerance());
                mask |= 1;
            }
            Orientation::Reversed => {
                let c = eb.current()?;
                (p2, tol2) = (c.parameter(), c.tolerance());
                mask |= 2;
            }
            Orientation::Internal | Orientation::External => {}
        }
        eb.next_vertex()?;
    }
    if mask != 3 || p2 - p1 <= 1.0e-7 {
        return Ok(None);
    }
    Ok(Some((p1, tol1, p2, tol2)))
}

// Hides the parts of the edge `e` the builder finds for one rule.
fn hide_built_parts(
    ds: &Data,
    e: i32,
    eb: &mut EdgeBuilder,
    rule: &PartRule,
) -> Result<(), Failure> {
    eb.builds(rule.to_build)?;
    while eb.more_edges() {
        let Some((mut p1, tol1, mut p2, tol2)) = part_bounds(eb)? else {
            eb.next_edge();
            continue;
        };
        if let Some((pmin, pmax)) = rule.clamp {
            if p1 < pmin {
                p1 = pmin;
            }
            if p2 > pmax {
                p2 = pmax;
            }
        }
        let test_state = if rule.classify {
            ds.classify(e, true, (p1 + p2) / 2.)?.0
        } else {
            State::In
        };
        if test_state != State::Out {
            let mut ed = ds.e_data_array();
            let ed = ed
                .change_value(e)
                .map_err(|x| x.into_failure("HLRBRep_Hider::Hide"))?;
            ed.status_mut()
                .hide(p1, tol1, p2, tol2, rule.on_face, rule.on_boundary);
        }
        eb.next_edge();
    }
    Ok(())
}

// The parameter range of more than one interference, all INTERNAL (IFV).
fn internal_range(il: &[Interference]) -> Option<(f64, f64)> {
    if il.len() <= 1 || !il.iter().all(|i| i.transition() == Orientation::Internal) {
        return None;
    }
    let pmin = real_last();
    let range = il.iter().fold((pmin, -pmin), |(lo, hi), i| {
        let p = i.intersection().parameter();
        (if p < lo { p } else { lo }, if p > hi { p } else { hi })
    });
    Some(range)
}

// Removes the second of two adjacent FORWARD (the first of two REVERSED)
// interferences of a simple hiding face.
fn remove_excess(il: &mut Vec<Interference>) {
    let mut to_remove: Vec<f64> = Vec::new();
    let (mut prev_trans, mut prev_param) = (Orientation::External, 0.);
    for int in il.iter() {
        let a_trans = int.transition();
        if a_trans == prev_trans {
            match a_trans {
                Orientation::Forward => to_remove.push(int.intersection().parameter()),
                Orientation::Reversed => to_remove.push(prev_param),
                _ => {}
            }
        }
        prev_trans = a_trans;
        prev_param = int.intersection().parameter();
    }
    let mut i = 0;
    while i < il.len() {
        let a_param = il[i].intersection().parameter();
        if let Some(k) = to_remove.iter().position(|p| a_param == *p) {
            il.remove(i);
            to_remove.remove(k);
        } else {
            i += 1;
        }
    }
}

// Suppresses the transitions inside other layers: an interference is removed
// while the level of face layers in front of the edge is positive.
fn suppress_multi_inside(il: &mut Vec<Interference>, mut level: i32) {
    let mut i = 0;
    while i < il.len() {
        let int = il[i];
        match int.transition() {
            Orientation::Forward => {
                let decal = int.intersection().level();
                if level > 0 {
                    il.remove(i);
                } else {
                    i += 1;
                }
                level += decal;
            }
            Orientation::Reversed => {
                level -= int.intersection().level();
                if level > 0 {
                    il.remove(i);
                } else {
                    i += 1;
                }
            }
            Orientation::External | Orientation::Internal => i += 1,
        }
    }
}

fn as_list(il: &[Interference]) -> List<Interference> {
    il.iter().copied().collect()
}

impl Hider {
    pub(super) fn hide_parts(
        &self,
        ds: &Data,
        eit: &EdgeInterferenceTool,
        e: i32,
        lists: &mut EdgeLists,
    ) -> Result<(), Failure> {
        let mut found_hidden = false;
        if !lists.hidden.is_empty() {
            EdgeIList::process_complex_in(&mut lists.hidden, eit)?;
            let simple = ds.simple_hiding_face();
            let level = if simple {
                0
            } else {
                ds.hiding_start_level(e, &as_list(&lists.hidden))?
            };
            if simple {
                remove_excess(&mut lists.hidden);
            }
            suppress_multi_inside(&mut lists.hidden, level);
            if lists.hidden.is_empty() {
                hide_all(ds, e)?; // Edge hidden
            } else {
                found_hidden = true;
            }
        }
        if !lists.hidden.is_empty() {
            let clamp = internal_range(&lists.hidden);
            let mut v_list = VertexList::new(eit, &lists.hidden)?;
            let mut eb = EdgeBuilder::new(&mut v_list)?;
            let rules = [
                PartRule {
                    to_build: State::In,
                    classify: true,
                    clamp,
                    on_face: false,
                    on_boundary: false,
                },
                PartRule {
                    to_build: State::On,
                    classify: true,
                    clamp: None,
                    on_face: false,
                    on_boundary: true,
                },
            ];
            for rule in &rules {
                hide_built_parts(ds, e, &mut eb, rule)?;
            }
        }
        if !lists.on.is_empty() {
            let level = if ds.simple_hiding_face() {
                0
            } else {
                ds.hiding_start_level(e, &as_list(&lists.on))?
            };
            if level > 0 {
                suppress_multi_inside(&mut lists.on, level);
                if lists.on.is_empty() && !found_hidden {
                    hide_all(ds, e)?; // Edge hidden
                }
            }
        }
        if !lists.on.is_empty() {
            let mut v_list = VertexList::new(eit, &lists.on)?;
            let mut eb = EdgeBuilder::new(&mut v_list)?;
            let rules = [
                PartRule {
                    to_build: State::In,
                    classify: false,
                    clamp: None,
                    on_face: true,
                    on_boundary: false,
                },
                PartRule {
                    to_build: State::On,
                    classify: false,
                    clamp: None,
                    on_face: true,
                    on_boundary: true,
                },
            ];
            for rule in &rules {
                hide_built_parts(ds, e, &mut eb, rule)?;
            }
        }
        Ok(())
    }
}
