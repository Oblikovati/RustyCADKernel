//! ShapeConstruct_Curve oracle twin: adjustments, conversions, identity and knot mutation.
use super::brep_lib_common::{geom_curve, geom2d_curve};
use super::common::{Reader, attempt, end, flag, int, real, show};
use super::sh_common::{curve, curve2d};
use rustycad_tkernel::ncollection::{Array1, HArray1};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::{Geom2dBSplineCurve, Geom2dCurve};
use rustycad_tkg3d::geom::{GeomBSplineCurve, GeomCurve};
use rustycad_tkshhealing::shape_construct::ShapeConstructCurve;

/// Print one independent request; `construct_curve_case(&[5., 1., 2., 0., 0.])` fixes knots.
pub fn construct_curve_case(values: &[f64]) {
    let mut reader = Reader::new(values);
    let operation = reader.integer();
    match operation {
        0 | 1 => construct_adjust_3d(&mut reader, operation).unwrap(),
        2 => construct_adjust_2d(&mut reader).unwrap(),
        3 => construct_convert_3d(&mut reader).unwrap(),
        4 => construct_convert_2d(&mut reader).unwrap(),
        _ => construct_knots(&mut reader, operation == 6),
    }
    end();
}

fn construct_adjust_3d(reader: &mut Reader<'_>, operation: i32) -> Result<(), Failure> {
    let input = geom_curve(reader)?;
    let (first, last) = (reader.pnt(), reader.pnt());
    let (a, b) = (reader.real(), reader.real());
    let result = if operation == 0 {
        ShapeConstructCurve.adjust_curve(&input, first, last, a != 0., b != 0.)
    } else {
        ShapeConstructCurve.adjust_curve_segment(&input, first, last, a, b)
    };
    show(result, flag);
    curve(&input);
    construct_spline_3d(&input);
    Ok(())
}

fn construct_adjust_2d(reader: &mut Reader<'_>) -> Result<(), Failure> {
    let input = geom2d_curve(reader)?;
    let (first, last) = (reader.pnt2d(), reader.pnt2d());
    let (take_first, take_last) = (reader.flag(), reader.flag());
    show(
        ShapeConstructCurve.adjust_curve_2d(&input, first, last, take_first, take_last),
        flag,
    );
    curve2d(&input);
    construct_spline_2d(&input);
    Ok(())
}

fn construct_convert_3d(reader: &mut Reader<'_>) -> Result<(), Failure> {
    let input = geom_curve(reader)?;
    let (first, last, precision) = (reader.real(), reader.real(), reader.real());
    attempt(|| {
        let converted = ShapeConstructCurve.convert_to_bspline(&input, first, last, precision)?;
        flag(converted.object_address() == input.object_address());
        curve(&converted.cast());
        construct_spline_3d(&converted.cast());
        Ok(())
    });
    curve(&input);
    construct_spline_3d(&input);
    Ok(())
}

fn construct_convert_2d(reader: &mut Reader<'_>) -> Result<(), Failure> {
    let input = geom2d_curve(reader)?;
    let (first, last, precision) = (reader.real(), reader.real(), reader.real());
    attempt(|| {
        let converted =
            ShapeConstructCurve.convert_to_bspline_2d(&input, first, last, precision)?;
        flag(converted.object_address() == input.object_address());
        curve2d(&converted.cast());
        construct_spline_2d(&converted.cast());
        Ok(())
    });
    curve2d(&input);
    construct_spline_2d(&input);
    Ok(())
}

fn construct_knots(reader: &mut Reader<'_>, use_handle: bool) {
    let (lower, count) = (reader.integer(), reader.integer());
    let values: Vec<f64> = (0..count).map(|_| reader.real()).collect();
    let mut array = Array1::from_shared(&values, lower);
    if use_handle {
        let handle = Handle::new(HArray1::from_array(&array));
        show(ShapeConstructCurve::fix_knots_handle(&handle), flag);
        handle.array1().iter().copied().for_each(real);
        return;
    }
    show(ShapeConstructCurve::fix_knots(&mut array), flag);
    array.iter().copied().for_each(real);
}

fn construct_spline_3d(input: &Handle<dyn GeomCurve>) {
    let handle = input.cast::<GeomBSplineCurve>();
    let Some(spline) = handle.get() else {
        return;
    };
    int(spline.degree());
    int(spline.nb_poles());
    int(spline.nb_knots());
    flag(spline.is_periodic());
    for index in 1..=spline.nb_knots() {
        real(spline.knot(index).unwrap());
        int(spline.multiplicity(index).unwrap());
    }
    for index in 1..=spline.nb_poles() {
        super::common::pnt(spline.pole(index).unwrap());
        real(spline.weight(index).unwrap());
    }
}

pub(super) fn construct_spline_2d(input: &Handle<dyn Geom2dCurve>) {
    let handle = input.cast::<Geom2dBSplineCurve>();
    let Some(spline) = handle.get() else {
        return;
    };
    int(spline.degree());
    int(spline.nb_poles());
    int(spline.nb_knots());
    flag(spline.is_periodic());
    for index in 1..=spline.nb_knots() {
        real(spline.knot(index).unwrap());
        int(spline.multiplicity(index).unwrap());
    }
    for index in 1..=spline.nb_poles() {
        super::common::pnt2d(spline.pole(index).unwrap());
        real(spline.weight(index).unwrap());
    }
}
