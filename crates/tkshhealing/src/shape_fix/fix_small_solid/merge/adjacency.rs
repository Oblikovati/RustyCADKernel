use super::FaceShellMap;
use rustycad_tkbrep::topods::{Shape, ShapeIterator};
use rustycad_tkbrep::toptools::ShapeMapHasher;
use rustycad_tkernel::ncollection::DataMap;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::ShapeEnum;

#[derive(Default)]
pub(super) struct AdjacentShells {
    pub(super) outer: Shape,
    pub(super) selected: Shape,
    pub(super) remaining: Vec<Shape>,
}

pub(super) fn map_faces_to_shells(
    solid: &Shape,
    face_shells: &mut FaceShellMap,
) -> Result<(), Failure> {
    for shell in direct_children(solid)? {
        if shell.shape_type()? != ShapeEnum::Shell {
            continue;
        }
        unmap_shared_shell_faces(&shell, face_shells)?;
    }
    Ok(())
}

fn unmap_shared_shell_faces(shell: &Shape, face_shells: &mut FaceShellMap) -> Result<(), Failure> {
    for face in direct_children(shell)? {
        if face.shape_type()? != ShapeEnum::Face {
            continue;
        }
        if !face_shells.try_bind(face.clone(), shell.clone()) {
            face_shells.unbind(&face);
        }
    }
    Ok(())
}

pub(super) fn most_shared_shell(
    solid: &Shape,
    face_shells: &FaceShellMap,
) -> Result<Option<AdjacentShells>, Failure> {
    let mut state = SharedShellSearch::default();
    for shell in direct_children(solid)? {
        state.last_child = Some(shell.clone());
        if shell.shape_type()? != ShapeEnum::Shell {
            continue;
        }
        state.remaining.push(shell.clone());
        accumulate_shell_faces(&shell, face_shells, &mut state)?;
    }
    state.result()
}

#[derive(Default)]
struct SharedShellSearch {
    areas: DataMap<Shape, f64, ShapeMapHasher>,
    max_area: f64,
    outer: Option<Shape>,
    selected: Option<Shape>,
    last_child: Option<Shape>,
    remaining: Vec<Shape>,
}

impl SharedShellSearch {
    fn result(mut self) -> Result<Option<AdjacentShells>, Failure> {
        let Some(outer) = self.outer else {
            return Ok(None);
        };
        if self.selected.is_none() {
            return Ok(None);
        }
        // Native stores a pointer to TopoDS_Iterator::Value()'s reused slot, so a later
        // child overwrites the selected shell before it is copied to output.
        let selected = self.last_child.clone().ok_or_else(|| {
            Failure::new("ShapeFix_FixSmallSolid::Merge expected a direct solid child")
        })?;
        remove_selected_shell(&mut self.remaining, &selected)?;
        Ok(Some(AdjacentShells {
            outer,
            selected,
            remaining: self.remaining,
        }))
    }
}

fn remove_selected_shell(remaining: &mut Vec<Shape>, selected: &Shape) -> Result<(), Failure> {
    let Some(index) = remaining.iter().position(|shell| shell.is_same(selected)) else {
        let message = format!(
            "ShapeFix_FixSmallSolid::Merge native shell alias resolved to {:?}, expected a listed shell",
            selected.shape_type()?
        );
        return Err(Failure::with_type("Standard_Failure", &message, ""));
    };
    remaining.remove(index);
    Ok(())
}

fn accumulate_shell_faces(
    solid_shell: &Shape,
    face_shells: &FaceShellMap,
    state: &mut SharedShellSearch,
) -> Result<(), Failure> {
    for face in direct_children(solid_shell)? {
        accumulate_face(&face, solid_shell, face_shells, state)?;
    }
    Ok(())
}

fn accumulate_face(
    face: &Shape,
    solid_shell: &Shape,
    face_shells: &FaceShellMap,
    state: &mut SharedShellSearch,
) -> Result<(), Failure> {
    if face.shape_type()? != ShapeEnum::Face {
        return Ok(());
    }
    let Some(outer_shell) = face_shells.seek(face) else {
        return Ok(());
    };
    update_shared_area(state, outer_shell.clone(), solid_shell.clone(), area(face)?)
}

fn update_shared_area(
    state: &mut SharedShellSearch,
    outer: Shape,
    solid_shell: Shape,
    face_area: f64,
) -> Result<(), Failure> {
    let total_area = *state.areas.try_emplaced(outer.clone(), || 0.0) + face_area;
    *state.areas.change_find(&outer)? = total_area;
    if state.max_area < total_area {
        state.max_area = total_area;
        state.outer = Some(outer);
        state.selected = Some(solid_shell);
    }
    Ok(())
}

fn area(face: &Shape) -> Result<f64, Failure> {
    super::super::config::surface_area(face)
}

fn direct_children(shape: &Shape) -> Result<Vec<Shape>, Failure> {
    let mut iterator = ShapeIterator::new(shape, true, true);
    let mut children = Vec::new();
    while iterator.more() {
        children.push(iterator.value()?.clone());
        iterator.next();
    }
    Ok(children)
}
