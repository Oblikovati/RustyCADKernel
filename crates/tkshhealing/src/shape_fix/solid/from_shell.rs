use super::{ShapeExtend, ShapeExtendStatus, ShapeFixSolid};
use rustycad_tkbrep::{
    brep::BRepBuilder,
    topods::{Shell, Solid},
};
use rustycad_tkernel::{precision::confusion, standard::Failure};
use rustycad_tkg3d::top_abs::State;
use rustycad_tktopalgo::brep_class3d::BRepClass3dSolidClassifier;
impl ShapeFixSolid {
    /// Build and orient a finite solid; `tool.solid_from_shell(&shell)?`.
    pub fn solid_from_shell(&self, shell: &Shell) -> Result<Solid, Failure> {
        shell.set_free(true)?;
        let solid = build_solid(shell)?;
        // OCCT catches classification failures and returns its already built solid.
        if infinite_state(&solid).ok() != Some(State::In) {
            return Ok(solid);
        }
        let mut reversed = shell.clone();
        reversed.set_free(true)?;
        reversed.reverse();
        let result = build_solid(&reversed)?;
        self.fields.write().status |= ShapeExtend::encode_status(ShapeExtendStatus::Done2);
        Ok(result)
    }
}
pub(super) fn build_solid(shell: &Shell) -> Result<Solid, Failure> {
    let builder = BRepBuilder::default();
    let mut solid = Solid::default();
    builder.make_solid(&mut solid);
    builder.add(&mut solid, shell)?;
    Ok(solid)
}
pub(super) fn infinite_state(solid: &Solid) -> Result<State, Failure> {
    let mut classifier = BRepClass3dSolidClassifier::with_shape(solid)?;
    classifier.perform_infinite_point(confusion())?;
    Ok(classifier.state())
}
