use rustycad_tkbrep::topods::Solid;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::State;
use rustycad_tktopalgo::brep_class3d::BRepClass3dSolidClassifier;
/// Classify the infinite point at the native 1e-7 tolerance; `is_inverted_solid(&solid)?`.
pub fn is_inverted_solid(solid: &Solid) -> Result<bool, Failure> {
    let mut classifier = BRepClass3dSolidClassifier::with_shape(solid)?;
    classifier.perform_infinite_point(1e-7)?;
    Ok(classifier.state() == State::In)
}
