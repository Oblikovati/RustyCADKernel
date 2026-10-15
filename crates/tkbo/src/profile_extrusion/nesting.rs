// SPDX-License-Identifier: GPL-2.0-only
//! Certified loop nesting and seed confirmation. With pairwise disjoint Jordan loops
//! (certified by `contacts`), one point of a loop decides containment; each location is the
//! exact nonzero-winding classification of the loop's Bézier pieces.
use super::admitted::AdmittedProfile;
use super::bezier::{BezierPiece, loop_pieces};
use super::decline::DeclineCause;
use super::loops::ProfileLoop;
use rustycad_tkmath::predicates::roots::RootIsolationOptions;
use rustycad_tkmath::predicates::roots::bezier_line::*;

/// Loop containment forest: `parents[i]` is the innermost loop strictly containing loop `i`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct LoopForest {
    pub parents: Vec<Option<usize>>,
    /// `contains[i][j]`: loop `i` strictly contains loop `j`.
    pub contains: Vec<Vec<bool>>,
}

/// Certify the nesting of disjoint loops; `certify_nesting(&profile, &loops)?`.
pub(crate) fn certify_nesting(
    profile: &AdmittedProfile,
    loops: &[ProfileLoop],
) -> Result<LoopForest, DeclineCause> {
    let wires: Vec<_> = loops
        .iter()
        .map(|closed| loop_pieces(profile, closed))
        .collect();
    let mut contains = vec![vec![false; loops.len()]; loops.len()];
    for (inner, pieces) in wires.iter().enumerate() {
        let witness = pieces[0].controls[0];
        for (outer, wire) in wires
            .iter()
            .enumerate()
            .filter(|(outer, _)| *outer != inner)
        {
            contains[outer][inner] = locate(wire, witness, profile)?;
        }
    }
    let parents = (0..loops.len())
        .map(|inner| innermost(&contains, inner))
        .collect::<Result<_, _>>()?;
    Ok(LoopForest { parents, contains })
}

/// Whether `point` is inside the Jordan loop `wire`; a boundary point or a winding other
/// than `+-1` contradicts the certified disjointness and is an incomplete certificate.
pub(crate) fn locate(
    wire: &[BezierPiece],
    point: [f64; 2],
    profile: &AdmittedProfile,
) -> Result<bool, DeclineCause> {
    let segments: Vec<_> = wire
        .iter()
        .enumerate()
        .map(|(id, piece)| segment(id, piece))
        .collect();
    let options = RootIsolationOptions::new(profile.resolution.weld);
    let line = [point, [point[0] + 1.0, point[1]]];
    let query = BezierWireQuery {
        segments: &segments,
        line,
        options,
    };
    let result =
        classify_bezier_wire(query, || false).map_err(|cause| incomplete(format!("{cause:?}")))?;
    match result.location() {
        BezierWireLocation::Outside => Ok(false),
        BezierWireLocation::Inside { winding: 1 | -1 } => Ok(true),
        other => Err(incomplete(format!("point {point:?} located {other:?}"))),
    }
}

fn segment(id: usize, piece: &BezierPiece) -> BezierWireSegment<'_> {
    BezierWireSegment {
        id: id as u64,
        controls: &piece.controls,
        weights: piece.weights.as_deref(),
        interval: [0.0, 1.0],
        reversed: false,
    }
}

fn incomplete(reason: String) -> DeclineCause {
    DeclineCause::IncompleteCertification {
        reason: format!("loop nesting: {reason}"),
    }
}

/// The container of `inner` that every other container of `inner` also contains.
fn innermost(contains: &[Vec<bool>], inner: usize) -> Result<Option<usize>, DeclineCause> {
    let containers: Vec<_> = (0..contains.len())
        .filter(|outer| contains[*outer][inner])
        .collect();
    let innermost = containers.iter().copied().find(|candidate| {
        containers
            .iter()
            .all(|other| other == candidate || contains[*other][*candidate])
    });
    match (containers.is_empty(), innermost) {
        (true, _) => Ok(None),
        (false, Some(parent)) => Ok(Some(parent)),
        (false, None) => Err(incomplete(format!(
            "containers {containers:?} of loop {inner} are not a chain"
        ))),
    }
}

/// Confirm each seed lies inside its own loop and its ancestors and outside every other
/// loop, i.e. in the face bounded by the loop and its children.
pub(crate) fn confirm_seeds(
    profile: &AdmittedProfile,
    loops: &[ProfileLoop],
    forest: &LoopForest,
    seeds: &[[f64; 2]],
) -> Result<(), DeclineCause> {
    let wires: Vec<_> = loops
        .iter()
        .map(|closed| loop_pieces(profile, closed))
        .collect();
    for (owner, seed) in seeds.iter().enumerate() {
        for (other, wire) in wires.iter().enumerate() {
            let expected = other == owner || forest.contains[other][owner];
            if locate(wire, *seed, profile)? != expected {
                return Err(incomplete(format!(
                    "seed {seed:?} of loop {owner} against loop {other}"
                )));
            }
        }
    }
    Ok(())
}
