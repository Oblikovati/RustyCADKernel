//! `BOPAlgo_BOP::BuildRC` and `TypeToExplore` (BOPAlgo_BOP.cxx): the splits making the
//! result of the operation.
use super::BOPAlgoBOP;
use crate::bop_algo::Operation;
use crate::bop_ds::DataStructure;
use crate::bop_tools::{ShapeSet, dimension, make_container, treat_compound};
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::topexp::{self, Explorer};
use rustycad_tkbrep::topods::{Edge, Shape};
use rustycad_tkbrep::toptools::{IndexedMapOfShape, MapOfShape, ShapeMapHasher};
use rustycad_tkernel::ncollection::{IndexedDataMap, List, Map};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::ShapeEnum;

/// The splits of the building elements of one group, and the sets of its untouched solids.
struct GroupSplits {
    splits: IndexedMapOfShape,
    solid_sets: IndexedDataMap<ShapeSet, Shape>,
}

impl BOPAlgoBOP {
    /// The splits of the result (`BuildRC`): for FUSE all splits of the result dimension,
    /// else the splits of one group contained (COMMON) or not contained (CUT, CUT21) in the
    /// other, with the degenerated edges of their vertices.
    pub(super) fn build_rc(&mut self, structure: &DataStructure) -> Result<(), Failure> {
        let mut compound = Shape::default();
        make_container(ShapeEnum::Compound, &mut compound);
        if self.operation == Operation::Fuse {
            let mut fence = MapOfShape::default();
            let builder = BRepBuilder::default();
            let shape = self.builder().shape().clone();
            for part in Explorer::new(&shape, type_to_explore(self.dims[0]), ShapeEnum::Shape)? {
                if fence.add(part.clone()) {
                    builder.add(&mut compound, &part)?;
                }
            }
            self.rc = compound;
            return Ok(());
        }
        let mut check_edges = false;
        let arguments = self.provider.builder.arguments.clone();
        let tools = self.provider.tools().clone();
        let groups = [
            self.group_splits(&arguments, &mut check_edges)?,
            self.group_splits(&tools, &mut check_edges)?,
        ];
        self.select_splits(&groups, &mut compound)?;
        if check_edges {
            add_squat_degenerated_edges(structure, &mut compound)?;
        }
        self.rc = compound;
        Ok(())
    }

    /// The splits of the building elements (the shapes of their own dimension) of a group.
    fn group_splits(
        &self,
        shapes: &List<Shape>,
        check_edges: &mut bool,
    ) -> Result<GroupSplits, Failure> {
        let mut elements = IndexedMapOfShape::default();
        for shape in shapes.iter() {
            let mut leaves = Vec::new();
            treat_compound(shape, &mut leaves, None)?;
            for leaf in leaves {
                let dim = dimension(&leaf)?;
                if dim >= 0 {
                    topexp::map_shapes_of_type(&leaf, type_to_explore(dim), &mut elements)?;
                }
            }
        }
        let mut group = GroupSplits {
            splits: IndexedMapOfShape::default(),
            solid_sets: IndexedDataMap::new(1),
        };
        for element in elements.iter() {
            if element.shape_type()? == ShapeEnum::Edge {
                *check_edges = true;
                if BRepTool::degenerated(&Edge::unchecked(element.clone()))? {
                    continue;
                }
            }
            if let Some(images) = self.builder().images().seek(element) {
                for image in images.iter() {
                    group.splits.add(image.clone());
                }
                continue;
            }
            group.splits.add(element.clone());
            if element.shape_type()? == ShapeEnum::Solid {
                let mut set = ShapeSet::new();
                set.add(element, ShapeEnum::Face)?;
                if !group.solid_sets.contains(&set) {
                    group.solid_sets.add(set, element.clone());
                }
            }
        }
        Ok(group)
    }

    /// Adds the splits of the iterated group that the checked group contains (COMMON) or
    /// does not contain (CUT, CUT21); COMMON keeps the highest dimensions first.
    fn select_splits(
        &self,
        groups: &[GroupSplits; 2],
        compound: &mut Shape,
    ) -> Result<(), Failure> {
        let common = self.operation == Operation::Common;
        let (iterated, checked) = if self.operation == Operation::Cut21 {
            (&groups[1], &groups[0])
        } else {
            (&groups[0], &groups[1])
        };
        let dim_min = self.dims[0].min(self.dims[1]);
        let iterated_expanded = if common {
            expand(&iterated.splits, dim_min)?
        } else {
            iterated.splits.clone()
        };
        let checked_expanded = expand(&checked.splits, dim_min)?;
        let builder = BRepBuilder::default();
        let mut selected = Shape::default();
        make_container(ShapeEnum::Compound, &mut selected);
        for split in iterated_expanded.iter() {
            let mut contains = checked_expanded.contains(split);
            if !contains && split.shape_type()? == ShapeEnum::Solid {
                let mut set = ShapeSet::new();
                set.add(split, ShapeEnum::Face)?;
                contains = checked.solid_sets.contains(&set);
            }
            if contains == common {
                builder.add(&mut selected, split)?;
            }
        }
        *compound = if common {
            filter_common(&selected, dim_min)?
        } else {
            selected
        };
        Ok(())
    }
}

/// The splits with their sub-shapes of the dimensions from `dim_min` to below their own.
fn expand(splits: &IndexedMapOfShape, dim_min: i32) -> Result<IndexedMapOfShape, Failure> {
    let mut expanded = IndexedMapOfShape::default();
    for split in splits.iter() {
        let dim_max = dimension(split)?;
        for dim in dim_min..dim_max {
            topexp::map_shapes_of_type(split, type_to_explore(dim), &mut expanded)?;
        }
        expanded.add(split.clone());
    }
    Ok(expanded)
}

/// The COMMON result without the shapes contained in shapes of a higher dimension.
fn filter_common(selected: &Shape, dim_min: i32) -> Result<Shape, Failure> {
    let builder = BRepBuilder::default();
    let mut filtered = Shape::default();
    make_container(ShapeEnum::Compound, &mut filtered);
    let mut fence: Map<Shape, ShapeMapHasher> = Map::with_hasher(ShapeMapHasher, 1);
    for dim in (dim_min..=3).rev() {
        for part in Explorer::new(selected, type_to_explore(dim), ShapeEnum::Shape)? {
            if fence.add(part.clone()) {
                builder.add(&mut filtered, &part)?;
                topexp::map_shapes_into_map(&part, &mut fence, true, true);
            }
        }
    }
    Ok(filtered)
}

/// Adds the original degenerated edges whose vertex is in the result and untouched by any
/// interference (the squats around degenerated edges).
fn add_squat_degenerated_edges(
    structure: &DataStructure,
    compound: &mut Shape,
) -> Result<(), Failure> {
    let mut vertices = IndexedMapOfShape::default();
    topexp::map_shapes_of_type(compound, ShapeEnum::Vertex, &mut vertices)?;
    let builder = BRepBuilder::default();
    for index in 0..structure.nb_source_shapes() {
        let info = structure.shape_info(index)?;
        if info.shape_type() != ShapeEnum::Edge {
            continue;
        }
        let edge = info.shape().clone();
        if !BRepTool::degenerated(&Edge::unchecked(edge.clone()))? {
            continue;
        }
        let Some(&vertex) = info.sub_shapes().first() else {
            continue;
        };
        if !vertices.contains(structure.shape(vertex)?)
            || structure.is_new_shape(vertex)
            || structure.has_interference_for(vertex)
        {
            continue;
        }
        builder.add(compound, &edge)?;
    }
    Ok(())
}

/// The shape type of a dimension (`TypeToExplore`): vertex, edge, face, solid, else shape.
pub(super) fn type_to_explore(dim: i32) -> ShapeEnum {
    match dim {
        0 => ShapeEnum::Vertex,
        1 => ShapeEnum::Edge,
        2 => ShapeEnum::Face,
        3 => ShapeEnum::Solid,
        _ => ShapeEnum::Shape,
    }
}
