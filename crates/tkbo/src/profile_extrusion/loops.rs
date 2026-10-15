// SPDX-License-Identifier: GPL-2.0-only
//! Loop assembly from the caller vertex graph. Joins come from shared vertex ids only; the
//! certified loop proof later checks that the authored loops are disjoint Jordan curves.
use super::admitted::{AdmittedCurve, AdmittedProfile};
use super::decline::DeclineCause;
use super::profile::{ProfileCurveId, ProfileVertexId};
use std::collections::BTreeMap;

/// One directed use of a caller curve inside a loop.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LoopUse {
    /// The caller curve.
    pub curve: ProfileCurveId,
    /// Traversed from the authored end to the authored start.
    pub reversed: bool,
}

/// A closed loop of directed curve uses in traversal order. Its first use is the first
/// curve of the loop in caller order, so the value is deterministic for a given input.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProfileLoop {
    /// Directed uses in traversal order.
    pub uses: Vec<LoopUse>,
}

type Incidence = BTreeMap<ProfileVertexId, Vec<(usize, usize)>>;

/// Assemble every closed loop; `assemble_loops(&profile)?` declines open or branching graphs.
pub(crate) fn assemble_loops(profile: &AdmittedProfile) -> Result<Vec<ProfileLoop>, DeclineCause> {
    let incidence = incidence(&profile.curves);
    require_manifold(&incidence)?;
    let mut visited = vec![false; profile.curves.len()];
    let mut loops = Vec::new();
    for start in 0..profile.curves.len() {
        if !visited[start] {
            loops.push(trace_loop(&profile.curves, &incidence, start, &mut visited));
        }
    }
    Ok(loops)
}

/// Curve ends at each vertex as `(curve index, end index)`, in caller order.
fn incidence(curves: &[AdmittedCurve]) -> Incidence {
    let mut incidence = Incidence::new();
    for (index, curve) in curves.iter().enumerate() {
        for (end, vertex) in curve.ends().into_iter().flatten().enumerate() {
            incidence.entry(vertex).or_default().push((index, end));
        }
    }
    incidence
}

fn require_manifold(incidence: &Incidence) -> Result<(), DeclineCause> {
    for (vertex, uses) in incidence {
        let (vertex, uses) = (*vertex, uses.len());
        match uses {
            2 => {}
            0 | 1 => return Err(DeclineCause::OpenProfile { vertex, uses }),
            _ => return Err(DeclineCause::BranchingProfile { vertex, uses }),
        }
    }
    Ok(())
}

fn trace_loop(
    curves: &[AdmittedCurve],
    incidence: &Incidence,
    start: usize,
    visited: &mut [bool],
) -> ProfileLoop {
    let mut uses = Vec::new();
    let (mut index, mut reversed) = (start, false);
    while !visited[index] {
        visited[index] = true;
        uses.push(LoopUse {
            curve: curves[index].id(),
            reversed,
        });
        let Some(ends) = curves[index].ends() else {
            break;
        };
        let exit = ends[usize::from(!reversed)];
        (index, reversed) = next_use(&incidence[&exit], index);
    }
    ProfileLoop { uses }
}

/// The other curve end at a degree-2 vertex; entering at its end 1 means reversed traversal.
fn next_use(at_vertex: &[(usize, usize)], from: usize) -> (usize, bool) {
    let other = at_vertex
        .iter()
        .find(|(curve, _)| *curve != from)
        .copied()
        .expect("admission refuses closed open-curves, so a manifold vertex joins two curves");
    (other.0, other.1 == 1)
}
