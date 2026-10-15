// SPDX-License-Identifier: GPL-2.0-only
//! `build_planar_regions`: admission, the closed-graph check, certification by the plane
//! graph region proof, native construction and ordered validation, in that order. One path
//! serves every admitted profile.
use super::admission::admit_profile;
use super::admitted::{AdmittedCurve, AdmittedProfile};
use super::decline::{Decliner, OperationDecline, OperationKind, OperationStage as Stage};
use super::loop_geometry::{loop_scale, signed_area};
use super::loops::{ProfileLoop, require_closed};
use super::profile::{ProfileCurveId, ProfileVertexId, SketchProfile};
use super::proof::{CertifiedFace, CertifiedRegions, certify_regions};
use super::region_area::{RegionAreaInput, region_area};
use super::regions::*;
use super::resolution::Resolution;
use super::topology::{ProfileTopology, build_topology, loop_wire, region_face};
use super::validation::{CheckedShape, SelfIntersectionEvidence, check_publication};
use crate::measured::Measured;
use rustycad_tkbrep::topods::{Edge, Face, Vertex};
use rustycad_tkernel::standard::Failure;
use std::collections::BTreeSet;

const DECLINE: Decliner = Decliner {
    operation: OperationKind::BuildPlanarRegions,
};

/// Convert a solved sketch into every bounded planar face it encloses.
///
/// Curves join only through shared caller vertex ids. Every bounded face of the curve
/// graph is one region: loops may be nested (a circle in a circle yields the ring and the
/// inner disc), may touch at a vertex, and adjacent regions may share a curve (a rectangle
/// split by a line yields two regions whose shared edge is mapped to both). A rectangle
/// takes the same path as any other profile.
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
    require_closed(&profile).map_err(|c| DECLINE.at(Stage::Loops, c))?;
    let certified = certify_regions(&profile).map_err(|c| DECLINE.at(Stage::Certification, c))?;
    let topology = build_topology(&profile)
        .map_err(|f| DECLINE.native(Stage::Construction, "profile topology", f))?;
    let context = RegionContext {
        profile: &profile,
        certified: &certified,
        topology: &topology,
    };
    let regions = certified
        .faces
        .iter()
        .enumerate()
        .map(|(index, face)| context.region(index, face))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(context.publish(regions))
}

struct RegionContext<'a> {
    profile: &'a AdmittedProfile,
    certified: &'a CertifiedRegions,
    topology: &'a ProfileTopology,
}

impl RegionContext<'_> {
    fn publish(&self, regions: Vec<PlanarRegion>) -> PlanarRegionSet {
        let certificate = RegionCertificate {
            pair_domains: self.certified.pair_domains,
            carrier_pairs: self.certified.carrier_pairs,
            loops: regions.len(),
            components: self.certified.components,
        };
        let curve_edges = curve_edges(self.profile, self.topology, &regions);
        PlanarRegionSet {
            regions,
            curve_edges,
            resolution: self.profile.resolution,
            certificate,
        }
    }

    fn region(&self, index: usize, face: &CertifiedFace) -> Result<PlanarRegion, OperationDecline> {
        let loops: Vec<&ProfileLoop> = std::iter::once(&face.outer).chain(&face.holes).collect();
        let native = self
            .face(&loops)
            .map_err(|f| DECLINE.native(Stage::Construction, "region face", f))?;
        let evidence = SelfIntersectionEvidence::RegionContactCertificate;
        let checked = check_publication(&native, evidence, self.profile.resolution.cap, DECLINE)?;
        let area = self
            .area(face, &loops)
            .map_err(|c| DECLINE.at(Stage::Certification, c))?;
        Ok(self.assemble(index, native, face, &loops, area, checked))
    }

    fn assemble(
        &self,
        index: usize,
        native: Face,
        face: &CertifiedFace,
        loops: &[&ProfileLoop],
        area: Measured,
        checked: CheckedShape,
    ) -> PlanarRegion {
        PlanarRegion {
            id: RegionId(index as u32),
            face: native,
            outer: face.outer.clone(),
            holes: face.holes.clone(),
            area,
            edges: self.edges(loops),
            vertices: self.vertices(loops),
            frame: self.profile.frame,
            validation: checked.validation,
            achieved: checked.achieved,
        }
    }

    /// Face bounded by the first loop (outer, counterclockwise) and the rest (holes).
    fn face(&self, loops: &[&ProfileLoop]) -> Result<Face, Failure> {
        let wires = loops.iter().map(|closed| loop_wire(self.topology, closed));
        let wires = wires.collect::<Result<Vec<_>, _>>()?;
        region_face(&self.profile.frame, &wires[0], &wires[1..])
    }

    fn area(
        &self,
        face: &CertifiedFace,
        loops: &[&ProfileLoop],
    ) -> Result<Measured, super::decline::DeclineCause> {
        region_area(&RegionAreaInput {
            certified: face.area,
            loop_areas: loops
                .iter()
                .map(|closed| signed_area(self.profile, closed))
                .collect(),
            line_only: loops.iter().all(|closed| self.line_only(closed)),
            scale: loops.iter().map(|l| loop_scale(self.profile, l)).sum(),
        })
    }

    fn line_only(&self, closed: &ProfileLoop) -> bool {
        let mut curves = closed
            .uses
            .iter()
            .map(|used| self.profile.curve(used.curve));
        curves.all(|curve| matches!(curve, AdmittedCurve::Line { .. }))
    }

    fn edges(&self, loops: &[&ProfileLoop]) -> Vec<(ProfileCurveId, Edge)> {
        let uses = loops.iter().flat_map(|closed| closed.uses.iter());
        uses.map(|used| (used.curve, self.topology.edges[&used.curve].clone()))
            .collect()
    }

    fn vertices(&self, loops: &[&ProfileLoop]) -> Vec<(ProfileVertexId, Vertex)> {
        let uses = loops.iter().flat_map(|closed| closed.uses.iter());
        let ids: BTreeSet<_> = uses
            .filter_map(|used| self.profile.curve(used.curve).ends())
            .flatten()
            .collect();
        ids.into_iter()
            .map(|id| (id, self.topology.vertices[&id].clone()))
            .collect()
    }
}

/// One entry per caller curve: its edge and the regions it bounds (two for a curve shared
/// by adjacent regions, none for a bridge outside every region).
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
