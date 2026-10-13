// SPDX-License-Identifier: GPL-2.0-only
//! Declared body invariants precede mapped representation checks.
use super::outcome::{BodyKind, ValidationCause, ValidationStage};
use rustycad_tkbrep::brep_adaptor::BRepAdaptorSurface;
use rustycad_tkbrep::topods::{Shape, face};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::geom_abs::SurfaceType;

pub(super) fn body_cause(
    shape: &Shape,
    kind: BodyKind,
    stage: ValidationStage,
) -> Result<Option<ValidationCause>, Failure> {
    let actual = shape.shape_type()?;
    let expected = expected_shape_kind(kind);
    if actual != expected {
        return Ok(Some(ValidationCause::BodyKindMismatch {
            expected: kind,
            actual,
        }));
    }
    if kind == BodyKind::PlanarRegion && stage == ValidationStage::Geometry {
        return plane_cause(shape);
    }
    Ok(None)
}

fn plane_cause(shape: &Shape) -> Result<Option<ValidationCause>, Failure> {
    let surface = BRepAdaptorSurface::with_face(&face(shape)?, false)?;
    if surface.get_type()? == SurfaceType::Plane {
        return Ok(None);
    }
    Ok(Some(ValidationCause::NonPlanarRegion))
}

fn expected_shape_kind(kind: BodyKind) -> ShapeEnum {
    match kind {
        BodyKind::PlanarRegion => ShapeEnum::Face,
        BodyKind::Solid => ShapeEnum::Solid,
    }
}
