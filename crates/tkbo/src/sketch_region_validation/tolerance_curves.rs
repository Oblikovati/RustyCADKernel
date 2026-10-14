// SPDX-License-Identifier: GPL-2.0-only
//! Capture real immutable coefficient values, never rounded line endpoints.
use super::tolerance_types::PlanarMetricCause;
use rustycad_tkernel::standard::Handle;
use rustycad_tkg2d::geom2d::{Geom2dBezierCurve, Geom2dCurve, Geom2dLine};
use rustycad_tkg3d::geom::{GeomBezierCurve, GeomCurve, GeomLine};
use rustycad_tkmath::predicates::roots::metric_bounds::{MetricBezier, MetricCurve};
pub(super) enum CapturedMetricCurve<const DIM: usize> {
    Bezier {
        poles: Vec<[f64; DIM]>,
        weights: Option<Vec<f64>>,
    },
    Line {
        origin: [f64; DIM],
        direction: [f64; DIM],
    },
}
impl<const DIM: usize> CapturedMetricCurve<DIM> {
    pub(super) fn borrow(&self) -> MetricCurve<'_, DIM> {
        match self {
            Self::Bezier { poles, weights } => MetricCurve::Bezier(MetricBezier {
                poles,
                weights: weights.as_deref(),
            }),
            Self::Line { origin, direction } => MetricCurve::Line {
                origin: *origin,
                direction: *direction,
            },
        }
    }
}
pub(super) fn capture_space_curve(
    curve: &Handle<dyn GeomCurve>,
) -> Result<CapturedMetricCurve<3>, PlanarMetricCause> {
    if let Some(bezier) = curve.cast::<GeomBezierCurve>().get() {
        return space_bezier(bezier);
    }
    if let Some(line) = curve.cast::<GeomLine>().get() {
        let frame = line.position();
        let p = frame.location();
        let d = frame.direction();
        return Ok(CapturedMetricCurve::Line {
            origin: [p.x(), p.y(), p.z()],
            direction: [d.x(), d.y(), d.z()],
        });
    }
    unsupported_curve(curve, "3D curve")
}
pub(super) fn capture_plane_curve(
    curve: &Handle<dyn Geom2dCurve>,
) -> Result<CapturedMetricCurve<2>, PlanarMetricCause> {
    if let Some(bezier) = curve.cast::<Geom2dBezierCurve>().get() {
        return plane_bezier(bezier);
    }
    if let Some(line) = curve.cast::<Geom2dLine>().get() {
        let p = line.location();
        let d = line.direction();
        return Ok(CapturedMetricCurve::Line {
            origin: [p.x(), p.y()],
            direction: [d.x(), d.y()],
        });
    }
    unsupported_curve(curve, "pcurve")
}
fn unsupported_curve<
    T: ?Sized + rustycad_tkernel::standard::Transient + rustycad_tkernel::standard::RuntimeTyped,
    const DIM: usize,
>(
    curve: &Handle<T>,
    representation: &'static str,
) -> Result<CapturedMetricCurve<DIM>, PlanarMetricCause> {
    let Some(curve) = curve.get() else {
        return Err(PlanarMetricCause::MissingCurve { representation });
    };
    Err(PlanarMetricCause::Unsupported {
        representation,
        dynamic_type: curve.dynamic_type().name().to_owned(),
    })
}
fn space_bezier(curve: &GeomBezierCurve) -> Result<CapturedMetricCurve<3>, PlanarMetricCause> {
    capture_bezier_coefficients(curve.nb_poles(), curve.is_rational(), |index| {
        let pole = curve.pole(index)?;
        Ok(([pole.x(), pole.y(), pole.z()], curve.weight(index)?))
    })
}
fn plane_bezier(curve: &Geom2dBezierCurve) -> Result<CapturedMetricCurve<2>, PlanarMetricCause> {
    capture_bezier_coefficients(curve.nb_poles(), curve.is_rational(), |index| {
        let pole = curve.pole(index)?;
        Ok(([pole.x(), pole.y()], curve.weight(index)?))
    })
}
fn capture_bezier_coefficients<const DIM: usize>(
    count: i32,
    rational: bool,
    mut coefficient: impl FnMut(i32) -> Result<([f64; DIM], f64), PlanarMetricCause>,
) -> Result<CapturedMetricCurve<DIM>, PlanarMetricCause> {
    let mut poles = Vec::new();
    let mut weights = Vec::new();
    for index in 1..=count {
        let (pole, weight) = coefficient(index)?;
        poles.push(pole);
        weights.push(weight);
    }
    Ok(CapturedMetricCurve::Bezier {
        poles,
        weights: rational.then_some(weights),
    })
}
