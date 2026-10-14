// SPDX-License-Identifier: GPL-2.0-only
//! Capture actual stored coefficients; alternate evaluators and unsupported surfaces stay incomplete.
use super::super::{PrismImageBinding, PrismImageRole};
use super::{trim_capture::capture_trims, types::*};
use crate::sketch_region_geometry_capture::{capture_space_curve, captured_placement};
use crate::sketch_region_source_conformance::{capture_space_representation, capture_support};
use rustycad_tkbrep::{
    brep::BRepTool,
    topods::{Edge, Shape, face},
};
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::geom::{
    GeomBezierCurve, GeomCurve, GeomPlane, GeomSurfaceOfLinearExtrusion, GeomSweptSurface,
};

pub(super) fn capture_chart(
    binding: &PrismImageBinding,
    bindings: &[PrismImageBinding],
    extrusion: [f64; 3],
    sources: &PrismChartSources,
) -> Result<CapturedSurfaceChart, PrismSurfaceCause> {
    let sources = sources
        .as_ref()
        .map_err(|cause| PrismSurfaceCause::SourceSnapshot(cause.clone()))?;
    let source = source_chart(binding, sources)?;
    let actual = actual_surface(&binding.context.occurrence)?;
    let trims = capture_trims(binding, bindings, sources)?;
    Ok(CapturedSurfaceChart {
        source,
        actual,
        trims,
        extrusion,
    })
}
fn source_chart(
    binding: &PrismImageBinding,
    sources: &CapturedPrismSources,
) -> Result<CapturedChartSource, PrismSurfaceCause> {
    if binding.native.role != PrismImageRole::Generated {
        return cap_source(binding, sources);
    }
    let boundary = super::sources::source_boundary(sources, &binding.native.query)?;
    Ok(CapturedChartSource::Side(boundary.space.clone()))
}
fn cap_source(
    binding: &PrismImageBinding,
    sources: &CapturedPrismSources,
) -> Result<CapturedChartSource, PrismSurfaceCause> {
    let key = crate::sketch_region::PartitionOccurrenceIdentity::capture(&binding.native.query);
    if key != sources.face {
        return Err(PrismSurfaceCause::MissingHistory {
            entity: "pre-sweep face history source",
        });
    }
    Ok(CapturedChartSource::Cap(sources.support.clone()))
}
pub(super) fn capture_bezier_edge(
    edge: &Edge,
) -> Result<crate::sketch_region_source_conformance::CapturedSpaceRepresentation, PrismSurfaceCause>
{
    let raw = BRepTool::curve_with_location(edge)?;
    require_bezier_evaluator(&raw.curve)?;
    let captured = capture_space_representation(edge)?;
    super::identity::require_range(captured.interval, [0., 1.])?;
    if !captured.same_parameter {
        return Err(PrismSurfaceCause::SameParameterRequired);
    }
    Ok(captured)
}
pub(super) fn require_bezier_evaluator(
    curve: &Handle<dyn GeomCurve>,
) -> Result<(), PrismSurfaceCause> {
    let bezier = curve.cast::<GeomBezierCurve>();
    let Some(bezier) = bezier.get() else {
        return Err(PrismSurfaceCause::Unsupported {
            representation: "stored full-range Bézier",
            actual: curve
                .get()
                .map_or("null", |c| c.dynamic_type().name())
                .to_owned(),
        });
    };
    if bezier.eval_representation().is_some() {
        return Err(PrismSurfaceCause::AlternateEvaluator {
            representation: "Bézier curve",
        });
    }
    Ok(())
}
fn actual_surface(shape: &Shape) -> Result<CapturedPrismSurface, PrismSurfaceCause> {
    let actual = face(shape)?;
    let (surface, location) = BRepTool::surface_with_location(&actual)?;
    if surface.cast::<GeomPlane>().is_some() {
        return Ok(CapturedPrismSurface::Plane(capture_support(&actual)?));
    }
    let extrusion = surface.cast::<GeomSurfaceOfLinearExtrusion>();
    let Some(extrusion) = extrusion.get() else {
        return Err(PrismSurfaceCause::Unsupported {
            representation: "plane or stored linear extrusion",
            actual: surface
                .get()
                .map_or("null", |s| s.dynamic_type().name())
                .to_owned(),
        });
    };
    captured_extrusion(extrusion, location.transformation())
}
fn captured_extrusion(
    surface: &GeomSurfaceOfLinearExtrusion,
    location: rustycad_tkmath::gp::Transform3,
) -> Result<CapturedPrismSurface, PrismSurfaceCause> {
    if surface.eval_representation().is_some() {
        return Err(PrismSurfaceCause::AlternateEvaluator {
            representation: "extrusion surface",
        });
    }
    let basis = surface.basis_curve();
    require_bezier_evaluator(&basis)?;
    let direction = surface.direction();
    Ok(CapturedPrismSurface::Extrusion {
        basis: capture_space_curve(&basis)?,
        direction: [direction.x(), direction.y(), direction.z()],
        placement: captured_placement(location)?,
    })
}
