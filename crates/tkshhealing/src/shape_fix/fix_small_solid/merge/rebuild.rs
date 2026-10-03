use super::FaceShellMap;
use rustycad_tkbrep::topods::{Builder, Compound, Shape, ShapeIterator};
use rustycad_tkbrep::toptools::MapOfShape;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::ShapeEnum;

pub(super) struct ShellMerge<'a> {
    pub(super) base_shell: &'a Shape,
    pub(super) shells_to_merge: &'a [Shape],
    pub(super) face_shells: &'a FaceShellMap,
    pub(super) new_free_faces: &'a mut FaceShellMap,
}

pub(super) fn merge_shells(input: ShellMerge<'_>) -> Result<Shape, Failure> {
    let mut shell = input.base_shell.empty_copied()?;
    let (mut remove_faces, mut free_faces) = (MapOfShape::default(), MapOfShape::default());
    collect_merge_faces(
        input.shells_to_merge,
        input.base_shell,
        input.face_shells,
        &mut shell,
        &mut remove_faces,
        &mut free_faces,
    )?;
    append_free_faces(&mut shell, &mut free_faces, input.new_free_faces)?;
    append_base_faces(&mut shell, input.base_shell, &remove_faces)?;
    if shell.nb_children() == 0 {
        return Ok(Shape::default());
    }
    Ok(shell)
}

fn collect_merge_faces(
    shells: &[Shape],
    base_shell: &Shape,
    face_shells: &FaceShellMap,
    result: &mut Shape,
    remove_faces: &mut MapOfShape,
    free_faces: &mut MapOfShape,
) -> Result<(), Failure> {
    for shell in shells {
        classify_shell_components(
            shell,
            base_shell,
            face_shells,
            result,
            remove_faces,
            free_faces,
        )?;
    }
    Ok(())
}

fn classify_shell_components(
    shell: &Shape,
    base_shell: &Shape,
    face_shells: &FaceShellMap,
    result: &mut Shape,
    remove_faces: &mut MapOfShape,
    free_faces: &mut MapOfShape,
) -> Result<(), Failure> {
    for face in direct_children(shell)? {
        classify_component(
            &face,
            base_shell,
            face_shells,
            result,
            remove_faces,
            free_faces,
        )?;
    }
    Ok(())
}

fn classify_component(
    face: &Shape,
    base_shell: &Shape,
    face_shells: &FaceShellMap,
    result: &mut Shape,
    remove_faces: &mut MapOfShape,
    free_faces: &mut MapOfShape,
) -> Result<(), Failure> {
    if face.shape_type()? != ShapeEnum::Face {
        return Builder.add(result, face);
    }
    classify_face(
        face,
        base_shell,
        face_shells,
        result,
        remove_faces,
        free_faces,
    )
}

fn classify_face(
    face: &Shape,
    base_shell: &Shape,
    face_shells: &FaceShellMap,
    result: &mut Shape,
    remove_faces: &mut MapOfShape,
    free_faces: &mut MapOfShape,
) -> Result<(), Failure> {
    let Some(outer) = face_shells.seek(face) else {
        toggle_free_face(face, free_faces);
        return Ok(());
    };
    if outer.is_same(base_shell) {
        remove_faces.add(face.clone());
        return Ok(());
    }
    Builder.add(result, face)
}

fn toggle_free_face(face: &Shape, free_faces: &mut MapOfShape) {
    if !free_faces.remove(face) {
        free_faces.add(face.clone());
    }
}

fn append_free_faces(
    shell: &mut Shape,
    free_faces: &mut MapOfShape,
    new_free_faces: &mut FaceShellMap,
) -> Result<(), Failure> {
    for face in free_faces.iter() {
        Builder.add(shell, face)?;
        new_free_faces.try_bind(face.clone(), shell.clone());
    }
    free_faces.clear(true);
    Ok(())
}

fn append_base_faces(
    result: &mut Shape,
    base_shell: &Shape,
    removed: &MapOfShape,
) -> Result<(), Failure> {
    for face in direct_children(base_shell)? {
        if !removed.contains(&face) {
            Builder.add(result, &face)?;
        }
    }
    Ok(())
}

pub(super) fn add_shells(base_shell: &Shape, shells_to_add: Vec<Shape>) -> Result<Shape, Failure> {
    let mut compound = Compound::default();
    Builder.make_compound(&mut compound);
    let mut result = compound.into_shape();
    if !base_shell.is_null() {
        Builder.add(&mut result, base_shell)?;
    }
    for shell in shells_to_add {
        Builder.add(&mut result, &shell)?;
    }
    Ok(result)
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
