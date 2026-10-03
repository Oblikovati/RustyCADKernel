use super::build_groups::{ShellGroups, build_shell_group, empty_shell};
use super::{FixRoot, ShapeExtend, ShapeExtendStatus, ShapeFixShell};
use rustycad_tkbrep::{
    brep::{BRepBuilder, BRepTool},
    brep_tools::ReShape,
    topexp,
    topods::{self, Compound, Face, Shape, ShapeIterator, Shell},
    toptools::{IndexedDataMapOfShapeListOfShape, MapOfShape},
};
use rustycad_tkernel::{message::Msg, standard::Failure, tcollection::AsciiString};
use rustycad_tkg3d::top_abs::ShapeEnum;
impl ShapeFixShell {
    /// Orient connected faces and split disconnected shells; `tool.fix_face_orientation(&shell,true,false)?`.
    pub fn fix_face_orientation(
        &self,
        shell: &Shell,
        account_multiple: bool,
        non_manifold: bool,
    ) -> Result<bool, Failure> {
        self.state.write().shell = shell.clone();
        self.set_shape(shell.clone().into_shape());
        let (mut faces, duplicates) = unique_faces(shell)?;
        let (multiple, free) = shell_boundaries(shell, account_multiple)?;
        if BRepTool::is_closed(shell)? == free {
            shell.set_closed(!free)?;
            self.warn("FixAdvShell.FixClosedFlag.MSG0");
        }
        let mut groups = ShellGroups {
            changed: duplicates,
            ..ShellGroups::default()
        };
        loop {
            let before = groups.shells.len();
            build_shell_group(&mut faces, &multiple, &mut groups)?;
            if faces.is_empty() || groups.shells.len() == before {
                break;
            }
        }
        groups.changed |= groups.shells.len() > 1;
        self.finish_groups(shell, faces, multiple, non_manifold, groups)
    }
    fn finish_groups(
        &self,
        shell: &Shell,
        mut faces: Vec<Face>,
        multiple: MapOfShape,
        non_manifold: bool,
        mut groups: ShellGroups,
    ) -> Result<bool, Failure> {
        if !faces.is_empty() && !groups.shells.is_empty() && !multiple.is_empty() {
            let added = super::multiple_faces::add_multiple_faces(
                shell,
                &mut faces,
                &multiple,
                &mut groups,
                non_manifold,
            )?;
            groups.changed |= added;
        }
        if !groups.errors.is_empty() {
            return self.store_errors(&groups);
        }
        if groups.shells.len() > 1 {
            partition_and_glue(&mut groups.shells, &multiple)?;
        }
        for face in faces {
            let mut part = empty_shell();
            BRepBuilder::default().add(&mut part, &face)?;
            groups.shells.push(part);
        }
        if non_manifold && groups.shells.len() > 1 {
            super::non_manifold::combine_non_manifold(&mut groups.shells, &multiple)?;
        }
        groups.changed |= groups.shells.len() > 1;
        self.store_shells(&groups.shells)?;
        if !groups.changed {
            return Ok(false);
        }
        self.state.write().status = ShapeExtend::encode_status(ShapeExtendStatus::Done2);
        let context = self.context();
        if !context.is_null() {
            context.replace(shell, &self.shape())?;
        }
        self.warn(if self.nb_shells() == 1 {
            "FixAdvShell.FixOrientation.MSG0"
        } else {
            "FixAdvShell.FixOrientation.MSG30"
        });
        Ok(true)
    }
    fn store_shells(&self, shells: &[Shell]) -> Result<(), Failure> {
        self.state.write().count = shells.len() as i32;
        if shells.len() == 1 {
            self.state.write().shell = shells[0].clone();
            self.set_shape(shells[0].clone().into_shape());
            return Ok(());
        }
        let shapes = shells
            .iter()
            .map(|shell| shell.clone().into_shape())
            .collect::<Vec<_>>();
        self.set_shape(compound(&shapes)?.into_shape());
        Ok(())
    }
    fn store_errors(&self, groups: &ShellGroups) -> Result<bool, Failure> {
        let errors = groups.errors.to_vec();
        self.state.write().errors = compound(&errors)?;
        if !groups.shells.is_empty() {
            let mut shapes = groups
                .shells
                .iter()
                .map(|shell| shell.clone().into_shape())
                .collect::<Vec<_>>();
            for face in &errors {
                let mut shell = empty_shell();
                BRepBuilder::default().add(&mut shell, face)?;
                shapes.push(shell.into_shape());
            }
            self.set_shape(compound(&shapes)?.into_shape());
        }
        self.state.write().status = ShapeExtend::encode_status(ShapeExtendStatus::Fail);
        self.warn("FixAdvShell.FixOrientation.MSG20");
        Ok(true)
    }
    fn set_shape(&self, shape: Shape) {
        self.fix_root_data().with_mut(|fields| fields.shape = shape);
    }
    pub(super) fn warn(&self, key: &str) {
        self.send_warning_on_shape(&Msg::from_key(&AsciiString::from(key)));
    }
}
fn unique_faces(shell: &Shell) -> Result<(Vec<Face>, bool), Failure> {
    let mut seen = MapOfShape::default();
    let mut faces = Vec::new();
    let mut duplicates = false;
    for shape in ShapeIterator::new(shell, true, true) {
        if seen.add(shape.clone()) {
            faces.push(topods::face(&shape)?);
        } else {
            duplicates = true;
        }
    }
    Ok((faces, duplicates))
}
fn shell_boundaries(shell: &Shell, account_multiple: bool) -> Result<(MapOfShape, bool), Failure> {
    let mut ancestors = IndexedDataMapOfShapeListOfShape::default();
    topexp::map_shapes_and_ancestors(shell, ShapeEnum::Edge, ShapeEnum::Face, &mut ancestors)?;
    let mut multiple = MapOfShape::default();
    let mut free = false;
    for index in 1..=ancestors.extent() {
        let count = ancestors.find_from_index(index)?.extent();
        let edge = ancestors.find_key(index)?;
        if !free && count == 1 {
            if !BRepTool::degenerated(&topods::edge(edge)?)? {
                free = true;
            }
        } else if account_multiple && count > 2 {
            multiple.add(edge.clone());
        }
    }
    Ok((multiple, free))
}
fn compound(shapes: &[Shape]) -> Result<Compound, Failure> {
    let mut result = Compound::default();
    let builder = BRepBuilder::default();
    builder.make_compound(&mut result);
    for shape in shapes {
        builder.add(&mut result, shape)?;
    }
    Ok(result)
}
fn partition_and_glue(shells: &mut Vec<Shell>, multiple: &MapOfShape) -> Result<(), Failure> {
    let mut open = Vec::new();
    let mut closed = Vec::new();
    for shell in shells.drain(..) {
        if BRepTool::is_closed(&shell)? {
            closed.push(shell);
        } else {
            open.push(shell);
        }
    }
    if open.len() > 1 {
        super::glue::create_closed_shells(&mut open, multiple)?;
    }
    closed.extend(open);
    *shells = closed;
    Ok(())
}
