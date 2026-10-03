//! Native CreateNonManifoldShells replacement-chain ordering.
use super::{build_groups::empty_shell, groups::ShapeMap};
use rustycad_tkbrep::{
    brep::BRepBuilder,
    topexp::Explorer,
    topods::{self, Shape, Shell},
    toptools::{MapOfShape, ShapeMapHasher},
};
use rustycad_tkernel::{ncollection::IndexedDataMap, standard::Failure};
use rustycad_tkg3d::top_abs::ShapeEnum;
pub(super) fn combine_non_manifold(
    shells: &mut Vec<Shell>,
    multiple: &MapOfShape,
) -> Result<(), Failure> {
    let adjacency = shared_shells(shells, multiple)?;
    let mut replacements = ShapeMap::default();
    for index in 1..=adjacency.extent() {
        merge_group(adjacency.find_from_index(index)?, &mut replacements)?;
    }
    let (mut retained, mut appended) = (Vec::new(), Vec::new());
    let mut seen = MapOfShape::default();
    for shell in shells.iter() {
        if replacements.is_bound(shell) {
            let result = resolve(shell, &replacements)?;
            if seen.add(result.clone()) {
                appended.push(topods::shell(&result)?);
            }
        } else {
            retained.push(shell.clone());
        }
    }
    retained.extend(appended);
    *shells = retained;
    Ok(())
}
fn shared_shells(
    shells: &[Shell],
    multiple: &MapOfShape,
) -> Result<IndexedDataMap<Shape, Vec<Shell>, ShapeMapHasher>, Failure> {
    let mut adjacency = IndexedDataMap::default();
    for shell in shells {
        let mut present = MapOfShape::default();
        for edge in Explorer::new(shell, ShapeEnum::Edge, ShapeEnum::Shape)? {
            present.add(edge);
        }
        for edge in multiple.iter().filter(|edge| present.contains(edge)) {
            adjacency
                .try_emplaced(edge.clone(), Vec::new)
                .push(shell.clone());
        }
    }
    Ok(adjacency)
}
fn merge_group(shells: &[Shell], replacements: &mut ShapeMap<Shape>) -> Result<(), Failure> {
    let mut merged = empty_shell();
    let mut included = MapOfShape::default();
    let mut changed = false;
    for shell in shells {
        if replacements.is_bound(shell) {
            let resolved = resolve(shell, replacements)?;
            if included.add(resolved.clone()) {
                append_faces(&mut merged, &resolved, false)?;
            }
        } else {
            append_faces(&mut merged, shell, false)?;
            included.add(shell.clone().into_shape());
            changed = true;
        }
    }
    if included.extent() > 1 || changed {
        for old in included.iter() {
            replacements.bind(old.clone(), merged.clone().into_shape());
        }
    }
    Ok(())
}
fn resolve(shape: &Shape, replacements: &ShapeMap<Shape>) -> Result<Shape, Failure> {
    let mut current = replacements.find(shape)?.clone();
    while let Some(next) = replacements.seek(&current) {
        if next.is_same(&current) {
            break;
        }
        current = next.clone();
    }
    Ok(current)
}
pub(super) fn append_faces(shell: &mut Shell, shape: &Shape, reverse: bool) -> Result<(), Failure> {
    for face in Explorer::new(shape, ShapeEnum::Face, ShapeEnum::Shape)? {
        let face = if reverse { face.reversed() } else { face };
        BRepBuilder::default().add(shell, &face)?;
    }
    Ok(())
}
