//! Independent shell-sewing calls and persistent instance histories.
use super::brep_lib_common::shape;
use super::common::{Reader, attempt, end, flag, int, real, show};
#[path = "../../tests/shell_sewing_legacy/geometry.rs"]
mod geometry;
#[path = "../../tests/healing_support/mod.rs"]
mod healing_support;
use rustycad_tkbrep::{topexp::Explorer, topods::Shape};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkgeombase::gprop::GProps;
use rustycad_tkshhealing::shape_upgrade::ShapeUpgradeShellSewing;
use rustycad_tktopalgo::{
    brep_check::BRepCheckAnalyzer, brep_class3d::BRepClass3dSolidClassifier, brep_gprop::BRepGProp,
};

pub fn shell_sewing_case(values: &[f64]) {
    let mut reader = Reader::new(values);
    attempt(|| shell_sewing_request(&mut reader));
    end();
}
fn shell_sewing_request(reader: &mut Reader<'_>) -> Result<(), Failure> {
    let mut sewer = ShapeUpgradeShellSewing::new();
    let mut result = Shape::default();
    for _ in 0..reader.integer() {
        let input = sewing_input(reader);
        let tolerance = reader.real();
        let previous = reader.flag();
        let input = if previous { result.clone() } else { input };
        let applied = sewer.apply_sewing(&input, tolerance);
        show(applied, |output| result = output);
        attempt(|| sewing_result(&result, &input));
    }
    Ok(())
}
fn sewing_input(reader: &mut Reader<'_>) -> Shape {
    let kind = reader.integer();
    let gap = reader.real();
    let offset = reader.real();
    let scale = reader.real();
    let reverse = reader.flag();
    let shape = sewing_geometry(kind, gap, offset, scale);
    if reverse { shape.reversed() } else { shape }
}
fn sewing_geometry(kind: i32, gap: f64, offset: f64, scale: f64) -> Shape {
    match kind {
        0 => Shape::default(),
        1 => geometry::rectangle(offset, offset + scale),
        2 => healing_support::shell(&[]).into_shape(),
        3 => geometry::open_shell(gap),
        4 => geometry::box_shape(offset, (2. * scale, 3. * scale, 4. * scale)),
        7 => healing_support::shell(&[geometry::rectangle(offset, offset + scale)]).into_shape(),
        _ => sewing_compound(kind, gap, offset, scale),
    }
}
fn sewing_compound(kind: i32, gap: f64, offset: f64, scale: f64) -> Shape {
    let shapes = match kind {
        5 => [
            geometry::box_shape(0., (2., 3., 4.)),
            geometry::box_shape(offset, (scale, 2. * scale, 3. * scale)),
        ],
        6 => [
            geometry::open_shell(gap),
            geometry::rectangle(offset, offset + scale),
        ],
        _ => [
            geometry::rectangle(0., 1.),
            geometry::rectangle(offset, offset + scale),
        ],
    };
    healing_support::compound(&shapes).into_shape()
}

fn sewing_result(result: &Shape, input: &Shape) -> Result<(), Failure> {
    flag(result.is_same(input));
    shape(result)?;
    if result.is_null() {
        return Ok(());
    }
    let mut properties = GProps::default();
    BRepGProp::surface_properties(result, &mut properties, false, false)?;
    real(properties.mass());
    flag(BRepCheckAnalyzer::new(result, true, false, false)?.is_valid()?);
    for solid in Explorer::new(result, ShapeEnum::Solid, ShapeEnum::Shape)? {
        let mut classifier = BRepClass3dSolidClassifier::with_shape(&solid)?;
        classifier.perform_infinite_point(1e-6)?;
        int(classifier.state() as i32);
    }
    Ok(())
}
