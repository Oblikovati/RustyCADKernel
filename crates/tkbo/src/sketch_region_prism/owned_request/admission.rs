// SPDX-License-Identifier: GPL-2.0-only
//! Empty input is declined; unsupported domains are incomplete, never empty successes.
use super::{names::*, request::*};
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum OwnedPrismAdmissionCause {
    EmptyInput,
    UnsupportedVersion {
        actual: u32,
        expected: u32,
    },
    MultipleBoundaries {
        actual: Vec<CallerBoundaryId>,
        supported: usize,
    },
    PoleCount {
        source: CallerBoundaryId,
        actual: usize,
        minimum: usize,
        maximum: usize,
    },
    NonfinitePole {
        source: CallerBoundaryId,
        pole: usize,
        axis: usize,
        actual: f64,
    },
    OpenBoundary {
        source: CallerBoundaryId,
        first: [f64; 3],
        last: [f64; 3],
    },
    InvalidExtrusion {
        actual: [f64; 3],
    },
    NonfinitePlane {
        component: DeclaredPlaneComponent,
        axis: usize,
        actual: f64,
    },
    Names(SemanticNameFailure),
}
pub(super) fn admit_owned_prism(
    request: &OwnedPolynomialPrismRequest,
) -> Result<SingleBoundaryNamePlan, OwnedPrismAdmissionCause> {
    admit_source_count(request)?;
    let source = request
        .sources
        .first()
        .ok_or(OwnedPrismAdmissionCause::EmptyInput)?;
    validate_poles(source)?;
    finite_plane(request.declared_plane)?;
    if request.extrusion.iter().any(|x| !x.is_finite()) || request.extrusion == [0.; 3] {
        return Err(OwnedPrismAdmissionCause::InvalidExtrusion {
            actual: request.extrusion,
        });
    }
    SingleBoundaryNamePlan::v1(source.names(request.operation), request.selection)
        .map_err(OwnedPrismAdmissionCause::Names)
}
fn admit_source_count(
    request: &OwnedPolynomialPrismRequest,
) -> Result<(), OwnedPrismAdmissionCause> {
    if request.version != 1 {
        return Err(OwnedPrismAdmissionCause::UnsupportedVersion {
            actual: request.version,
            expected: 1,
        });
    }
    if request.sources.is_empty() {
        return Err(OwnedPrismAdmissionCause::EmptyInput);
    }
    if request.sources.len() != 1 {
        return Err(OwnedPrismAdmissionCause::MultipleBoundaries {
            actual: request.sources.iter().map(|s| s.source).collect(),
            supported: 1,
        });
    }
    Ok(())
}
pub(super) fn validate_poles(
    source: &OwnedPolynomialBoundary,
) -> Result<(), OwnedPrismAdmissionCause> {
    let maximum = rustycad_tkg3d::geom::GeomBezierCurve::max_degree() as usize + 1;
    let actual = source.world_poles.len();
    if !(2..=maximum).contains(&actual) {
        return Err(OwnedPrismAdmissionCause::PoleCount {
            source: source.source,
            actual,
            minimum: 2,
            maximum,
        });
    }
    for (pole, coordinates) in source.world_poles.iter().enumerate() {
        finite_pole(source.source, pole, *coordinates)?;
    }
    validate_closed_poles(source)
}
fn validate_closed_poles(source: &OwnedPolynomialBoundary) -> Result<(), OwnedPrismAdmissionCause> {
    // Called after the pole-count gate, so both endpoint operands exist.
    let (first, last) = (
        source.world_poles[0],
        source.world_poles[source.world_poles.len() - 1],
    );
    if first != last {
        return Err(OwnedPrismAdmissionCause::OpenBoundary {
            source: source.source,
            first,
            last,
        });
    }
    Ok(())
}

fn finite_pole(
    source: CallerBoundaryId,
    pole: usize,
    coordinates: [f64; 3],
) -> Result<(), OwnedPrismAdmissionCause> {
    for (axis, actual) in coordinates.into_iter().enumerate() {
        if !actual.is_finite() {
            return Err(OwnedPrismAdmissionCause::NonfinitePole {
                source,
                pole,
                axis,
                actual,
            });
        }
    }
    Ok(())
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum DeclaredPlaneComponent {
    Origin,
    Normal,
    U,
    V,
}
pub(super) fn finite_plane(
    plane: rustycad_tkmath::gp::Plane3,
) -> Result<(), OwnedPrismAdmissionCause> {
    use DeclaredPlaneComponent::*;
    let frame = plane.position();
    for (component, point) in [
        (Origin, *frame.location().coordinates()),
        (Normal, *frame.direction().coordinates()),
        (U, *frame.x_direction().coordinates()),
        (V, *frame.y_direction().coordinates()),
    ] {
        finite_plane_component(component, point)?;
    }
    Ok(())
}
fn finite_plane_component(
    component: DeclaredPlaneComponent,
    point: [f64; 3],
) -> Result<(), OwnedPrismAdmissionCause> {
    for (axis, actual) in point.into_iter().enumerate() {
        if !actual.is_finite() {
            return Err(OwnedPrismAdmissionCause::NonfinitePlane {
                component,
                axis,
                actual,
            });
        }
    }
    Ok(())
}
