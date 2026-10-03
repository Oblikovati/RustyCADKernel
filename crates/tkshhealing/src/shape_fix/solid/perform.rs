use super::{
    FixRoot, ShapeExtend, ShapeExtendStatus, ShapeFixSolid, create::create_solids,
    from_shell::build_solid,
};
use crate::{
    shape_analysis::ShapeAnalysisFreeBounds, shape_build::ShapeBuildReShape, shape_fix::need_fix,
};
use rustycad_tkbrep::{
    brep::BRepBuilder,
    brep_tools::ReShape,
    topexp::Explorer,
    topods::{self, Compound, Shape, ShapeIterator},
};
use rustycad_tkernel::{
    message::{Msg, ProgressRange, ProgressScope},
    standard::{Failure, Handle},
    tcollection::AsciiString,
};
use rustycad_tkg3d::top_abs::ShapeEnum;
impl ShapeFixSolid {
    /// Repair shells and solid orientation; `tool.perform()?`.
    pub fn perform(&self) -> Result<bool, Failure> {
        self.perform_with_progress(&ProgressRange::default())
    }
    /// Repair with OCCT cancellation points; `tool.perform_with_progress(&progress)?`.
    pub fn perform_with_progress(&self, progress: &ProgressRange<'_>) -> Result<bool, Failure> {
        if self.context().is_null() {
            self.set_context(&Handle::new(ShapeBuildReShape::new()));
        }
        self.shell_tool.set_context(&self.context());
        let shape = self.context().apply(&self.shape(), ShapeEnum::Shape)?;
        let scope = ProgressScope::with_literal(progress, "Fixing solid stage", 2., false);
        let Some((mut changed, count)) = self.fix_shells(&shape, &scope)? else {
            return Ok(false);
        };
        if need_fix(self.fix_shell_orientation_mode(), true) {
            if count == 1 {
                changed |= self.fix_single_shell()?;
            } else if !self.fix_multiple_shells(&scope)? {
                return Ok(false);
            }
        }
        let result = self.context().apply(&self.shape(), ShapeEnum::Shape)?;
        self.parent
            .fix_root_data()
            .with_mut(|fields| fields.shape = result);
        Ok(changed)
    }
    fn fix_shells(
        &self,
        shape: &Shape,
        scope: &ProgressScope<'_>,
    ) -> Result<Option<(bool, i32)>, Failure> {
        let shells = Explorer::new(shape, ShapeEnum::Shell, ShapeEnum::Shape)?.collect::<Vec<_>>();
        if !need_fix(self.fix_shell_mode(), true) {
            return Ok(Some((false, shells.len() as i32)));
        }
        let range = scope.next(1.);
        let shell_scope =
            ProgressScope::with_literal(&range, "Fixing shell", shells.len() as f64, false);
        let (mut changed, mut count) = (false, 0);
        for shell in shells {
            if !shell_scope.more() {
                break;
            }
            self.shell_tool.init(&topods::shell(&shell)?);
            if self
                .shell_tool
                .perform_with_progress(&shell_scope.next(1.))?
            {
                changed = true;
                self.fields.write().status |= ShapeExtend::encode_status(ShapeExtendStatus::Done1);
            }
            count += self.shell_tool.nb_shells();
        }
        Ok(shell_scope.more().then_some((changed, count)))
    }
    fn fix_single_shell(&self) -> Result<bool, Failure> {
        let shape = self.context().apply(&self.shape(), ShapeEnum::Shape)?;
        let explorer = Explorer::new(&shape, ShapeEnum::Shell, ShapeEnum::Shape)?;
        let closed = if explorer.more() {
            set_closed_flag(explorer.current())?
        } else {
            false
        };
        if closed || self.create_open_solid_mode() {
            self.store_single_solid(shape)?;
            return Ok(false);
        }
        self.fields.write().status |= ShapeExtend::encode_status(ShapeExtendStatus::Done3);
        self.context()
            .replace(&shape, ShapeIterator::new(&shape, false, true).value()?)?;
        self.solid_message("FixAdvSolid.FixShell.MSG10", true);
        Ok(true)
    }
    fn store_single_solid(&self, mut shape: Shape) -> Result<(), Failure> {
        let children = ShapeIterator::new(&shape, true, true);
        if children.more() && children.value()?.shape_type()? == ShapeEnum::Shell {
            let solid = self.solid_from_shell(&topods::shell(children.value()?)?)?;
            if self.status_flag(ShapeExtendStatus::Done2) {
                self.solid_message("FixAdvSolid.FixOrientation.MSG20", false);
                self.context().replace(&shape, &solid)?;
                shape = solid.into_shape();
            }
        }
        self.fields.write().solid = topods::solid(&shape)?.into_shape();
        Ok(())
    }
    fn fix_multiple_shells(&self, scope: &ProgressScope<'_>) -> Result<bool, Failure> {
        let shape = self.context().apply(&self.shape(), ShapeEnum::Shape)?;
        let (changed, solids) = create_solids(&shape)?;
        if !changed {
            return Ok(true);
        }
        self.solid_message("FixAdvSolid.FixOrientation.MSG20", false);
        if solids.len() == 1 {
            let result = self.single_created_solid(&shape, &solids[0])?;
            self.fields.write().solid = result.clone();
            self.context().replace(&shape, &result)?;
        } else if solids.len() > 1 {
            return self.store_created_compound(&shape, &solids, scope);
        }
        Ok(true)
    }
    fn single_created_solid(&self, source: &Shape, result: &Shape) -> Result<Shape, Failure> {
        if source.shape_type()? == ShapeEnum::Shell && self.create_open_solid_mode() {
            return Ok(build_solid(&topods::shell(result)?)?.into_shape());
        }
        if result.shape_type()? == ShapeEnum::Shell {
            self.solid_message("FixAdvSolid.FixShell.MSG10", true);
        }
        Ok(result.clone())
    }
    fn store_created_compound(
        &self,
        source: &Shape,
        solids: &[Shape],
        scope: &ProgressScope<'_>,
    ) -> Result<bool, Failure> {
        self.solid_message("FixAdvSolid.FixOrientation.MSG30", false);
        let builder = BRepBuilder::default();
        let mut compound = Compound::default();
        builder.make_compound(&mut compound);
        let range = scope.next(1.);
        let creation =
            ProgressScope::with_literal(&range, "Creating solid", solids.len() as f64, false);
        for solid in solids {
            if !creation.more() {
                break;
            }
            builder.add(&mut compound, &self.created_compound_child(source, solid)?)?;
            creation.next(1.);
        }
        if !creation.more() {
            return Ok(false);
        }
        self.context().replace(source, &compound)?;
        Ok(true)
    }
    fn created_compound_child(&self, source: &Shape, solid: &Shape) -> Result<Shape, Failure> {
        if source.shape_type()? != ShapeEnum::Shell {
            return Ok(solid.clone());
        }
        if self.create_open_solid_mode() {
            solid.set_closed(false)?;
            return Ok(build_solid(&topods::shell(solid)?)?.into_shape());
        }
        self.solid_message("FixAdvSolid.FixShell.MSG10", true);
        Ok(solid.clone())
    }
    fn solid_message(&self, key: &str, fail: bool) {
        let message = Msg::from_key(&AsciiString::from(key));
        if fail {
            self.send_fail_on_shape(&message);
        } else {
            self.send_warning_on_shape(&message);
        }
    }
}
fn set_closed_flag(shell: &Shape) -> Result<bool, Failure> {
    let free = ShapeAnalysisFreeBounds::from_shape(shell, false, true, false)?;
    let closed = !Explorer::new(free.closed_wires(), ShapeEnum::Edge, ShapeEnum::Shape)?.more()
        && !Explorer::new(free.open_wires(), ShapeEnum::Edge, ShapeEnum::Shape)?.more();
    shell.set_closed(closed)?;
    Ok(closed)
}
