//! Clips plane-plane candidates against straight-pcurve face boundaries.
mod events;
mod intervals;
mod trimmed_curve;
use rustycad_tkbo::int_tools::{Curve, FClass2d};
use rustycad_tkbrep::topods::Face;
use rustycad_tkernel::{precision, standard::Failure};
use rustycad_tkg3d::top_abs::State;

pub(super) fn clip_curves(
    curves: &[Curve],
    first: &Face,
    second: &Face,
) -> Result<Vec<Curve>, Failure> {
    let mut clipped = Vec::new();
    for curve in curves {
        clipped.extend(clip_curve(curve, first, second)?);
    }
    Ok(clipped)
}

fn clip_curve(curve: &Curve, first: &Face, second: &Face) -> Result<Vec<Curve>, Failure> {
    let range = (
        curve.curve().first_parameter(),
        curve.curve().last_parameter(),
    );
    if !range.0.is_finite() || !range.1.is_finite() {
        return Err(not_implemented("unbounded section curve clipping"));
    }
    let mut events = vec![range.0, range.1];
    events.extend(events::boundary_events(curve, first, 0, range)?);
    events.extend(events::boundary_events(curve, second, 1, range)?);
    intervals::normalize_events(&mut events);
    let first_classifier = FClass2d::new(first, precision::confusion())?;
    let second_classifier = FClass2d::new(second, precision::confusion())?;
    let intervals =
        intervals::inside_intervals(curve, &events, &first_classifier, &second_classifier)?;
    intervals
        .into_iter()
        .map(|interval| trimmed_curve::trim(curve, interval))
        .collect()
}

pub(super) fn inside_both(
    curve: &Curve,
    parameter: f64,
    first: &FClass2d,
    second: &FClass2d,
) -> Result<bool, Failure> {
    let first_uv = required_pcurve(curve.first_curve2d())?.eval_d0(parameter)?;
    let second_uv = required_pcurve(curve.second_curve2d())?.eval_d0(parameter)?;
    Ok(
        first.perform(first_uv, true)? == State::In
            && second.perform(second_uv, true)? == State::In,
    )
}

fn required_pcurve(
    curve: &rustycad_tkernel::standard::Handle<dyn rustycad_tkg2d::geom2d::Geom2dCurve>,
) -> Result<&dyn rustycad_tkg2d::geom2d::Geom2dCurve, Failure> {
    curve
        .get()
        .ok_or_else(|| not_implemented("section curve with both face pcurves"))
}

fn not_implemented(branch: &str) -> Failure {
    Failure::with_message(
        "Standard_NotImplemented",
        &format!("BRepOffset_Inter3d::FaceInter does not support {branch}"),
    )
}

#[cfg(test)]
mod tests {
    use super::intervals::normalize_events;

    #[test]
    fn normalizes_repeated_boundary_parameters() {
        let mut events = vec![0.0, 1.0, 0.0, 1.0 + 1.0e-10];
        normalize_events(&mut events);
        assert_eq!(events, vec![0.0, 1.0]);
    }
}
