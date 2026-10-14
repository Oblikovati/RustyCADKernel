// SPDX-License-Identifier: GPL-2.0-only
//! Inspect stored representation sizes before the existing capture adapters materialize coefficients.
use super::super::super::contexts::body_contexts;
use super::*;
use crate::sketch_region_geometry_capture::CapturedMetricCurve;
use rustycad_tkbrep::{
    brep::BRepTool,
    topods::{Shape, edge, face},
};
use rustycad_tkernel::standard::Handle;
use rustycad_tkg2d::geom2d::{Geom2dBezierCurve, Geom2dCurve, Geom2dLine};
use rustycad_tkg3d::geom::{
    GeomBezierCurve, GeomCurve, GeomLine, GeomPlane, GeomSurfaceOfLinearExtrusion, GeomSweptSurface,
};

pub(super) fn inspect(
    request: &TwoLoopRegionRequest<'_, '_>,
    receipt: &PrismReceipt,
    reserved: usize,
) -> AnnularCheck<'static, usize> {
    let mut counts = Vec::new();
    let selected = &receipt
        .input
        .selected
        .as_ref()
        .expect("preflight selected")
        .face;
    let copied = receipt.copied.as_ref().expect("preflight copied");
    let body = &receipt.diagnostic.as_ref().expect("preflight body").body;
    for root in [selected.as_ref(), &copied.base, body] {
        root_counts(request, root, &mut counts)?;
    }
    if let Ok(sources) = copied.chart_sources.as_ref() {
        require(
            "bounded retained copied source boundaries",
            sources.boundaries.len(),
            2,
        )?;
        for boundary in &sources.boundaries {
            counts.push(captured_count(&boundary.space.curve)?);
            counts.push(captured_count(&boundary.pcurve)?);
        }
    }
    admit_two_loop_polynomial_counts(request, &counts, reserved)
        .map_err(|cause| Box::new(AnnularBindingCause::Region(cause)))
}
fn root_counts(
    request: &TwoLoopRegionRequest<'_, '_>,
    root: &Shape,
    counts: &mut Vec<i32>,
) -> AnnularCheck<'static> {
    // The caller has bounded this complete tree before this temporary context materialization.
    let contexts = body_contexts(root)
        .map_err(|cause| Box::new(AnnularBindingCause::Body(Box::new(cause))))?;
    for context in &contexts {
        match context.occurrence.shape_type()? {
            ShapeEnum::Face => surface_count(request, &context.occurrence, counts)?,
            ShapeEnum::Edge => edge_counts(request, context, counts)?,
            _ => (),
        }
    }
    Ok(())
}
fn edge_counts(
    request: &TwoLoopRegionRequest<'_, '_>,
    context: &PrismBodyContext,
    counts: &mut Vec<i32>,
) -> AnnularCheck<'static> {
    let curve = BRepTool::curve_with_location(&edge(&context.occurrence)?)?;
    counts.push(admit_count(request, space_count(&curve.curve)?)?);
    let support = face_ancestor(context)?;
    let pcurve = BRepTool::curve_on_face(&edge(&context.occurrence)?, &face(support)?)?;
    holds("stored actual pcurve before capture", pcurve.is_stored)?;
    counts.push(admit_count(request, plane_count(&pcurve.pcurve)?)?);
    Ok(())
}
fn surface_count(
    request: &TwoLoopRegionRequest<'_, '_>,
    shape: &Shape,
    counts: &mut Vec<i32>,
) -> AnnularCheck<'static> {
    let (surface, _) = BRepTool::surface_with_location(&face(shape)?)?;
    if surface.cast::<GeomPlane>().is_some() {
        counts.push(0);
        return Ok(());
    }
    let handle = surface.cast::<GeomSurfaceOfLinearExtrusion>();
    let extrusion = handle.get().ok_or_else(|| {
        representation(
            "plane or stored linear extrusion",
            surface
                .get()
                .map_or("null", |value| value.dynamic_type().name()),
        )
    })?;
    no_alternate(
        "extrusion surface",
        extrusion.eval_representation().is_some(),
    )?;
    counts.push(admit_count(
        request,
        space_count(&extrusion.basis_curve())?,
    )?);
    Ok(())
}
fn space_count(curve: &Handle<dyn GeomCurve>) -> AnnularCheck<'static, i32> {
    let handle = curve.cast::<GeomBezierCurve>();
    if let Some(curve) = handle.get() {
        polynomial("space Bézier", curve.is_rational())?;
        no_alternate("Bézier curve", curve.eval_representation().is_some())?;
        return Ok(curve.nb_poles());
    }
    if curve.cast::<GeomLine>().is_some() {
        return Ok(0);
    }
    Err(representation(
        "stored line or polynomial space curve",
        curve
            .get()
            .map_or("null", |value| value.dynamic_type().name()),
    ))
}
fn plane_count(curve: &Handle<dyn Geom2dCurve>) -> AnnularCheck<'static, i32> {
    let handle = curve.cast::<Geom2dBezierCurve>();
    if let Some(curve) = handle.get() {
        polynomial("trim Bézier pcurve", curve.is_rational())?;
        no_alternate("trim Bézier pcurve", curve.eval_representation().is_some())?;
        return Ok(curve.nb_poles());
    }
    if curve.cast::<Geom2dLine>().is_some() {
        return Ok(0);
    }
    Err(representation(
        "stored line or polynomial pcurve",
        curve
            .get()
            .map_or("null", |value| value.dynamic_type().name()),
    ))
}
fn captured_count<const D: usize>(curve: &CapturedMetricCurve<D>) -> AnnularCheck<'static, i32> {
    let CapturedMetricCurve::Bezier { poles, weights } = curve else {
        return Ok(0);
    };
    polynomial("retained source Bézier", weights.is_some())?;
    i32::try_from(poles.len()).map_err(|_| {
        representation(
            "retained polynomial count representable as i32",
            &poles.len().to_string(),
        )
    })
}
fn unsupported(required: &'static str) -> Box<AnnularBindingCause<'static>> {
    Box::new(AnnularBindingCause::Count {
        required,
        actual: 0,
        expected: 1,
    })
}

fn face_ancestor(context: &PrismBodyContext) -> AnnularCheck<'static, &Shape> {
    for ancestor in context.ancestors.iter().rev() {
        if ancestor.shape_type()? == ShapeEnum::Face {
            return Ok(ancestor);
        }
    }
    Err(unsupported("actual trim face ancestor"))
}
fn admit_count(request: &TwoLoopRegionRequest<'_, '_>, count: i32) -> AnnularCheck<'static, i32> {
    admit_two_loop_polynomial_counts(request, &[count], 0)
        .map_err(|cause| Box::new(AnnularBindingCause::Region(cause)))?;
    Ok(count)
}

fn polynomial(kind: &'static str, rational: bool) -> AnnularCheck<'static> {
    if rational {
        return Err(representation(
            "non-rational stored polynomial",
            &format!("rational {kind}"),
        ));
    }
    Ok(())
}
fn no_alternate(representation: &'static str, present: bool) -> AnnularCheck<'static> {
    if present {
        return Err(Box::new(AnnularBindingCause::Capture(
            super::super::super::surface_charts::PrismSurfaceCause::AlternateEvaluator {
                representation,
            },
        )));
    }
    Ok(())
}
fn representation(expected: &'static str, actual: &str) -> Box<AnnularBindingCause<'static>> {
    Box::new(AnnularBindingCause::Capture(
        super::super::super::surface_charts::PrismSurfaceCause::Unsupported {
            representation: expected,
            actual: actual.to_owned(),
        },
    ))
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn raw_missing_and_rational_representation_refusals_keep_actual_values() {
        let Err(cause) = space_count(&Handle::default()) else {
            panic!("expected null space refusal");
        };
        assert!(
            matches!(*cause, AnnularBindingCause::Capture(super::super::super::super::surface_charts::PrismSurfaceCause::Unsupported { actual, .. }) if actual == "null")
        );
        assert!(plane_count(&Handle::default()).is_err());
        let curve = CapturedMetricCurve::Bezier {
            poles: vec![[0., 0.]; 2],
            weights: Some(vec![1., 2.]),
        };
        let Err(cause) = captured_count(&curve) else {
            panic!("expected rational representation refusal");
        };
        assert!(
            matches!(*cause, AnnularBindingCause::Capture(super::super::super::super::surface_charts::PrismSurfaceCause::Unsupported { actual, .. }) if actual == "rational retained source Bézier")
        );
    }
    #[test]
    fn alternate_and_fixed_line_metadata_keep_native_capture_policy() {
        assert!(no_alternate("authored evaluator", false).is_ok());
        let Err(cause) = no_alternate("authored evaluator", true) else {
            panic!("expected alternate evaluator refusal");
        };
        assert!(matches!(
            *cause,
            AnnularBindingCause::Capture(
                super::super::super::super::surface_charts::PrismSurfaceCause::AlternateEvaluator {
                    representation: "authored evaluator"
                }
            )
        ));
        let curve = CapturedMetricCurve::Line {
            origin: [0., 0.],
            direction: [1., 0.],
        };
        assert_eq!(
            captured_count(&curve).unwrap_or_else(|_| panic!("fixed line descriptor")),
            0
        );
    }
}
