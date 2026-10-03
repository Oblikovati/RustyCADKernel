use super::{FixRoot, ShapeExtend, ShapeExtendStatus, ShapeFixShell};
use crate::{
    shape_analysis::ShapeAnalysisShell, shape_build::ShapeBuildReShape, shape_fix::need_fix,
};
use rustycad_tkbrep::{
    brep_tools::ReShape,
    topexp::Explorer,
    topods::{self, Shape, ShapeIterator},
};
use rustycad_tkernel::{
    message::{ProgressRange, ProgressScope},
    standard::{Failure, Handle},
};
use rustycad_tkg3d::top_abs::ShapeEnum;
impl ShapeFixShell {
    /// Heal faces and shell orientation; `tool.perform()?`.
    pub fn perform(&self) -> Result<bool, Failure> {
        self.perform_with_progress(&ProgressRange::default())
    }
    /// Heal with native cancellation points; `tool.perform_with_progress(&progress)?`.
    pub fn perform_with_progress(&self, progress: &ProgressRange<'_>) -> Result<bool, Failure> {
        if self.context().is_null() {
            self.set_context(&Handle::new(ShapeBuildReShape::new()));
        }
        let context = self.context();
        self.face_tool.set_context(&context);
        let Some(changed) = self.fix_faces(progress)? else {
            return Ok(false);
        };
        let replaced = context.apply(&self.shell(), ShapeEnum::Shape)?;
        if need_fix(self.fix_orientation_mode(), true) {
            let non_manifold = self.state.read().non_manifold;
            self.fix_face_orientation(&topods::shell(&replaced)?, true, non_manifold)?;
        }
        self.check_closed_flags(&context.apply(&replaced, ShapeEnum::Shape)?)?;
        if changed {
            self.state.write().status |= ShapeExtend::encode_status(ShapeExtendStatus::Done1);
        }
        Ok(changed || self.status_flag(ShapeExtendStatus::Done2))
    }
    fn fix_faces(&self, progress: &ProgressRange<'_>) -> Result<Option<bool>, Failure> {
        if !need_fix(self.fix_face_mode(), true) {
            return Ok(Some(false));
        }
        let shape = self.context().apply(&self.shell(), ShapeEnum::Shape)?;
        let scope = ProgressScope::with_literal(
            progress,
            "Fixing face",
            f64::from(shape.nb_children()),
            false,
        );
        let mut changed = false;
        let mut faces = ShapeIterator::new(&shape, true, true);
        while faces.more() && scope.more() {
            let face = topods::face(faces.value()?)?;
            self.face_tool.init(face.into_shape());
            if self.face_tool.perform()? {
                changed = true;
                self.state.write().status |= ShapeExtend::encode_status(ShapeExtendStatus::Done1);
            }
            faces.next();
            scope.next(1.);
        }
        Ok(scope.more().then_some(changed))
    }
    fn check_closed_flags(&self, shape: &Shape) -> Result<(), Failure> {
        let mut analysis = ShapeAnalysisShell::new();
        for shell in Explorer::new(shape, ShapeEnum::Shell, ShapeEnum::Shape)? {
            if !shell.closed()? {
                continue;
            }
            analysis.load_shells(&shell)?;
            analysis.check_oriented_shells(&shell, true, false)?;
            if analysis.has_free_edges() {
                shell.set_closed(false)?;
                self.warn("FixAdvShell.FixClosedFlag.MSG0");
            }
            analysis.clear();
        }
        Ok(())
    }
}
