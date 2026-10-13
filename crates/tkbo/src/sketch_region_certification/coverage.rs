// SPDX-License-Identifier: GPL-2.0-only
//! History and actual shared vertex incidence gate numerical endpoint evidence, never proximity.
use super::outcome::*;
use crate::sketch_region::{ArrangementRelation, OwnedArrangement, equal_occurrences};
use rustycad_tkbrep::{
    brep::BRepTool,
    topods::{Shape, edge, vertex},
};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::{gp::Plane3, predicates::roots::bezier_line::CertifiedBezierIncidence};
use std::cmp::Ordering;

type EndpointRead = Result<EndpointEvidence, Failure>;
struct CoverageScope<'a> {
    census: &'a BoundaryCensus,
    arranged: &'a OwnedArrangement,
    plane: Plane3,
}
struct ContactProbe<'a> {
    scope: &'a CoverageScope<'a>,
    pair: [usize; 2],
    identity: (usize, usize),
    contact: &'a CertifiedBezierIncidence,
}
/// Inspect actual split endpoints; `inspect_contact_coverage(&census, &arranged, plane)`.
/// A witness is provisional until source conversion, all pair domains and residuals are certified.
pub(crate) fn inspect_contact_coverage(
    census: &BoundaryCensus,
    arranged: &OwnedArrangement,
    plane: Plane3,
) -> Result<Vec<ContactCoverage>, Failure> {
    let scope = CoverageScope {
        census,
        arranged,
        plane,
    };
    let mut coverage = Vec::new();
    for report in 0..census.reports.len() {
        coverage.extend(scope.inspect_report(report)?);
    }
    Ok(coverage)
}
impl CoverageScope<'_> {
    fn inspect_report(&self, report: usize) -> Result<Vec<ContactCoverage>, Failure> {
        let (curve, line) = report_inputs(self.census, report);
        self.census.reports[report]
            .contacts()
            .iter()
            .enumerate()
            .map(|(index, contact)| {
                ContactProbe {
                    scope: self,
                    pair: [curve, line],
                    identity: (report, index),
                    contact,
                }
                .inspect()
            })
            .collect()
    }
}
fn report_inputs(census: &BoundaryCensus, target: usize) -> (usize, usize) {
    census
        .domains
        .iter()
        .find_map(|domain| match domain.evidence {
            DomainEvidence::StoredIncidence {
                report,
                curve,
                line,
            } if report == target => Some((curve, line)),
            _ => None,
        })
        .expect("retained incidence report must have its actual source domain")
}
impl ContactProbe<'_> {
    fn inspect(&self) -> Result<ContactCoverage, Failure> {
        // No plane/conversion residual certificate exists yet, even for an exact stored root witness.
        Ok(ContactCoverage {
            report: self.identity.0,
            contact: self.identity.1,
            vertices: self.collect_endpoints()?,
            complete: false,
            missing: EndpointGap::SourcePlaneAndParameterConstructionUncertified,
        })
    }
    fn collect_endpoints(&self) -> Result<Vec<EndpointEvidence>, Failure> {
        let mut vertices = Vec::new();
        for relation in &self.scope.arranged.relations {
            if relation.output.shape_type()? == ShapeEnum::Vertex && self.generated_pair(relation) {
                vertices.extend(self.endpoint(relation)?);
            }
        }
        Ok(vertices)
    }
    fn generated_pair(&self, relation: &ArrangementRelation) -> bool {
        self.pair.iter().all(|index| {
            self.scope.census.inputs.segments[*index]
                .parents
                .iter()
                .all(|parent| relation.generated.contains(parent))
        })
    }
    fn endpoint(
        &self,
        relation: &ArrangementRelation,
    ) -> Result<Option<EndpointEvidence>, Failure> {
        let Some(edges) = self.incident_pair(&relation.output)? else {
            return Ok(None);
        };
        let target = vertex(&relation.output)?;
        let curve = BRepTool::parameter(&target, &edge(&edges[0])?)?;
        let line = BRepTool::parameter(&target, &edge(&edges[1])?)?;
        self.observe_split(relation, (curve, line)).map(Some)
    }
    fn incident_pair(&self, target: &Shape) -> Result<Option<[Shape; 2]>, Failure> {
        let segments = &self.scope.census.inputs.segments;
        let first = incident_edge(self.scope.arranged, target, &segments[self.pair[0]].parents)?;
        let second = incident_edge(self.scope.arranged, target, &segments[self.pair[1]].parents)?;
        Ok(match (first, second) {
            (Some(first), Some(second)) => Some([first, second]),
            _ => None,
        })
    }
    fn observe_split(
        &self,
        relation: &ArrangementRelation,
        parameters: (f64, f64),
    ) -> EndpointRead {
        let (in_bracket, exact_parameter) = self.observe_parameter(parameters.0);
        let point = BRepTool::pnt(&vertex(&relation.output)?)?;
        Ok(EndpointEvidence {
            vertex: relation.output.clone(),
            parameter: parameters.0,
            line_parameter: parameters.1,
            parameter_in_bracket: in_bracket,
            exact_parameter,
            world_inside_enclosure: self.observed_metric_bound(point),
            generated_from_both: true,
            certificate_world_error: self.contact.achieved_world_error(),
        })
    }
    fn observed_metric_bound(&self, point: rustycad_tkmath::gp::Point3) -> bool {
        // Observation cannot certify the unmeasured source-plane construction residual.
        inside_bounds(
            project(point, self.scope.plane),
            self.contact.world_bounds(),
        )
    }
    fn observe_parameter(&self, parameter: f64) -> (bool, bool) {
        let source = &self.scope.census.inputs.segments[self.pair[0]];
        let bounds = source.original_interval;
        let basis = if source.interval == bounds {
            parameter
        } else {
            (parameter - bounds[0]) / (bounds[1] - bounds[0])
        };
        let (in_bracket, singleton) = parameter_membership(self.contact, basis);
        (in_bracket, singleton && source.interval == bounds)
    }
}

fn incident_edge(
    arranged: &OwnedArrangement,
    target: &Shape,
    parents: &[crate::sketch_region::OriginalMemberIdentity],
) -> Result<Option<Shape>, Failure> {
    for relation in &arranged.relations {
        if relation.output.shape_type()? != ShapeEnum::Edge || !edge_parents(relation, parents) {
            continue;
        }
        if equal_occurrences(&relation.output)?
            .iter()
            .any(|child| child.is_same(target))
        {
            return Ok(Some(relation.output.clone()));
        }
    }
    Ok(None)
}
fn edge_parents(
    relation: &ArrangementRelation,
    parents: &[crate::sketch_region::OriginalMemberIdentity],
) -> bool {
    parents.iter().all(|parent| {
        relation.preserved.contains(parent)
            || relation.modified.contains(parent)
            || relation.generated.contains(parent)
    })
}
fn parameter_membership(contact: &CertifiedBezierIncidence, parameter: f64) -> (bool, bool) {
    let (lower, upper) = contact.parameter().bounds();
    let lo = lower.compare_f64(parameter).ok();
    let hi = upper.compare_f64(parameter).ok();
    (
        matches!(lo, Some(Ordering::Less | Ordering::Equal))
            && matches!(hi, Some(Ordering::Greater | Ordering::Equal)),
        lo == Some(Ordering::Equal) && hi == Some(Ordering::Equal),
    )
}
fn project(point: rustycad_tkmath::gp::Point3, plane: Plane3) -> [f64; 2] {
    let origin = plane.location();
    let delta = [
        point.x() - origin.x(),
        point.y() - origin.y(),
        point.z() - origin.z(),
    ];
    let frame = plane.position();
    [
        dot(delta, *frame.x_direction().coordinates()),
        dot(delta, *frame.y_direction().coordinates()),
    ]
}
fn dot(first: [f64; 3], second: [f64; 3]) -> f64 {
    // This observation is ordinary floating projection, never an outward residual certificate.
    first[0] * second[0] + first[1] * second[1] + first[2] * second[2]
}
fn inside_bounds(point: [f64; 2], bounds: [[f64; 2]; 2]) -> bool {
    // Bounds are inspected without widening; source-to-plane conversion is still uncertified.
    (0..2).all(|axis| point[axis] >= bounds[axis][0] && point[axis] <= bounds[axis][1])
}
