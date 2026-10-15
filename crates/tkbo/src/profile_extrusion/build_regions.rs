// SPDX-License-Identifier: GPL-2.0-only
//! `build_planar_regions`: admission, loop assembly, certification by the general loop
//! region proof, native construction and ordered validation, in that order. One path
//! serves every admitted profile.
use super::admission::admit_profile;
use super::admitted::{AdmittedCurve, AdmittedProfile};
use super::decline::{Decliner, OperationDecline, OperationKind, OperationStage as Stage};
use super::loops::{ProfileLoop, assemble_loops};
use super::profile::{ProfileCurveId, ProfileVertexId, SketchProfile};
use super::proof::{CertifiedLoops, certify_loops};
use super::region_area::{RegionAreaInput, region_area};
use super::regions::*;
use super::resolution::{Measure, Resolution};
use super::topology::{ProfileTopology, build_topology, loop_wire, region_face};
use super::validation::{CheckedShape, SelfIntersectionEvidence, check_publication};
use rustycad_tkbrep::topods::{Edge, Face, Vertex};
use rustycad_tkernel::standard::Failure;
use std::collections::BTreeSet;

const DECLINE: Decliner = Decliner {
    operation: OperationKind::BuildPlanarRegions,
};

/// Convert a solved sketch into every bounded planar face it encloses.
///
/// Curves join only through shared caller vertex ids. Each closed loop bounds one region:
/// the loop minus the loops directly inside it, so a circle inside a circle yields the ring
/// and the inner disc. A rectangle takes the same path as any other profile.
///
/// ```
/// use rustycad_tkbo::profile_extrusion::*;
/// let corners = [[0., 0.], [2., 0.], [2., 1.], [0., 1.]];
/// let vertices = corners.iter().enumerate()
///     .map(|(i, p)| ProfileVertex { id: ProfileVertexId(i as u64), point: *p }).collect();
/// let curves = (0..4u64).map(|i| ProfileCurve::Line { id: ProfileCurveId(10 + i),
///     start: ProfileVertexId(i), end: ProfileVertexId((i + 1) % 4) }).collect();
/// let profile = SketchProfile { plane: ProfilePlane::world_xy(), vertices, curves };
/// let display = DisplayTolerance { chordal: 1e-3, angular: 0.1 };
/// let resolution = Resolution::new(1e-6, 1e-7, 1e-9, display, ToleranceCap::Refuse(1e-6));
/// let set = build_planar_regions(&profile, resolution)?;
/// assert_eq!(set.regions().len(), 1);
/// assert!(set.regions()[0].area().encloses(2.0));
/// # Ok::<(), OperationDecline>(())
/// ```
pub fn build_planar_regions(profile: &SketchProfile, resolution: Resolution) -> PlanarRegionResult {
    let profile =
        admit_profile(profile, resolution).map_err(|c| DECLINE.at(Stage::Admission, c))?;
    let loops = assemble_loops(&profile).map_err(|c| DECLINE.at(Stage::Loops, c))?;
    let certified =
        certify_loops(&profile, &loops).map_err(|c| DECLINE.at(Stage::Certification, c))?;
    let topology = build_topology(&profile)
        .map_err(|f| DECLINE.native(Stage::Construction, "profile topology", f))?;
    let context = RegionContext {
        profile: &profile,
        loops: &loops,
        certified: &certified,
        topology: &topology,
    };
    let regions = (0..loops.len())
        .map(|index| context.region(index))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(context.publish(regions))
}

struct RegionContext<'a> {
    profile: &'a AdmittedProfile,
    loops: &'a [ProfileLoop],
    certified: &'a CertifiedLoops,
    topology: &'a ProfileTopology,
}

impl RegionContext<'_> {
    fn publish(&self, regions: Vec<PlanarRegion>) -> PlanarRegionSet {
        let certificate = RegionCertificate {
            pair_domains: self.certified.pair_domains,
            carrier_pairs: self.certified.carrier_pairs,
            loops: self.loops.len(),
        };
        let curve_edges = curve_edges(self.profile, self.topology, &regions);
        PlanarRegionSet {
            regions,
            curve_edges,
            resolution: self.profile.resolution,
            certificate,
        }
    }

    fn region(&self, outer: usize) -> Result<PlanarRegion, OperationDecline> {
        let parents = &self.certified.parents;
        let holes: Vec<usize> = (0..self.loops.len())
            .filter(|hole| parents[*hole] == Some(outer))
            .collect();
        let members: Vec<usize> = std::iter::once(outer)
            .chain(holes.iter().copied())
            .collect();
        let face = self
            .face(&members)
            .map_err(|f| DECLINE.native(Stage::Construction, "region face", f))?;
        let evidence = SelfIntersectionEvidence::RegionContactCertificate;
        let checked = check_publication(&face, evidence, self.profile.resolution.cap, DECLINE)?;
        let area = self
            .area(&members)
            .map_err(|c| DECLINE.at(Stage::Certification, c))?;
        Ok(self.assemble(face, &members, area, checked))
    }

    fn assemble(
        &self,
        face: Face,
        members: &[usize],
        area: Measure,
        checked: CheckedShape,
    ) -> PlanarRegion {
        let hole_loops = members[1..].iter().map(|index| self.loops[*index].clone());
        PlanarRegion {
            id: RegionId(members[0] as u32),
            face,
            outer: self.loops[members[0]].clone(),
            holes: hole_loops.collect(),
            area,
            edges: self.edges(members),
            vertices: self.vertices(members),
            frame: self.profile.frame,
            validation: checked.validation,
            achieved: checked.achieved,
        }
    }

    /// Face bounded by the first member (outer) and the remaining members (holes).
    fn face(&self, members: &[usize]) -> Result<Face, Failure> {
        let orientation = &self.certified.orientation;
        let wire = |index: usize, outer| {
            loop_wire(self.topology, &self.loops[index], orientation[index], outer)
        };
        let wires = members
            .iter()
            .enumerate()
            .map(|(rank, index)| wire(*index, rank == 0));
        let wires = wires.collect::<Result<Vec<_>, _>>()?;
        region_face(&self.profile.frame, &wires[0], &wires[1..])
    }

    fn area(&self, members: &[usize]) -> Result<Measure, super::decline::DeclineCause> {
        region_area(&RegionAreaInput {
            certified: self.certified.areas[members[0]],
            loop_areas: members
                .iter()
                .map(|index| self.certified.orientation[*index])
                .collect(),
            line_only: members.iter().all(|index| self.line_only(*index)),
        })
    }

    fn line_only(&self, index: usize) -> bool {
        let mut curves = self.loops[index]
            .uses
            .iter()
            .map(|used| self.profile.curve(used.curve));
        curves.all(|curve| matches!(curve, AdmittedCurve::Line { .. }))
    }

    fn edges(&self, members: &[usize]) -> Vec<(ProfileCurveId, Edge)> {
        let uses = members
            .iter()
            .flat_map(|index| self.loops[*index].uses.iter());
        uses.map(|used| (used.curve, self.topology.edges[&used.curve].clone()))
            .collect()
    }

    fn vertices(&self, members: &[usize]) -> Vec<(ProfileVertexId, Vertex)> {
        let uses = members
            .iter()
            .flat_map(|index| self.loops[*index].uses.iter());
        let ids: BTreeSet<_> = uses
            .filter_map(|used| self.profile.curve(used.curve).ends())
            .flatten()
            .collect();
        ids.into_iter()
            .map(|id| (id, self.topology.vertices[&id].clone()))
            .collect()
    }
}

fn curve_edges(
    profile: &AdmittedProfile,
    topology: &ProfileTopology,
    regions: &[PlanarRegion],
) -> Vec<CurveEdge> {
    let bounded = move |curve| {
        regions
            .iter()
            .filter(move |region| region.edges.iter().any(|(id, _)| *id == curve))
    };
    let entry = |curve: &AdmittedCurve| CurveEdge {
        curve: curve.id(),
        edge: topology.edges[&curve.id()].clone(),
        regions: bounded(curve.id()).map(|region| region.id).collect(),
    };
    profile.curves.iter().map(entry).collect()
}
