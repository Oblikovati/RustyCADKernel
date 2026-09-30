use super::Tools;
use crate::int_tools::Curve;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dTrimmedCurve};
use rustycad_tkg3d::geom::{GeomCurve, GeomLine, GeomTrimmedCurve};
use rustycad_tkmath::{geom_abs::CurveType, gp::Direction3};
impl Tools {
    /// Appends two trimmed halves only for a bounded closed curve; `Tools::split_curve(&curve, &mut curves)?`.
    pub fn split_curve(curve: &Curve, output: &mut Vec<Curve>) -> Result<i32, Failure> {
        let geom = curve.curve();
        if geom.is_null() || !Self::is_closed(geom)? {
            return Ok(0);
        }
        let (first, last) = (geom.first_parameter(), geom.last_parameter());
        let middle = if matches!(
            curve.curve_type()?,
            CurveType::BSplineCurve | CurveType::BezierCurve
        ) {
            Self::intermediate_point(first, last)
        } else {
            0.5 * (first + last)
        };
        let first_half = trim_intersection(curve, first, middle)?;
        let last_half = trim_intersection(curve, middle, last)?;
        output.push(first_half);
        output.push(last_half);
        Ok(2)
    }
    /// Unwraps input trimmed curves and retains the first two differing directions; `Tools::reject_lines(&mut input, &mut output)?`.
    pub fn reject_lines(input: &mut [Curve], output: &mut Vec<Curve>) -> Result<(), Failure> {
        output.clear();
        let mut direction = None;
        for index in 0..input.len() {
            let Some(next) = unwrap_line(&mut input[index])? else {
                output.clear();
                output.extend_from_slice(input);
                return Ok(());
            };
            let Some(first) = direction else {
                output.push(input[index].clone());
                direction = Some(next);
                continue;
            };
            if !Self::is_dirs_coinside(first, next) {
                output.push(input[index].clone());
                return Ok(());
            }
        }
        Ok(())
    }
}
fn unwrap_line(curve: &mut Curve) -> Result<Option<Direction3>, Failure> {
    let trimmed = curve.curve().cast::<GeomTrimmedCurve>();
    if !trimmed.is_null() {
        curve.set_curve(&trimmed.basis_curve());
    }
    let line = curve.curve().cast::<GeomLine>();
    if line.is_null() {
        return Ok(None);
    }
    Ok(Some(line.lin().direction()))
}
fn trim_intersection(curve: &Curve, first: f64, last: f64) -> Result<Curve, Failure> {
    let geom: Handle<dyn GeomCurve> = Handle::new(GeomTrimmedCurve::new(
        curve.curve(),
        first,
        last,
        true,
        true,
    )?)
    .into_cast();
    let a = trim_pcurve(curve.first_curve2d(), first, last)?;
    let b = trim_pcurve(curve.second_curve2d(), first, last)?;
    Ok(Curve::new(
        &geom,
        &a,
        &b,
        curve.tolerance(),
        curve.tangential_tolerance(),
    ))
}
fn trim_pcurve(
    curve: &Handle<dyn Geom2dCurve>,
    first: f64,
    last: f64,
) -> Result<Handle<dyn Geom2dCurve>, Failure> {
    if curve.is_null() {
        return Ok(Handle::default());
    }
    Ok(Handle::new(Geom2dTrimmedCurve::new(curve, first, last, true, true)?).into_cast())
}
