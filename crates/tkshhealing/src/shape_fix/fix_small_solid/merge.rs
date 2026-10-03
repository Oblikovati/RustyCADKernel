mod adjacency;
mod rebuild;

use super::ShapeFixFixSmallSolid;
use super::config::SmallSolidConfig;
use super::remove::valid_input;
use crate::shape_build::ShapeBuildReShape;
use crate::shape_fix::FixRoot;
use rustycad_tkbrep::brep_tools::ReShape;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::Shape;
use rustycad_tkbrep::toptools::ShapeMapHasher;
use rustycad_tkernel::message::Msg;
use rustycad_tkernel::ncollection::DataMap;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkernel::tcollection::AsciiString;
use rustycad_tkg3d::top_abs::ShapeEnum;

type ShellGroups = DataMap<Shape, Vec<Shape>, ShapeMapHasher>;

pub(super) fn merge(
    fixer: &ShapeFixFixSmallSolid,
    shape: &Shape,
    context: &Handle<ShapeBuildReShape>,
) -> Result<Shape, Failure> {
    let config = fixer.config();
    if !config.has_threshold() || !valid_input(shape)? {
        return Ok(shape.clone());
    }
    let (mut small_solids, mut faces_to_shells) = initial_sets(shape, &config)?;
    let Some(reshape) = context.get() else {
        return Err(Failure::new(
            "ShapeFix_FixSmallSolid::Merge requires a non-null ShapeBuild_ReShape context",
        ));
    };
    merge_rounds(fixer, &mut small_solids, &mut faces_to_shells, reshape)?;
    reshape.apply(shape, ShapeEnum::Shape)
}

fn initial_sets(
    shape: &Shape,
    config: &SmallSolidConfig,
) -> Result<(Vec<Shape>, FaceShellMap), Failure> {
    let mut small_solids = Vec::new();
    let mut faces_to_shells = FaceShellMap::default();
    let mut solids = Explorer::new(shape, ShapeEnum::Solid, ShapeEnum::Shape)?;
    while solids.more() {
        let solid = solids.current().clone();
        if config.is_small(&solid)? {
            small_solids.push(solid);
        } else {
            adjacency::map_faces_to_shells(&solid, &mut faces_to_shells)?;
        }
        solids.next()?;
    }
    Ok((small_solids, faces_to_shells))
}

fn merge_rounds(
    fixer: &ShapeFixFixSmallSolid,
    small_solids: &mut Vec<Shape>,
    faces_to_shells: &mut FaceShellMap,
    context: &ShapeBuildReShape,
) -> Result<(), Failure> {
    let mut next_face_map = FaceShellMap::default();
    let mut state = MergeRoundState {
        fixer,
        small_solids,
        faces_to_shells,
        next_face_map: &mut next_face_map,
        context,
    };
    while !state.small_solids.is_empty() && merge_round(&mut state)? {}
    Ok(())
}

struct MergeRoundState<'a> {
    fixer: &'a ShapeFixFixSmallSolid,
    small_solids: &'a mut Vec<Shape>,
    faces_to_shells: &'a mut FaceShellMap,
    next_face_map: &'a mut FaceShellMap,
    context: &'a ShapeBuildReShape,
}

fn merge_round(state: &mut MergeRoundState<'_>) -> Result<bool, Failure> {
    let (mut shells_to_merge, mut shells_to_add) = grouped_shells(
        state.fixer,
        state.small_solids,
        state.faces_to_shells,
        state.context,
    )?;
    if shells_to_merge.is_empty() {
        return Ok(false);
    }
    rebuild_round(
        &mut shells_to_merge,
        &mut shells_to_add,
        state.faces_to_shells,
        state.next_face_map,
        state.context,
    )?;
    advance_face_map(state.faces_to_shells, state.next_face_map);
    Ok(true)
}

fn advance_face_map(current: &mut FaceShellMap, next: &mut FaceShellMap) {
    current.clear(true);
    std::mem::swap(current, next);
}

fn grouped_shells(
    fixer: &ShapeFixFixSmallSolid,
    small_solids: &mut Vec<Shape>,
    faces_to_shells: &FaceShellMap,
    context: &ShapeBuildReShape,
) -> Result<(ShellGroups, ShellGroups), Failure> {
    let mut groups = ShellGroupings::default();
    let mut index = 0;
    while index < small_solids.len() {
        let solid = small_solids[index].clone();
        if groups.group_current_solid(fixer, &solid, faces_to_shells, context)? {
            small_solids.remove(index);
            continue;
        }
        index += 1;
    }
    Ok((groups.shells_to_merge, groups.shells_to_add))
}

#[derive(Default)]
struct ShellGroupings {
    shells_to_merge: ShellGroups,
    shells_to_add: ShellGroups,
}

impl ShellGroupings {
    fn group_current_solid(
        &mut self,
        fixer: &ShapeFixFixSmallSolid,
        solid: &Shape,
        faces_to_shells: &FaceShellMap,
        context: &ShapeBuildReShape,
    ) -> Result<bool, Failure> {
        let Some(adjacent) = adjacency::most_shared_shell(solid, faces_to_shells)? else {
            return Ok(false);
        };
        self.append_adjacent_shells(&adjacent);
        context.remove(solid)?;
        let key = AsciiString::from("ShapeFix.FixSmallSolid.MSG1");
        fixer.send_warning(solid, &Msg::from_key(&key));
        Ok(true)
    }

    fn append_adjacent_shells(&mut self, adjacent: &adjacency::AdjacentShells) {
        append_group(
            &mut self.shells_to_merge,
            &adjacent.outer,
            vec![adjacent.selected.clone()],
        );
        append_group(
            &mut self.shells_to_add,
            &adjacent.outer,
            adjacent.remaining.clone(),
        );
    }
}

fn append_group(groups: &mut ShellGroups, key: &Shape, shells: Vec<Shape>) {
    if shells.is_empty() {
        return;
    }
    groups.try_emplaced(key.clone(), Vec::new).extend(shells);
}

fn rebuild_round(
    shells_to_merge: &mut ShellGroups,
    shells_to_add: &mut ShellGroups,
    old_face_map: &FaceShellMap,
    new_face_map: &mut FaceShellMap,
    context: &ShapeBuildReShape,
) -> Result<(), Failure> {
    for (base_shell, shells) in shells_to_merge.items() {
        let mut merged = rebuild::merge_shells(rebuild::ShellMerge {
            base_shell,
            shells_to_merge: shells,
            face_shells: old_face_map,
            new_free_faces: new_face_map,
        })?;
        append_remaining_shells(shells_to_add, base_shell, &mut merged)?;
        context.replace(base_shell, &merged)?;
    }
    Ok(())
}

fn append_remaining_shells(
    groups: &mut ShellGroups,
    base_shell: &Shape,
    merged: &mut Shape,
) -> Result<(), Failure> {
    let Some(shells) = groups.change_seek(base_shell) else {
        return Ok(());
    };
    *merged = rebuild::add_shells(merged, std::mem::take(shells))?;
    Ok(())
}

pub(super) type FaceShellMap = DataMap<Shape, Shape, ShapeMapHasher>;
