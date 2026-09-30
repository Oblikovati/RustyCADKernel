use super::{fclass_faces, flag, integer};
use rustycad_tkbo::int_tools::FClass2d;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::Point2;
fn state(classifier: &FClass2d, p: Point2, v: &[f64]) -> Result<(), Failure> {
    flag(classifier.is_hole());
    integer(classifier.perform_infinite_point()? as i32);
    integer(classifier.perform(p, v[6] != 0.)? as i32);
    integer(classifier.test_on_restriction(p, v[5], v[6] != 0.)? as i32);
    Ok(())
}
fn run(v: &[f64]) -> Result<(), Failure> {
    let face = fclass_faces::face(v[0] as i32, v[1])?;
    let p = Point2::new(v[3], v[4]);
    let mut classifier = FClass2d::new(&face, v[2])?;
    state(&classifier, p, v)?;
    classifier.init(&face, v[2])?;
    state(&classifier, p, v)?;
    let other = fclass_faces::face(0, v[1] * 0.5)?;
    classifier.init(&other, v[2])?;
    state(&classifier, p, v)?;
    let mut fresh = FClass2d::default();
    integer(fresh.perform(p, true)? as i32);
    fresh.init(&face, v[2])?;
    state(&fresh, p, v)?;
    Ok(())
}
pub fn case(v: &[f64]) {
    if let Err(error) = run(v) {
        print!("{} ", error.exception_type());
    }
}
