use super::{
    collect::{ShellStatus, collect_shells, is_closed_nonempty, shell_infinity},
    from_shell::build_solid,
};
use rustycad_tkbrep::{
    brep::BRepBuilder,
    topexp::{self, Explorer},
    topods::{self, CompSolid, Shape, Shell},
    toptools::{IndexedDataMapOfShapeListOfShape, MapOfShape, ShapeMapHasher},
};
use rustycad_tkernel::{ncollection::IndexedDataMap, standard::Failure};
use rustycad_tkg3d::top_abs::{ShapeEnum, State};
type ShellSolids = IndexedDataMap<Shape, Shape, ShapeMapHasher>;
pub(super) fn create_solids(shape: &Shape) -> Result<(bool, Vec<Shape>), Failure> {
    let shells = Explorer::new(shape, ShapeEnum::Shell, ShapeEnum::Shape)?
        .map(|shell| topods::shell(&shell))
        .collect::<Result<Vec<_>, _>>()?;
    let mut status = ShellStatus::default();
    let holes = collect_shells(&shells, &mut status)?;
    let mut solids = ShellSolids::default();
    let mut changed = false;
    for index in 1..=holes.extent() {
        let shell = topods::shell(holes.find_key(index)?)?;
        let (oriented, solid) =
            orient_root(&shell, holes.find_from_index(index)?, &status, &mut changed)?;
        solids.add(oriented, solid);
    }
    combine_shared_faces(shape, &mut solids, &mut changed)?;
    let result = unique_solids(&solids)?;
    Ok((changed || result.len() > 1, result))
}
fn orient_root(
    shell: &Shell,
    holes: &[Shape],
    status: &ShellStatus,
    changed: &mut bool,
) -> Result<(Shape, Shape), Failure> {
    if !is_closed_nonempty(shell)? {
        *changed = true;
        return Ok((shell.clone().into_shape(), shell.clone().into_shape()));
    }
    let mut solid = build_solid(shell)?;
    let Ok(infinity) = shell_infinity(shell, status, false) else {
        return Ok((shell.clone().into_shape(), solid.into_shape()));
    };
    let mut oriented = shell.clone();
    if infinity == State::In {
        *changed = true;
        oriented.reverse();
        solid = build_solid(&oriented)?;
    }
    for hole in holes {
        add_hole(&mut solid, hole, status, changed)?;
    }
    Ok((oriented.into_shape(), solid.into_shape()))
}
fn add_hole(
    solid: &mut topods::Solid,
    hole: &Shape,
    status: &ShellStatus,
    changed: &mut bool,
) -> Result<(), Failure> {
    let mut shell = topods::shell(hole)?;
    if shell_infinity(&shell, status, true)? == State::Out {
        shell.reverse();
        *changed = true;
    }
    BRepBuilder::default().add(solid, &shell)
}
fn combine_shared_faces(
    shape: &Shape,
    solids: &mut ShellSolids,
    changed: &mut bool,
) -> Result<(), Failure> {
    let mut ancestors = IndexedDataMapOfShapeListOfShape::default();
    topexp::map_shapes_and_ancestors(shape, ShapeEnum::Face, ShapeEnum::Shell, &mut ancestors)?;
    for index in 1..=ancestors.extent() {
        let shells = ancestors.find_from_index(index)?;
        if shells.extent() < 2 {
            continue;
        }
        *changed |= shape.shape_type()? != ShapeEnum::CompSolid;
        combine_shell_solids(shells.iter().cloned().collect(), solids)?;
    }
    Ok(())
}
fn combine_shell_solids(shells: Vec<Shape>, solids: &mut ShellSolids) -> Result<(), Failure> {
    let builder = BRepBuilder::default();
    let mut compound = CompSolid::default();
    builder.make_comp_solid(&mut compound);
    let mut count = 0;
    for shell in &shells {
        let Ok(shape) = solids.find_from_key(shell) else {
            continue;
        };
        for solid in Explorer::new(shape, ShapeEnum::Solid, ShapeEnum::Shape)? {
            builder.add(&mut compound, &solid)?;
            count += 1;
        }
    }
    if count > 1 {
        for shell in &shells {
            if let Some(shape) = solids.change_seek(shell) {
                *shape = compound.clone().into_shape();
            }
        }
    }
    Ok(())
}
fn unique_solids(solids: &ShellSolids) -> Result<Vec<Shape>, Failure> {
    let mut seen = MapOfShape::default();
    let mut result = Vec::new();
    for index in 1..=solids.extent() {
        let shape = solids.find_from_index(index)?;
        if seen.add(shape.clone()) {
            result.push(shape.clone());
        }
    }
    Ok(result)
}
