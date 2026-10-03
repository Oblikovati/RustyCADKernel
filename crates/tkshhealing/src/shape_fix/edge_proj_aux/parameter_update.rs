use super::EdgeProjAuxState;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::{Geom2dBSplineCurve, Geom2dBoundedCurve, Geom2dCurve};
use rustycad_tkmath::elclib;

pub(super) fn update_parameters(
    state: &mut EdgeProjAuxState,
    handle: &Handle<dyn Geom2dCurve>,
) -> Result<(), Failure> {
    if state.first_param < state.last_param {
        return Ok(());
    }
    let curve = require_curve(handle)?;
    let range = (curve.first_parameter(), curve.last_parameter());
    if require_curve(handle)?.is_periodic() {
        update_periodic_range(state, range);
        return Ok(());
    }
    update_nonperiodic_range(state, handle, curve, range)
}

fn require_curve(handle: &Handle<dyn Geom2dCurve>) -> Result<&dyn Geom2dCurve, Failure> {
    handle.get().ok_or_else(|| {
        Failure::with_message(
            "Standard_NullObject",
            "ShapeFix_EdgeProjAux: null p-curve; expected a live Geom2d_Curve",
        )
    })
}

fn update_periodic_range(state: &mut EdgeProjAuxState, range: (f64, f64)) {
    (state.first_param, state.last_param) = elclib::adjust_periodic(
        range.0,
        range.1,
        precision::p_confusion(),
        state.first_param,
        state.last_param,
    );
}

fn update_nonperiodic_range(
    state: &mut EdgeProjAuxState,
    handle: &Handle<dyn Geom2dCurve>,
    curve: &dyn Geom2dCurve,
    range: (f64, f64),
) -> Result<(), Failure> {
    if curve.is_closed()? || bspline_is_file_closed(handle)? {
        update_closed_range(state, range);
        return Ok(());
    }
    reverse_parameters(state, curve)?;
    curve.reverse()
}

fn bspline_is_file_closed(handle: &Handle<dyn Geom2dCurve>) -> Result<bool, Failure> {
    let bspline = handle.cast::<Geom2dBSplineCurve>();
    let Some(curve) = bspline.get() else {
        return Ok(false);
    };
    Ok(curve.start_point()?.distance(curve.end_point()?) <= precision::p_confusion())
}

fn update_closed_range(state: &mut EdgeProjAuxState, range: (f64, f64)) {
    let tolerance = precision::p_confusion();
    if (state.first_param - range.1).abs() <= tolerance {
        state.first_param = range.0;
    } else if (state.last_param - range.0).abs() <= tolerance {
        state.last_param = range.1;
    }
}

fn reverse_parameters(
    state: &mut EdgeProjAuxState,
    curve: &dyn Geom2dCurve,
) -> Result<(), Failure> {
    state.first_param = curve.reversed_parameter(state.first_param)?;
    state.last_param = curve.reversed_parameter(state.last_param)?;
    Ok(())
}
