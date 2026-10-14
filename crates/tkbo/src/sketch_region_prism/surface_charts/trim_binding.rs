// SPDX-License-Identifier: GPL-2.0-only
use super::super::{PrismImageBinding, PrismImageRole};
use super::{capture::capture_bezier_edge, trim_capture::*, types::*};
use crate::sketch_region::PartitionOccurrenceIdentity;
use crate::sketch_region_geometry_capture::CapturedMetricCurve;
use crate::sketch_region_source_conformance::{
    CapturedSpaceRepresentation, capture_space_representation,
};
use rustycad_tkbrep::topods::{Shape, edge};
use rustycad_tkg3d::top_abs::ShapeEnum;

pub(super) fn capture_bound_trim(
    output: &Shape,
    face: &PrismImageBinding,
    bindings: &[PrismImageBinding],
    sources: &CapturedPrismSources,
) -> Result<CapturedPrismTrim, PrismSurfaceCause> {
    let mut matched = Vec::new();
    for parent in bindings
        .iter()
        .filter(|b| matching_context(b, output, &face.context.occurrence))
    {
        if let Some((role, expected)) = expected_trim(output, parent, face, sources)? {
            matched.push((parent, role, expected));
        }
    }
    require_single_trim_source(matched.len())?;
    let (parent, role, expected) = matched.pop().unwrap();
    make_trim(output, parent, &face.context.occurrence, role, expected)
}
fn require_single_trim_source(count: usize) -> Result<(), PrismSurfaceCause> {
    if count != 1 {
        return Err(PrismSurfaceCause::AmbiguousHistory {
            entity: "trim source",
            count,
        });
    }
    Ok(())
}
fn matching_context(binding: &PrismImageBinding, output: &Shape, face: &Shape) -> bool {
    binding.context.occurrence.is_equal(output)
        && binding
            .context
            .ancestors
            .iter()
            .any(|ancestor| ancestor.is_equal(face))
}
fn expected_trim(
    output: &Shape,
    parent: &PrismImageBinding,
    face: &PrismImageBinding,
    sources: &CapturedPrismSources,
) -> Result<Option<(PrismTrimRole, CapturedTrimExpectation)>, PrismSurfaceCause> {
    if face.native.role == PrismImageRole::Generated {
        return side_expectation(output, parent, face, sources);
    }
    if parent.native.role != face.native.role
        || parent.native.query.shape_type()? != ShapeEnum::Edge
    {
        return Ok(None);
    }
    cap_source_expectation(parent, sources)
}
fn cap_source_expectation(
    parent: &PrismImageBinding,
    sources: &CapturedPrismSources,
) -> Result<Option<(PrismTrimRole, CapturedTrimExpectation)>, PrismSurfaceCause> {
    let key = PartitionOccurrenceIdentity::capture(&parent.native.query);
    let source = sources
        .boundaries
        .iter()
        .find(|entry| entry.occurrence == key);
    Ok(source.map(|source| (cap_role(parent.native.role), cap_expectation(source))))
}

fn cap_role(role: PrismImageRole) -> PrismTrimRole {
    match role {
        PrismImageRole::First => PrismTrimRole::First,
        _ => PrismTrimRole::Last,
    }
}
fn side_expectation(
    output: &Shape,
    parent: &PrismImageBinding,
    face: &PrismImageBinding,
    sources: &CapturedPrismSources,
) -> Result<Option<(PrismTrimRole, CapturedTrimExpectation)>, PrismSurfaceCause> {
    if parent.native.query.is_equal(&face.native.query)
        && parent.native.role != PrismImageRole::Generated
    {
        let expected = CapturedTrimExpectation::SideGenerating;
        return Ok(Some((cap_role(parent.native.role), expected)));
    }
    if parent.native.role != PrismImageRole::Generated
        || parent.native.query.shape_type()? != ShapeEnum::Vertex
    {
        return Ok(None);
    }
    directing_expectation(output, parent, face, sources)
}
fn directing_expectation(
    output: &Shape,
    parent: &PrismImageBinding,
    face: &PrismImageBinding,
    sources: &CapturedPrismSources,
) -> Result<Option<(PrismTrimRole, CapturedTrimExpectation)>, PrismSurfaceCause> {
    let source = super::sources::source_boundary(sources, &face.native.query)?;
    let key = PartitionOccurrenceIdentity::capture(&parent.native.query);
    let Some((_, parameter)) = source.endpoints.iter().find(|(vertex, _)| *vertex == key) else {
        return Ok(None);
    };
    let parameter = *parameter;
    match_directing_parameter(output, &face.context.occurrence, parameter)
}
fn match_directing_parameter(
    output: &Shape,
    face: &Shape,
    parameter: f64,
) -> Result<Option<(PrismTrimRole, CapturedTrimExpectation)>, PrismSurfaceCause> {
    let (pcurve, _) = pcurve_on(output, face)?;
    let CapturedMetricCurve::Line { origin, .. } = pcurve else {
        return Ok(None);
    };
    // Both seam occurrences can share physical topology: the stored parameter map disambiguates charts.
    if origin[0] != parameter {
        return Ok(None);
    }
    Ok(directing_parameter_role(parameter))
}
fn directing_parameter_role(parameter: f64) -> Option<(PrismTrimRole, CapturedTrimExpectation)> {
    let role = match parameter {
        0. => PrismTrimRole::Start,
        1. => PrismTrimRole::End,
        _ => return None,
    };
    Some((role, CapturedTrimExpectation::SideDirecting { parameter }))
}
pub(super) fn trim_space(
    output: &Shape,
    role: PrismTrimRole,
) -> Result<CapturedSpaceRepresentation, PrismSurfaceCause> {
    if matches!(role, PrismTrimRole::First | PrismTrimRole::Last) {
        return capture_bezier_edge(&edge(output)?);
    }
    let captured = capture_space_representation(&edge(output)?)?;
    if !matches!(captured.curve, CapturedMetricCurve::Line { .. }) {
        return Err(PrismSurfaceCause::Unsupported {
            representation: "directing edge line",
            actual: "nonlinear curve".to_owned(),
        });
    }
    if !captured.same_parameter {
        return Err(PrismSurfaceCause::SameParameterRequired);
    }
    Ok(captured)
}
