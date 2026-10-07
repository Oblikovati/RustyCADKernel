//! `BOPAlgo_Tools::ClassifyFaces` with its `BOPAlgo_FillIn3DParts` task: the faces lying
//! inside each solid.
use crate::bop_tools::{BoxSelector, BoxSet, internal_face_in_solid, is_inverted_solid};
use crate::int_tools::Context;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topexp::{self, Explorer};
use rustycad_tkbrep::topods::{Edge, Face, Shape, ShapeIterator, Solid, Vertex};
use rustycad_tkbrep::toptools::{IndexedDataMapOfShapeListOfShape, ShapeMapHasher};
use rustycad_tkernel::ncollection::{DataMap, IndexedDataMap, IndexedMap, List, Map};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::bnd::{BoundingBox3, box3_to_bvh};
use rustycad_tkmath::bvh::{BvhSetTraverse, BvhVec3d};
use rustycad_tktopalgo::brep_bnd_lib::BRepBndLib;

/// Bounding boxes already known for shapes (`theShapeBoxMap`).
pub type ShapeBoxMap = DataMap<Shape, BoundingBox3, ShapeMapHasher>;
/// Shapes and the faces classified inside them (`theInParts`).
pub type InParts = IndexedDataMap<Shape, List<Shape>, ShapeMapHasher>;
/// Own INTERNAL faces of solids (`theSolidsIF`).
pub type SolidInternalFaces = DataMap<Shape, List<Shape>, ShapeMapHasher>;

/// The faces to classify with their boxes, in a box tree (`BOPAlgo_VectorOfShapeBox`).
struct FaceBoxes {
    faces: Vec<Shape>,
    tree: BoxSet<BvhVec3d>,
}

/// Classifies `faces` against `solids`, adding to `in_parts` each solid with the faces
/// inside it (`ClassifyFaces`).
/// `classify_faces(&faces, &solids, &context, &mut in_parts, &boxes, &internal)?`.
pub fn classify_faces(
    faces: &List<Shape>,
    solids: &List<Shape>,
    context: &Context,
    in_parts: &mut InParts,
    boxes: &ShapeBoxMap,
    solid_internal_faces: &SolidInternalFaces,
) -> Result<(), Failure> {
    let mut face_boxes = face_boxes(faces, boxes)?;
    for solid in solids.iter() {
        let solid = Solid::unchecked(solid.clone());
        let solid_box = solid_box(&solid, boxes)?;
        let own_internal = solid_internal_faces
            .seek(&solid)
            .cloned()
            .unwrap_or_default();
        let inside = faces_in_solid(&solid, &solid_box, &own_internal, &mut face_boxes, context)?;
        if !in_parts.contains(&solid) {
            in_parts.add(solid.into_shape(), inside);
        }
    }
    Ok(())
}

fn face_boxes(faces: &List<Shape>, boxes: &ShapeBoxMap) -> Result<FaceBoxes, Failure> {
    let mut tree: BoxSet<BvhVec3d> = BoxSet::default();
    tree.set_size(faces.len());
    let mut list = Vec::new();
    for (position, face) in faces.iter().enumerate() {
        let bounds = match boxes.seek(face) {
            Some(bounds) => bounds.clone(),
            None => {
                let mut bounds = BoundingBox3::default();
                BRepBndLib::add(face, &mut bounds, true)?;
                bounds
            }
        };
        tree.add(position as i32, box3_to_bvh(&bounds)?);
        list.push(face.clone());
    }
    tree.build()?;
    Ok(FaceBoxes { faces: list, tree })
}

/// The known box of the solid, else its box, whole for an inverted solid.
fn solid_box(solid: &Solid, boxes: &ShapeBoxMap) -> Result<BoundingBox3, Failure> {
    if let Some(bounds) = boxes.seek(solid) {
        return Ok(bounds.clone());
    }
    let mut bounds = BoundingBox3::default();
    BRepBndLib::add(solid, &mut bounds, true)?;
    if !bounds.is_whole() && is_inverted_solid(solid)? {
        bounds.set_whole();
    }
    Ok(bounds)
}

/// The faces inside one solid (`BOPAlgo_FillIn3DParts::Perform`): the faces whose box meets
/// the solid box, not faces of the solid, grouped in blocks not crossing the solid edges;
/// one face of each block is classified.
fn faces_in_solid(
    solid: &Solid,
    solid_box: &BoundingBox3,
    own_internal: &List<Shape>,
    face_boxes: &mut FaceBoxes,
    context: &Context,
) -> Result<List<Shape>, Failure> {
    let mut inside = List::default();
    let mut selector = BoxSelector::default();
    selector.set_box(box3_to_bvh(solid_box)?);
    selector.set_bvh_set(Some(&mut face_boxes.tree));
    if selector.select()? == 0 {
        return Ok(inside);
    }
    let selected: Vec<i32> = selector.indices().iter().copied().collect();
    let mut solid_edges: IndexedMap<Shape, ShapeMapHasher> =
        IndexedMap::with_hasher(ShapeMapHasher, 1);
    topexp::map_shapes_of_type(solid, ShapeEnum::Edge, &mut solid_edges)?;
    let mut solid_faces: IndexedMap<Shape, ShapeMapHasher> =
        IndexedMap::with_hasher(ShapeMapHasher, 1);
    topexp::map_shapes_of_type(solid, ShapeEnum::Face, &mut solid_faces)?;
    let is_empty = solid_faces.is_empty();
    for face in own_internal.iter() {
        solid_faces.add(face.clone());
    }
    let mut candidates: Vec<i32> = selected
        .into_iter()
        .filter(|index| !solid_faces.contains(&face_boxes.faces[*index as usize]))
        .collect();
    candidates.sort_unstable();
    if is_empty {
        // A solid of INTERNAL faces only classifies every point IN: no real classification.
        for index in candidates {
            inside.append(face_boxes.faces[index as usize].clone());
        }
        return Ok(inside);
    }
    let candidate_faces: Vec<Shape> = candidates
        .iter()
        .map(|index| face_boxes.faces[*index as usize].clone())
        .collect();
    let blocks = ClassificationBlocks {
        solid,
        solid_box,
        solid_edges: &solid_edges,
        candidate_edges: candidate_edge_faces(&candidate_faces),
    };
    blocks.collect_inside(&candidate_faces, context, &mut inside)?;
    Ok(inside)
}

/// The edge/face map of the candidate faces through their wires (`MapEdgesAndFaces`);
/// empty for a single candidate.
fn candidate_edge_faces(faces: &[Shape]) -> IndexedDataMapOfShapeListOfShape {
    let mut edge_faces = IndexedDataMapOfShapeListOfShape::default();
    if faces.len() < 2 {
        return edge_faces;
    }
    for face in faces {
        for wire in ShapeIterator::new(face, true, true) {
            if wire.shape_type().ok() != Some(ShapeEnum::Wire) {
                continue;
            }
            for edge in ShapeIterator::new(&wire, true, true) {
                if let Some(found) = edge_faces.change_seek(&edge) {
                    found.append(face.clone());
                    continue;
                }
                let mut found = List::default();
                found.append(face.clone());
                edge_faces.add(edge, found);
            }
        }
    }
    edge_faces
}

/// The solid and the maps the blocks of candidate faces are built and classified with.
struct ClassificationBlocks<'a> {
    solid: &'a Solid,
    solid_box: &'a BoundingBox3,
    solid_edges: &'a IndexedMap<Shape, ShapeMapHasher>,
    candidate_edges: IndexedDataMapOfShapeListOfShape,
}

impl ClassificationBlocks<'_> {
    fn collect_inside(
        &self,
        faces: &[Shape],
        context: &Context,
        inside: &mut List<Shape>,
    ) -> Result<(), Failure> {
        let mut solid_edge_faces = IndexedDataMapOfShapeListOfShape::default();
        let mut done: Map<Shape, ShapeMapHasher> = Map::with_hasher(ShapeMapHasher, 1);
        for face in faces {
            if !done.add(face.clone()) {
                continue;
            }
            let (block, to_classify) = self.connexity_block(face, &mut done)?;
            if !self.solid_box.is_whole() && self.block_out_of_box(&block)? {
                continue;
            }
            let to_classify = to_classify.unwrap_or_else(|| Face::unchecked(face.clone()));
            if solid_edge_faces.is_empty() {
                topexp::map_shapes_and_ancestors(
                    self.solid,
                    ShapeEnum::Edge,
                    ShapeEnum::Face,
                    &mut solid_edge_faces,
                )?;
            }
            if internal_face_in_solid(
                &to_classify,
                self.solid,
                &solid_edge_faces,
                precision::confusion(),
                context,
            )? {
                for face in block {
                    inside.append(face);
                }
            }
        }
        Ok(())
    }

    /// The faces connected to `start` through edges off the solid; the first face touching
    /// a solid or degenerated edge is the one to classify (`MakeConnexityBlock`).
    fn connexity_block(
        &self,
        start: &Shape,
        done: &mut Map<Shape, ShapeMapHasher>,
    ) -> Result<(Vec<Shape>, Option<Face>), Failure> {
        let mut block = vec![start.clone()];
        let mut to_classify = None;
        if self.candidate_edges.is_empty() {
            return Ok((block, to_classify));
        }
        let mut position = 0;
        while position < block.len() {
            let face = block[position].clone();
            position += 1;
            for wire in ShapeIterator::new(&face, true, true) {
                if wire.shape_type()? != ShapeEnum::Wire {
                    continue;
                }
                for edge in ShapeIterator::new(&wire, true, true) {
                    if self.solid_edges.contains(&edge)
                        || BRepTool::degenerated(&Edge::unchecked(edge.clone()))?
                    {
                        if to_classify.is_none() {
                            to_classify = Some(Face::unchecked(face.clone()));
                        }
                        continue;
                    }
                    let Some(neighbours) = self.candidate_edges.seek(&edge) else {
                        continue;
                    };
                    for neighbour in neighbours.iter() {
                        if done.add(neighbour.clone()) {
                            block.push(neighbour.clone());
                        }
                    }
                }
            }
        }
        Ok((block, to_classify))
    }

    /// Whether a vertex box of the block is out of the solid box: the block is then out.
    fn block_out_of_box(&self, block: &[Shape]) -> Result<bool, Failure> {
        for face in block {
            for vertex in Explorer::new(face, ShapeEnum::Vertex, ShapeEnum::Shape)? {
                let vertex = Vertex::unchecked(vertex);
                let mut bounds = BoundingBox3::default();
                bounds.add_point(BRepTool::pnt(&vertex)?);
                bounds.set_gap(BRepTool::vertex_tolerance(&vertex)?);
                if self.solid_box.is_out_box(&bounds) {
                    return Ok(true);
                }
            }
        }
        Ok(false)
    }
}
