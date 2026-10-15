// SPDX-License-Identifier: GPL-2.0-only
//! `build_planar_regions`: admission, loop assembly, certification, native construction and
//! ordered validation, in that order. One path serves every admitted profile.
use super::admission::admit_profile;
use super::admitted::AdmittedProfile;
use super::contacts::certify_contacts;
use super::decline::{DeclineCause, Decliner, OperationKind, OperationStage as Stage};
use super::loop_geometry::signed_area;
use super::loops::{ProfileLoop, assemble_loops};
use super::nesting::{LoopForest, certify_nesting, confirm_seeds};
use super::profile::SketchProfile;
use super::regions::*;
use super::resolution::{Measure, MeasureKind, Resolution};
use super::seeds::loop_seeds;
use super::topology::{ProfileTopology, build_topology, loop_wire, region_face};
use super::validation::{
    SelfIntersectionEvidence, achieved_tolerance, ordered_validation, tolerance_stage,
};
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
    let admitted =
        admit_profile(profile, resolution).map_err(|c| DECLINE.at(Stage::Admission, c))?;
    let loops = assemble_loops(&admitted).map_err(|c| DECLINE.at(Stage::Loops, c))?;
    let certified = certify(&admitted, &loops).map_err(|c| DECLINE.at(Stage::Certification, c))?;
    let topology = build_topology(&admitted)
        .map_err(|f| DECLINE.native(Stage::Construction, "profile topology", f))?;
    let context = RegionContext {
        profile: &admitted,
        loops: &loops,
        certified: &certified,
        topology: &topology,
    };
    let regions = (0..loops.len())
        .map(|index| context.region(index))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(PlanarRegionSet {
        curve_edges: curve_edges(&admitted, &topology, &regions),
        regions,
        resolution,
        certificate: RegionCertificate {
            contact_pairs: certified.contact_pairs,
            loops: loops.len(),
        },
    })
}

/// Certified facts about the loops: pairwise contacts, nesting and confirmed seeds.
struct CertifiedLoops {
    forest: LoopForest,
    areas: Vec<(f64, f64)>,
    contact_pairs: usize,
}

fn certify(
    profile: &AdmittedProfile,
    loops: &[ProfileLoop],
) -> Result<CertifiedLoops, DeclineCause> {
    let contact_pairs = certify_contacts(profile)?;
    let areas: Vec<_> = loops
        .iter()
        .map(|closed| signed_area(profile, closed))
        .collect();
    let signs: Vec<f64> = areas.iter().map(|(area, _)| area.signum()).collect();
    let seeds = loop_seeds(profile, loops, &signs).ok_or_else(|| {
        DeclineCause::IncompleteCertification {
            reason: "no inward boundary crossing for a loop seed".into(),
        }
    })?;
    let forest = certify_nesting(profile, loops)?;
    confirm_seeds(profile, loops, &forest, &seeds)?;
    Ok(CertifiedLoops {
        forest,
        areas,
        contact_pairs,
    })
}

struct RegionContext<'a> {
    profile: &'a AdmittedProfile,
    loops: &'a [ProfileLoop],
    certified: &'a CertifiedLoops,
    topology: &'a ProfileTopology,
}

impl RegionContext<'_> {
    fn holes(&self, outer: usize) -> Vec<usize> {
        let parents = &self.certified.forest.parents;
        (0..self.loops.len())
            .filter(|hole| parents[*hole] == Some(outer))
            .collect()
    }

    fn region(&self, outer: usize) -> Result<PlanarRegion, super::decline::OperationDecline> {
        let holes = self.holes(outer);
        let face = self
            .face(outer, &holes)
            .map_err(|f| DECLINE.native(Stage::Construction, "region face", f))?;
        let mut validation =
            ordered_validation(&face, SelfIntersectionEvidence::RegionContactCertificate)
                .map_err(|c| DECLINE.at(Stage::Validation, c))?;
        let achieved = achieved_tolerance(&face)
            .map_err(|f| DECLINE.native(Stage::Tolerance, "region tolerance", f))?;
        let cap = self.profile.resolution.cap;
        tolerance_stage(achieved, cap, &mut validation, &mut Vec::new())
            .map_err(|c| DECLINE.at(Stage::Tolerance, c))?;
        let loops: Vec<&ProfileLoop> = std::iter::once(outer)
            .chain(holes.iter().copied())
            .map(|i| &self.loops[i])
            .collect();
        Ok(PlanarRegion {
            id: RegionId(outer as u32),
            face,
            outer: self.loops[outer].clone(),
            holes: holes.iter().map(|hole| self.loops[*hole].clone()).collect(),
            area: self.area(outer, &holes),
            edges: self.edges(&loops),
            vertices: self.vertices(&loops),
            frame: self.profile.frame,
            validation,
            achieved,
        })
    }

    fn face(
        &self,
        outer: usize,
        holes: &[usize],
    ) -> Result<rustycad_tkbrep::topods::Face, rustycad_tkernel::standard::Failure> {
        let wire = |index: usize, is_outer: bool| {
            loop_wire(
                self.topology,
                &self.loops[index],
                self.certified.areas[index].0,
                is_outer,
            )
        };
        let outer_wire = wire(outer, true)?;
        let hole_wires = holes
            .iter()
            .map(|hole| wire(*hole, false))
            .collect::<Result<Vec<_>, _>>()?;
        region_face(&self.profile.frame, &outer_wire, &hole_wires)
    }

    fn area(&self, outer: usize, holes: &[usize]) -> Measure {
        let areas = &self.certified.areas;
        let holes_area: f64 = holes.iter().map(|hole| areas[*hole].0.abs()).sum();
        let error = std::iter::once(outer)
            .chain(holes.iter().copied())
            .map(|i| areas[i].1)
            .sum();
        Measure {
            value: areas[outer].0.abs() - holes_area,
            error,
            kind: MeasureKind::ClosedForm,
        }
    }

    fn edges(
        &self,
        loops: &[&ProfileLoop],
    ) -> Vec<(
        super::profile::ProfileCurveId,
        rustycad_tkbrep::topods::Edge,
    )> {
        let uses = loops.iter().flat_map(|closed| closed.uses.iter());
        uses.map(|used| (used.curve, self.topology.edges[&used.curve].clone()))
            .collect()
    }

    fn vertices(
        &self,
        loops: &[&ProfileLoop],
    ) -> Vec<(
        super::profile::ProfileVertexId,
        rustycad_tkbrep::topods::Vertex,
    )> {
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
    profile
        .curves
        .iter()
        .map(|curve| CurveEdge {
            curve: curve.id(),
            edge: topology.edges[&curve.id()].clone(),
            regions: bounded(curve.id()).map(|region| region.id).collect(),
        })
        .collect()
}
