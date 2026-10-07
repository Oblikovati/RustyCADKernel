//! `BOPAlgo_Builder::FillInternalShapes` and `OwnInternalShapes` (BOPAlgo_Builder_3.cxx):
//! the free vertices, edges and wires of the arguments and the internal parts of the
//! solids put into the solid splits that contain them.
use super::{BOPAlgoBuilder, required_context};
use crate::bop_ds::DataStructure;
use crate::bop_tools::{compute_state_by_one_point, treat_compound};
use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::topexp;
use rustycad_tkbrep::topods::{Shape, ShapeIterator, Solid};
use rustycad_tkbrep::toptools::{IndexedDataMapOfShapeListOfShape, MapOfShape, ShapeMapHasher};
use rustycad_tkernel::ncollection::{IndexedMap, List, Map};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum, State};

/// The split solids and their sub-shape connections (`aLSd`, `aMSx`, `aMSOr`).
struct SolidParts {
    solids: Vec<Shape>,
    connections: IndexedDataMapOfShapeListOfShape,
    originals: Map<Shape, ShapeMapHasher>,
}

impl BOPAlgoBuilder {
    /// Puts the free vertices, edges and wires of the arguments and the internal parts of
    /// the solids into the solids containing them (`FillInternalShapes`).
    /// `builder.fill_internal_shapes(&ds)?`.
    pub fn fill_internal_shapes(&mut self, structure: &DataStructure) -> Result<(), Failure> {
        let mut internal = self.free_argument_parts(structure)?;
        let parts = self.solid_parts(structure, &mut internal)?;
        let mut loose: Vec<Shape> = Vec::new();
        for shape in internal.iter() {
            let connected = parts
                .connections
                .seek(shape)
                .is_some_and(|ancestors| !ancestors.is_empty());
            if !connected {
                loose.push(shape.clone());
            }
        }
        if loose.is_empty() {
            return Ok(());
        }
        self.settle_internal_parts(parts, loose)
    }

    /// The splits of the vertices, edges and wire edges of the arguments (`aMSI` part 1).
    fn free_argument_parts(
        &self,
        structure: &DataStructure,
    ) -> Result<IndexedMap<Shape, ShapeMapHasher>, Failure> {
        let mut fence = MapOfShape::default();
        let mut leaves = Vec::new();
        for argument in structure.arguments() {
            treat_compound(argument, &mut leaves, Some(&mut fence))?;
        }
        let mut parts = Vec::new();
        for leaf in &leaves {
            match leaf.shape_type()? {
                ShapeEnum::Wire => {
                    for edge in ShapeIterator::new(leaf, true, true) {
                        if fence.add(edge.clone()) {
                            parts.push(edge);
                        }
                    }
                }
                ShapeEnum::Vertex | ShapeEnum::Edge => parts.push(leaf.clone()),
                _ => {}
            }
        }
        let mut fence = MapOfShape::default();
        let mut internal: IndexedMap<Shape, ShapeMapHasher> =
            IndexedMap::with_hasher(ShapeMapHasher, 100);
        for part in parts {
            if fence.add(part.clone()) {
                self.add_images(&part, &mut internal);
            }
        }
        Ok(internal)
    }

    /// Adds the images of `shape`, or the shape itself.
    fn add_images(&self, shape: &Shape, target: &mut IndexedMap<Shape, ShapeMapHasher>) {
        match self.images.seek(shape) {
            Some(images) => images.iter().for_each(|image| {
                target.add(image.clone());
            }),
            None => {
                target.add(shape.clone());
            }
        }
    }

    /// Adds the own internal parts of the solids to `internal` and maps the sub-shapes of
    /// the solid splits (or untouched solids) to their ancestors.
    fn solid_parts(
        &self,
        structure: &DataStructure,
        internal: &mut IndexedMap<Shape, ShapeMapHasher>,
    ) -> Result<SolidParts, Failure> {
        let mut parts = SolidParts {
            solids: Vec::new(),
            connections: IndexedDataMapOfShapeListOfShape::default(),
            originals: Map::with_hasher(ShapeMapHasher, 100),
        };
        let mut fence: Map<Shape, ShapeMapHasher> = Map::with_hasher(ShapeMapHasher, 100);
        for index in 0..structure.nb_source_shapes() {
            let info = structure.shape_info(index)?;
            if info.shape_type() != ShapeEnum::Solid {
                continue;
            }
            let solid = info.shape();
            for own in own_internal_shapes(solid).iter() {
                self.add_images(own, internal);
            }
            let splits: Vec<Shape> = match self.images.seek(solid) {
                Some(splits) => splits.iter().cloned().collect(),
                None => vec![solid.clone()],
            };
            let original = !self.images.is_bound(solid);
            for split in splits {
                if !fence.add(split.clone()) {
                    continue;
                }
                map_connections(&split, &mut parts.connections)?;
                if original {
                    parts.originals.add(split.clone());
                }
                parts.solids.push(split);
            }
        }
        Ok(parts)
    }

    /// Adds each loose part into the first solid containing it; an original solid is
    /// replaced by a copy holding the part, bound as its image.
    fn settle_internal_parts(
        &mut self,
        mut parts: SolidParts,
        mut loose: Vec<Shape>,
    ) -> Result<(), Failure> {
        let handle = self.context.clone();
        let context = required_context(&handle)?;
        let builder = BRepBuilder::default();
        for solid in parts.solids.clone() {
            let mut solid = solid;
            let mut position = 0;
            while position < loose.len() {
                let part = loose[position].oriented(Orientation::Internal);
                let state = compute_state_by_one_point(
                    &part,
                    &Solid::unchecked(solid.clone()),
                    1.0e-11,
                    context,
                )?;
                if state != State::In {
                    position += 1;
                    continue;
                }
                if parts.originals.contains(&solid) {
                    solid = self.copy_original_solid(&solid, &part, &mut parts.originals)?;
                } else {
                    builder.add(&mut solid, &part)?;
                }
                loose.remove(position);
            }
        }
        Ok(())
    }

    /// A new solid of the shells of `solid` and `part`, bound as the image of `solid`.
    fn copy_original_solid(
        &mut self,
        solid: &Shape,
        part: &Shape,
        originals: &mut Map<Shape, ShapeMapHasher>,
    ) -> Result<Shape, Failure> {
        let builder = BRepBuilder::default();
        let mut copy = Solid::default();
        builder.make_solid(&mut copy);
        let mut copy = copy.into_shape();
        for shell in ShapeIterator::new(solid, true, true) {
            builder.add(&mut copy, &shell)?;
        }
        builder.add(&mut copy, part)?;
        // No images of `solid` to check: the originals are untouched solids.
        let mut images = List::default();
        images.append(copy.clone());
        self.images.bound(solid.clone(), images);
        let mut origins = List::default();
        origins.append(solid.clone());
        self.origins.bound(copy.clone(), origins);
        originals.remove(solid);
        Ok(copy)
    }
}

/// The vertex/edge, vertex/face and edge/face ancestors of the solid sub-shapes.
fn map_connections(
    solid: &Shape,
    connections: &mut IndexedDataMapOfShapeListOfShape,
) -> Result<(), Failure> {
    topexp::map_shapes_and_ancestors(solid, ShapeEnum::Vertex, ShapeEnum::Edge, connections)?;
    topexp::map_shapes_and_ancestors(solid, ShapeEnum::Vertex, ShapeEnum::Face, connections)?;
    topexp::map_shapes_and_ancestors(solid, ShapeEnum::Edge, ShapeEnum::Face, connections)
}

/// The direct sub-shapes of the solid that are not shells (`OwnInternalShapes`).
fn own_internal_shapes(solid: &Shape) -> Vec<Shape> {
    ShapeIterator::new(solid, true, true)
        .filter(|part| part.shape_type().ok() != Some(ShapeEnum::Shell))
        .collect()
}
