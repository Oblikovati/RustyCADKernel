//! `BOPAlgo_Builder::BuildBOP` (BOPAlgo_Builder.cxx): a Boolean operation on solids built
//! from the face splits of a General Fuse result, also for open solids.
use super::{BOPAlgoBuilder, required_context};
use crate::bop_algo::alerts::alert;
use crate::bop_algo::tools::fill_internals;
use crate::bop_algo::{
    AlertBOPNotSet, AlertBuilderFailed, AlertTooFewArguments, AlertUnknownShape,
    AlertUnsupportedType, BOPAlgoPaveFiller, BuilderSolid, Operation,
};
use crate::bop_ds::DataStructure;
use crate::bop_tools::{
    is_split_shape_to_reverse, make_connexity_compounds, make_container, orient_faces_on_shell,
    treat_compound,
};
use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::topexp::{self, Explorer};
use rustycad_tkbrep::topods::{Shape, ShapeIterator, Shell, Solid};
use rustycad_tkbrep::toptools::{MapOfShape, ShapeMapHasher};
use rustycad_tkernel::message::{AlertHandle, Gravity, Report};
use rustycad_tkernel::ncollection::{IndexedMap, List, Map};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum, State};
use std::sync::Arc;

/// The face splits of one group of solids: oriented (`aMObjFacesOri`), by identity
/// (`aMObjFaces`) and the IN faces of the group (`anINObjects`).
struct GroupFaces {
    oriented: IndexedMap<Shape>,
    faces: IndexedMap<Shape, ShapeMapHasher>,
    inside: Map<Shape, ShapeMapHasher>,
}

impl BOPAlgoBuilder {
    /// The Boolean operation of the objects (kept where `object_state`) with the tools
    /// (kept where `tools_state`), on the General Fuse result of `perform`; the alerts go
    /// to `report` or the own report (`BuildBOP`).
    /// `builder.build_bop(&objects, State::Out, &tools, State::In, None)?`.
    pub fn build_bop(
        &mut self,
        objects: &List<Shape>,
        object_state: State,
        tools: &List<Shape>,
        tools_state: State,
        report: Option<&Arc<Report>>,
    ) -> Result<(), Failure> {
        let Some(filler) = self.filler.take() else {
            // Only the filler of `perform` is kept by the builder.
            let report = report
                .cloned()
                .unwrap_or_else(|| self.options().report().clone());
            report.add_alert(Gravity::Fail, &alert(AlertBuilderFailed));
            return Ok(());
        };
        let result = match filler.data_structure() {
            Some(structure) => {
                self.build_bop_with(structure, objects, object_state, tools, tools_state, report)
            }
            None => Err(super::perform::no_structure()),
        };
        self.filler = Some(filler);
        result
    }

    /// `BuildBOP(theObjects, theTools, theOperation, ...)`: the states of the operation.
    /// `builder.build_bop_operation(&objects, &tools, Operation::Cut, None)?`.
    pub fn build_bop_operation(
        &mut self,
        objects: &List<Shape>,
        tools: &List<Shape>,
        operation: Operation,
        report: Option<&Arc<Report>>,
    ) -> Result<(), Failure> {
        let (object_state, tools_state) = operation_states(operation);
        self.build_bop(objects, object_state, tools, tools_state, report)
    }

    /// `BuildBOP(theObjects, theTools, theOperation, ...)` after `perform_with_filler(filler)`:
    /// the native builder keeps a pointer to that filler's data structure, the Rust one is
    /// given the filler again; `builder.build_bop_operation_with_filler(&filler, &objects,
    /// &tools, Operation::Cut, None)?`.
    pub fn build_bop_operation_with_filler(
        &mut self,
        filler: &BOPAlgoPaveFiller,
        objects: &List<Shape>,
        tools: &List<Shape>,
        operation: Operation,
        report: Option<&Arc<Report>>,
    ) -> Result<(), Failure> {
        let structure = filler
            .data_structure()
            .ok_or_else(super::perform::no_structure)?;
        let (object_state, tools_state) = operation_states(operation);
        self.build_bop_with(structure, objects, object_state, tools, tools_state, report)
    }

    /// `BuildBOP` on the data structure of the General Fuse run.
    pub(crate) fn build_bop_with(
        &mut self,
        structure: &DataStructure,
        objects: &List<Shape>,
        object_state: State,
        tools: &List<Shape>,
        tools_state: State,
        report: Option<&Arc<Report>>,
    ) -> Result<(), Failure> {
        if self.options().has_errors() {
            return Ok(());
        }
        let report = report
            .cloned()
            .unwrap_or_else(|| self.options().report().clone());
        if let Some(failure) =
            self.check_bop_input(structure, objects, object_state, tools, tools_state)?
        {
            report.add_alert(Gravity::Fail, &failure);
            return Ok(());
        }
        let groups = [self.group_faces(objects)?, self.group_faces(tools)?];
        let faces = select_result_faces(&groups, object_state, tools_state);
        let mut solid_builder = BuilderSolid::new();
        solid_builder.set_shapes(&faces);
        let parallel = self.options().run_parallel();
        solid_builder.options_mut().set_run_parallel(parallel);
        solid_builder.set_context(self.context.clone());
        let fuzzy = self.options().fuzzy_value();
        solid_builder.options_mut().set_fuzzy_value(fuzzy);
        solid_builder.perform()?;
        if solid_builder.options().has_errors() {
            return Ok(());
        }
        let kept = kept_solids(&solid_builder, &groups);
        let mut solids = kept.clone();
        unused_face_solids(&faces, &kept, &mut solids)?;
        if !(object_state == State::Out && tools_state == State::Out) {
            let parts = internal_parts(objects, tools)?;
            let context = required_context(&self.context)?;
            fill_internals(&solids, &parts, &self.images, context)?;
        }
        let mut result = Shape::default();
        make_container(ShapeEnum::Compound, &mut result);
        for solid in solids.iter() {
            BRepBuilder::default().add(&mut result, solid)?;
        }
        self.shape = result;
        self.prepare_history(structure)
    }

    /// The failure of the input checks, if any.
    fn check_bop_input(
        &self,
        structure: &DataStructure,
        objects: &List<Shape>,
        object_state: State,
        tools: &List<Shape>,
        tools_state: State,
    ) -> Result<Option<AlertHandle>, Failure> {
        if self.arguments.is_empty() || self.shape.is_null() {
            return Ok(Some(alert(AlertBuilderFailed)));
        }
        let valid = |state: State| state == State::In || state == State::Out;
        if !valid(object_state) || !valid(tools_state) {
            return Ok(Some(alert(AlertBOPNotSet)));
        }
        if objects.is_empty() && tools.is_empty() {
            return Ok(Some(alert(AlertTooFewArguments)));
        }
        for shape in objects.iter().chain(tools.iter()) {
            if structure.index(shape) < 0 {
                return Ok(Some(Arc::new(AlertUnknownShape::new(shape))));
            }
            if shape.shape_type()? != ShapeEnum::Solid && !is_solid_collection(shape)? {
                return Ok(Some(Arc::new(AlertUnsupportedType::new(shape))));
            }
        }
        Ok(None)
    }

    /// The face splits of the solids of a group, oriented as their original face goes in
    /// the solid, and the IN faces of those solids.
    fn group_faces(&self, shapes: &List<Shape>) -> Result<GroupFaces, Failure> {
        let mut group = GroupFaces {
            oriented: IndexedMap::new(1),
            faces: IndexedMap::with_hasher(ShapeMapHasher, 1),
            inside: Map::with_hasher(ShapeMapHasher, 1),
        };
        let context = required_context(&self.context)?;
        for shape in shapes.iter() {
            for solid in Explorer::new(shape, ShapeEnum::Solid, ShapeEnum::Shape)? {
                for face in Explorer::new(&solid, ShapeEnum::Face, ShapeEnum::Shape)? {
                    if !matches!(
                        face.orientation(),
                        Orientation::Forward | Orientation::Reversed
                    ) {
                        continue;
                    }
                    let splits: Vec<Shape> = match self.images.seek(&face) {
                        Some(splits) => splits.iter().cloned().collect(),
                        None => vec![face.clone()],
                    };
                    let modified = self.images.is_bound(&face);
                    for split in splits {
                        let mut error = 0;
                        let split = if modified
                            && is_split_shape_to_reverse(&split, &face, context, &mut error)?
                        {
                            split.reversed()
                        } else {
                            split
                        };
                        group.oriented.add(split.clone());
                        group.faces.add(split);
                    }
                }
                if let Some(inside) = self.in_parts.seek(&solid) {
                    for face in inside.iter() {
                        group.inside.add(face.clone());
                    }
                }
            }
        }
        Ok(group)
    }
}

/// The states of the groups for an operation (`BuildBOP(..., theOperation, ...)`).
pub(crate) fn operation_states(operation: Operation) -> (State, State) {
    match operation {
        Operation::Common => (State::In, State::In),
        Operation::Fuse => (State::Out, State::Out),
        Operation::Cut => (State::Out, State::In),
        Operation::Cut21 => (State::In, State::Out),
        _ => (State::Unknown, State::Unknown),
    }
}

/// Whether every non-compound part of the shape is a solid or a compsolid.
fn is_solid_collection(shape: &Shape) -> Result<bool, Failure> {
    let mut fence = MapOfShape::default();
    let mut leaves = Vec::new();
    treat_compound(shape, &mut leaves, Some(&mut fence))?;
    for leaf in &leaves {
        if !matches!(leaf.shape_type()?, ShapeEnum::Solid | ShapeEnum::CompSolid) {
            return Ok(false);
        }
    }
    Ok(true)
}

/// The faces of the result solids for the given states, the same-domain faces taken or
/// avoided by their orientation.
fn select_result_faces(
    groups: &[GroupFaces; 2],
    object_state: State,
    tools_state: State,
) -> List<Shape> {
    let (objects_in, tools_in) = (object_state == State::In, tools_state == State::In);
    let avoid_in = !objects_in && !tools_in;
    let avoid_in_for_both = objects_in != tools_in;
    let same_orientation_needed = object_state == tools_state;
    let mut result: IndexedMap<Shape> = IndexedMap::new(1);
    let mut result_fence: Map<Shape, ShapeMapHasher> = Map::with_hasher(ShapeMapHasher, 1);
    let mut fence: Map<Shape, ShapeMapHasher> = Map::with_hasher(ShapeMapHasher, 1);
    let mut to_avoid: Map<Shape, ShapeMapHasher> = Map::with_hasher(ShapeMapHasher, 1);
    let mut oriented_fence: Map<Shape> = Map::new(1);
    for side in 0..2 {
        let (group, opposite) = (&groups[side], &groups[1 - side]);
        let take_in = if side == 0 { objects_in } else { tools_in };
        for face in group.oriented.iter() {
            let is_in = group.inside.contains(face);
            let is_in_opposite = opposite.inside.contains(face);
            if (avoid_in && (is_in || is_in_opposite))
                || (avoid_in_for_both && is_in && is_in_opposite)
            {
                continue;
            }
            if !fence.add(face.clone()) {
                if !opposite.faces.contains(face) {
                    // The face belongs to one group only.
                    if take_in != same_orientation_needed {
                        to_avoid.add(face.clone());
                    }
                } else {
                    // In both groups: its orientation decides.
                    let same_orientation = !oriented_fence.add(face.clone());
                    if same_orientation_needed == same_orientation {
                        if result_fence.add(face.clone()) {
                            result.add(face.clone());
                        }
                    } else {
                        to_avoid.add(face.clone());
                    }
                    continue;
                }
            }
            if !oriented_fence.add(face.clone()) {
                continue;
            }
            if take_in == is_in_opposite {
                if is_in {
                    result.add(face.clone());
                    result.add(face.reversed());
                } else if take_in && !same_orientation_needed {
                    result.add(face.reversed());
                } else {
                    result.add(face.clone());
                }
                result_fence.add(face.clone());
            }
        }
    }
    result
        .iter()
        .filter(|face| !to_avoid.contains(face))
        .cloned()
        .collect()
}

/// The built solids with at least one face of the objects or the tools.
fn kept_solids(builder: &BuilderSolid, groups: &[GroupFaces; 2]) -> List<Shape> {
    let mut solids = List::default();
    for solid in builder.areas().iter() {
        let kept =
            Explorer::new(solid, ShapeEnum::Face, ShapeEnum::Shape).is_ok_and(|mut faces| {
                faces.any(|face| {
                    groups[0].oriented.contains(&face) || groups[1].oriented.contains(&face)
                })
            });
        if kept {
            solids.append(solid.clone());
        }
    }
    solids
}

/// A solid of each connected block of the faces used by no kept solid.
fn unused_face_solids(
    faces: &List<Shape>,
    kept: &List<Shape>,
    solids: &mut List<Shape>,
) -> Result<(), Failure> {
    let mut used = MapOfShape::default();
    for solid in kept.iter() {
        topexp::map_shapes_into_map(solid, &mut used, true, true);
    }
    let builder = BRepBuilder::default();
    let mut unused = Shape::default();
    make_container(ShapeEnum::Compound, &mut unused);
    for face in faces.iter() {
        if used.add(face.clone()) {
            builder.add(&mut unused, face)?;
        }
    }
    let mut blocks = Vec::new();
    make_connexity_compounds(&unused, ShapeEnum::Edge, ShapeEnum::Face, &mut blocks)?;
    for block in blocks {
        let mut shell = Shell::default();
        builder.make_shell(&mut shell);
        let mut shell = shell.into_shape();
        for face in Explorer::new(&block, ShapeEnum::Face, ShapeEnum::Shape)? {
            builder.add(&mut shell, &face)?;
        }
        orient_faces_on_shell(&mut shell)?;
        let mut solid = Solid::default();
        builder.make_solid(&mut solid);
        let mut solid = solid.into_shape();
        builder.add(&mut solid, &shell)?;
        solids.append(solid);
    }
    Ok(())
}

/// The INTERNAL parts of the solids of both groups: vertices and edges, and the shells
/// whose first face is INTERNAL.
fn internal_parts(objects: &List<Shape>, tools: &List<Shape>) -> Result<List<Shape>, Failure> {
    let mut parts = List::default();
    for shape in objects.iter().chain(tools.iter()) {
        for solid in Explorer::new(shape, ShapeEnum::Solid, ShapeEnum::Shape)? {
            for part in ShapeIterator::new(&solid, true, true) {
                if part.orientation() == Orientation::Internal {
                    parts.append(part);
                    continue;
                }
                let first = Iterator::next(&mut ShapeIterator::new(&part, true, true));
                if first.is_some_and(|face| face.orientation() == Orientation::Internal) {
                    parts.append(part);
                }
            }
        }
    }
    Ok(parts)
}
