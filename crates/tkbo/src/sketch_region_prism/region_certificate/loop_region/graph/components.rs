// SPDX-License-Identifier: GPL-2.0-only
//! ADR 0084 steps 3-4: connected components, outer walks, bounded faces (Euler's relation,
//! simple boundaries) and the nesting of components inside bounded faces.
use super::super::binding::SegmentBinding;
use super::super::types::*;
use super::super::winding::{locate_with, off_loop};
use super::darts::Dart;
use super::{GraphComponentProof, GraphRegionRequest};
use rustycad_tkmath::predicates::roots::bezier_line::BezierWireLocation;
use std::collections::BTreeSet;

/// Orbits grouped by connected component (components ordered by their first source).
pub(super) struct Graph {
    pub orbits: Vec<Vec<Dart>>,
    /// Component of each orbit.
    pub orbit_component: Vec<usize>,
    /// Sources of each component, ascending.
    pub component_sources: Vec<Vec<usize>>,
}

/// One bounded face before nesting: its counterclockwise walk and component.
pub(super) struct BoundedFace {
    pub orbit: Vec<Dart>,
    pub component: usize,
}

/// Bounded faces and the clockwise outer walk of every component.
pub(super) struct Faces {
    pub bounded: Vec<BoundedFace>,
    pub outer: Vec<Vec<Dart>>,
}

/// Group sources into components through shared vertex ids.
pub(super) fn components(request: &GraphRegionRequest<'_, '_>, orbits: Vec<Vec<Dart>>) -> Graph {
    let ends = request.ends;
    let mut label: Vec<usize> = (0..ends.len()).collect();
    for a in 0..ends.len() {
        for b in a + 1..ends.len() {
            if ends[a].iter().any(|v| ends[b].contains(v)) {
                let (ra, rb) = (root(&mut label, a), root(&mut label, b));
                label[ra.max(rb)] = ra.min(rb);
            }
        }
    }
    let roots: Vec<usize> = (0..ends.len()).map(|s| root(&mut label, s)).collect();
    let distinct: Vec<usize> = roots
        .iter()
        .copied()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();
    let component_of = |source: usize| distinct.binary_search(&roots[source]).expect("root");
    let component_sources = (0..distinct.len())
        .map(|k| (0..ends.len()).filter(|s| component_of(*s) == k).collect())
        .collect();
    let orbit_component = orbits
        .iter()
        .map(|orbit| component_of(orbit[0].source))
        .collect();
    Graph {
        orbits,
        orbit_component,
        component_sources,
    }
}

fn root(label: &mut [usize], mut x: usize) -> usize {
    while label[x] != x {
        label[x] = label[label[x]];
        x = label[x];
    }
    x
}

/// Split orbits by certified sign: one negative outer walk per component, the rest bounded
/// faces; Euler's relation `F = E - V + 2` per component; bounded faces must be simple.
pub(super) fn classify(
    request: &GraphRegionRequest<'_, '_>,
    graph: &Graph,
    signs: &[i8],
) -> LoopRegionCheck<Faces> {
    for (component, sources) in graph.component_sources.iter().enumerate() {
        euler(request, graph, component, sources)?;
    }
    let mut outer: Vec<Vec<Vec<Dart>>> = vec![Vec::new(); graph.component_sources.len()];
    let mut bounded = Vec::new();
    for (index, orbit) in graph.orbits.iter().enumerate() {
        let component = graph.orbit_component[index];
        if signs[index] < 0 {
            outer[component].push(orbit.clone());
            continue;
        }
        require_simple(request, orbit, |vertex| LoopRegionCause::PinchedFace {
            vertex,
        })?;
        let orbit = orbit.clone();
        bounded.push(BoundedFace { orbit, component });
    }
    let outer = outer
        .into_iter()
        .enumerate()
        .map(|(component, walks)| one_outer(component, walks))
        .collect::<LoopRegionCheck<Vec<_>>>()?;
    Ok(Faces { bounded, outer })
}

fn one_outer(component: usize, mut walks: Vec<Vec<Dart>>) -> LoopRegionCheck<Vec<Dart>> {
    if walks.len() == 1 {
        return Ok(walks.remove(0));
    }
    let outer = walks.len();
    let cause = LoopRegionCause::OrbitCount { component, outer };
    refuse(LoopRegionStage::Faces, cause)
}

/// `F = E - V + 2` for one connected component; any other count means the rotation is
/// not a planar embedding.
fn euler(
    request: &GraphRegionRequest<'_, '_>,
    graph: &Graph,
    component: usize,
    sources: &[usize],
) -> LoopRegionCheck<()> {
    let vertices: BTreeSet<u64> = sources.iter().flat_map(|s| request.ends[*s]).collect();
    let expected = (sources.len() + 2).saturating_sub(vertices.len());
    let actual = graph
        .orbit_component
        .iter()
        .filter(|c| **c == component)
        .count();
    if actual == expected {
        return Ok(());
    }
    let quantity = "Euler face count of a component";
    let cause = LoopRegionCause::Count {
        quantity,
        actual,
        expected,
    };
    refuse(LoopRegionStage::Faces, cause)
}

/// A walk that revisits a vertex is not a Jordan curve.
fn require_simple(
    request: &GraphRegionRequest<'_, '_>,
    walk: &[Dart],
    cause: impl Fn(u64) -> LoopRegionCause,
) -> LoopRegionCheck<()> {
    let mut seen = BTreeSet::new();
    let tails = walk.iter().map(|d| d.tail(request.ends));
    match tails.into_iter().find(|v| !seen.insert(*v)) {
        Some(vertex) => refuse(LoopRegionStage::Faces, cause(vertex)),
        None => Ok(()),
    }
}

/// The innermost bounded face (of another component) containing each component. A point
/// of one component decides it, because components are disjoint and bounded faces simple.
pub(super) fn nesting(
    request: &GraphRegionRequest<'_, '_>,
    binding: &SegmentBinding<'_>,
    graph: &Graph,
    faces: &Faces,
    cancelled: &mut impl FnMut() -> bool,
) -> LoopRegionCheck<Vec<GraphComponentProof>> {
    let count = graph.component_sources.len();
    let mut containers: Vec<BTreeSet<usize>> = vec![BTreeSet::new(); count];
    for (component, sources) in graph.component_sources.iter().enumerate() {
        let witness = binding.segments[sources[0]].controls[0];
        for (index, face) in faces.bounded.iter().enumerate() {
            if face.component == component {
                continue;
            }
            if winding(request, binding, index, face, witness, cancelled)? != 0 {
                containers[component].insert(index);
            }
        }
    }
    let mut proofs = Vec::with_capacity(count);
    for component in 0..count {
        let container = innermost(faces, &containers, component)?;
        if container.is_some() {
            let cause = |vertex| LoopRegionCause::PinchedHole { vertex };
            require_simple(request, &faces.outer[component], cause)?;
        }
        let outer = faces.outer[component]
            .iter()
            .map(|d| d.authored(binding))
            .collect();
        proofs.push(GraphComponentProof { outer, container });
    }
    Ok(proofs)
}

fn winding(
    request: &GraphRegionRequest<'_, '_>,
    binding: &SegmentBinding<'_>,
    index: usize,
    face: &BoundedFace,
    point: [f64; 2],
    cancelled: &mut impl FnMut() -> bool,
) -> LoopRegionCheck<i64> {
    let authored = AuthoredLoop {
        name: index as u64 + 1,
        uses: face.orbit.iter().map(|d| d.authored(binding)).collect(),
    };
    let options = request.options.winding;
    let location = locate_with(options, binding, &authored, point, cancelled)?;
    if matches!(location, BezierWireLocation::Boundary { .. }) {
        let loops = [authored.name, 0];
        return refuse(
            LoopRegionStage::Nesting,
            LoopRegionCause::NestingContradiction { loops },
        );
    }
    off_loop(&authored, point, location)
}

/// The container that lies inside every other container of `component`; containers must
/// come from distinct components and form a strict chain.
fn innermost(
    faces: &Faces,
    containers: &[BTreeSet<usize>],
    component: usize,
) -> LoopRegionCheck<Option<usize>> {
    let own = &containers[component];
    let inside =
        |face: usize, other: usize| containers[faces.bounded[face].component].contains(&other);
    for &f in own {
        for &g in own.iter().filter(|g| **g > f) {
            let same = faces.bounded[f].component == faces.bounded[g].component;
            if same || inside(f, g) == inside(g, f) {
                let loops = [f as u64 + 1, g as u64 + 1];
                return refuse(
                    LoopRegionStage::Nesting,
                    LoopRegionCause::NestingContradiction { loops },
                );
            }
        }
    }
    Ok(own
        .iter()
        .copied()
        .find(|f| own.iter().all(|g| g == f || inside(*f, *g))))
}
