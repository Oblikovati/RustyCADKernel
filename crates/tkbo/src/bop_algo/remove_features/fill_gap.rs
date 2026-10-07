//! `FillGap` of BOPAlgo_RemoveFeatures.cxx: the faces adjacent to one feature, extended
//! over the gap the feature leaves and trimmed by each other and by their original bounds.
use super::helpers::{
    children, compound_of, explore, make_removed, remove_internal_wires, take_modified,
};
use super::solids::find_extra_shapes;
use crate::bop_algo::{BOPAlgoBuilder, Glue};
use crate::bop_tools::make_connexity_blocks;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::brep_tools::History;
use rustycad_tkbrep::topexp;
use rustycad_tkbrep::topods::{self, Face, Shape};
use rustycad_tkbrep::toptools::{
    IndexedDataMapOfShapeListOfShape, IndexedMapOfShape, MapOfShape, ShapeMapHasher,
};
use rustycad_tkernel::ncollection::{IndexedDataMap, List};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::bnd::BoundingBox3;
use rustycad_tktopalgo::brep_bnd_lib::BRepBndLib;
use rustycad_tktopalgo::brep_lib::extend_face;

/// The extended face of each adjacent face (`aFaceExtFaceMap`).
type ExtendedFaces = IndexedDataMap<Shape, Shape, ShapeMapHasher>;

/// The connection maps of the input shape the gap filling reads.
pub(super) struct Connections<'a> {
    /// Edge to faces (`myEFMap`).
    pub(super) edge_faces: &'a IndexedDataMapOfShapeListOfShape,
    /// Face to solids (`myFSMap`).
    pub(super) face_solids: &'a IndexedDataMapOfShapeListOfShape,
}

/// The reconstruction of the faces adjacent to one feature (`FillGap`).
#[derive(Default)]
pub(super) struct FillGap {
    pub(super) feature: Shape,
    run_parallel: bool,
    pub(super) feature_faces: MapOfShape,
    pub(super) has_adjacent_faces: bool,
    pub(super) solids: IndexedMapOfShape,
    /// The trimmed images of each adjacent face (`myFaces`).
    pub(super) faces: IndexedDataMapOfShapeListOfShape,
    pub(super) history: Handle<History>,
}

impl FillGap {
    /// A gap filler of `feature` (`SetFeature`, `SetRunParallel`).
    pub(super) fn new(feature: &Shape, run_parallel: bool) -> Self {
        Self {
            feature: feature.clone(),
            run_parallel,
            ..Self::default()
        }
    }

    /// Finds, extends and trims the adjacent faces; a failure marks the feature as having
    /// adjacent faces without reconstruction, so the removal warns (`Perform`).
    pub(super) fn perform(&mut self, connections: &Connections<'_>) {
        let history = History::new();
        let outcome = self.fill(connections, &history);
        self.history = Handle::new(history);
        if outcome.is_err() {
            self.has_adjacent_faces = true;
            self.faces.clear(false);
        }
    }

    fn fill(&mut self, connections: &Connections<'_>, history: &History) -> Result<(), Failure> {
        let adjacent = self.find_adjacent_faces(connections)?;
        self.has_adjacent_faces = adjacent.extent() > 0;
        if !self.has_adjacent_faces {
            return Ok(());
        }
        let extended = self.extend_adjacent_faces(&adjacent, history)?;
        self.trim_extended_faces(&extended, history)
    }

    /// The faces sharing an edge with the feature, and the solids holding the feature or a
    /// face adjacent to an adjacent face (`FindAdjacentFaces`).
    fn find_adjacent_faces(
        &mut self,
        connections: &Connections<'_>,
    ) -> Result<IndexedMapOfShape, Failure> {
        for face in children(&self.feature) {
            self.feature_faces.add(face);
        }
        let mut adjacent = IndexedMapOfShape::default();
        for face in children(&self.feature) {
            for edge in explore(&face, ShapeEnum::Edge)? {
                for neighbour in connections
                    .edge_faces
                    .seek(&edge)
                    .into_iter()
                    .flat_map(List::iter)
                {
                    if !self.feature_faces.contains(neighbour) {
                        adjacent.add(neighbour.clone());
                    }
                }
            }
            self.add_solids_of(&face, connections);
        }
        for face in adjacent.iter() {
            for edge in explore(face, ShapeEnum::Edge)? {
                for neighbour in connections
                    .edge_faces
                    .seek(&edge)
                    .into_iter()
                    .flat_map(List::iter)
                {
                    self.add_solids_of(neighbour, connections);
                }
            }
        }
        Ok(adjacent)
    }

    fn add_solids_of(&mut self, face: &Shape, connections: &Connections<'_>) {
        for solid in connections
            .face_solids
            .seek(face)
            .into_iter()
            .flat_map(List::iter)
        {
            self.solids.add(solid.clone());
        }
    }

    /// Each adjacent face extended in all directions by the diagonal of the feature box,
    /// recorded as modified into its extension (`ExtendAdjacentFaces`).
    fn extend_adjacent_faces(
        &self,
        adjacent: &IndexedMapOfShape,
        history: &History,
    ) -> Result<ExtendedFaces, Failure> {
        let mut bounds = BoundingBox3::default();
        BRepBndLib::add(&self.feature, &mut bounds, true)?;
        let length = bounds.square_extent().sqrt();
        let mut extended: ExtendedFaces = IndexedDataMap::with_hasher(ShapeMapHasher, 1);
        for face in adjacent.iter() {
            let extension = extend_face(&topods::face(face)?, length, [true; 4])?.into_shape();
            extended.add(face.clone(), extension.clone());
            history.add_modified(face, &extension)?;
        }
        Ok(extended)
    }

    /// Intersects the extended faces with each other, then trims each by the bounds of its
    /// original face (`TrimExtendedFaces`).
    fn trim_extended_faces(
        &mut self,
        extended: &ExtendedFaces,
        history: &History,
    ) -> Result<(), Failure> {
        let mut intersector = BOPAlgoBuilder::new();
        for position in 1..=extended.extent() {
            intersector.add_argument(extended.find_from_index(position)?);
        }
        intersector
            .options_mut()
            .set_run_parallel(self.run_parallel);
        let result = if intersector.arguments().extent() > 1 {
            intersector.perform();
            if intersector.options().has_errors() {
                return Ok(());
            }
            if let Some(intersection) = intersector.history() {
                history.merge_handle(&intersection)?;
            }
            intersector.shape().clone()
        } else {
            intersector.arguments().first().cloned().unwrap_or_default()
        };
        let mut extended_edge_faces = IndexedDataMapOfShapeListOfShape::default();
        topexp::map_shapes_and_ancestors(
            &result,
            ShapeEnum::Edge,
            ShapeEnum::Face,
            &mut extended_edge_faces,
        )?;
        let mut feature_edges = IndexedMapOfShape::default();
        topexp::map_shapes_of_type(&self.feature, ShapeEnum::Edge, &mut feature_edges)?;
        let trim = TrimInputs {
            feature_edges: &feature_edges,
            extended_edge_faces: &extended_edge_faces,
            intersector: &intersector,
        };
        for position in 1..=extended.extent() {
            let original = topods::face(extended.find_key(position)?)?;
            let extension = topods::face(extended.find_from_index(position)?)?;
            self.trim_face(&extension, &original, &trim, history)?;
        }
        Ok(())
    }
}

/// What every trim of an extended face reads.
struct TrimInputs<'a> {
    feature_edges: &'a IndexedMapOfShape,
    extended_edge_faces: &'a IndexedDataMapOfShapeListOfShape,
    intersector: &'a BOPAlgoBuilder,
}

impl FillGap {
    /// The splits of the extended face trimmed by the edges of its original face outside
    /// the feature; the splits bounded by the extension or on the wrong side are dropped,
    /// and the history records the trim and the removed splits (`TrimFace`).
    fn trim_face(
        &mut self,
        extension: &Face,
        original: &Face,
        trim: &TrimInputs<'_>,
        history: &History,
    ) -> Result<(), Failure> {
        let extension_edges = extension_boundary(extension, trim)?;
        let mut trimmer = BOPAlgoBuilder::new();
        let splits: List<Shape> = take_modified(extension, trim.intersector)?
            .into_iter()
            .collect();
        trimmer.set_arguments(&splits);
        let mut edges_to_check = MapOfShape::default();
        for edge in explore(original, ShapeEnum::Edge)? {
            if trim.feature_edges.contains(&edge) {
                continue;
            }
            trimmer.add_argument(&edge);
            let typed = topods::edge(&edge)?;
            if !BRepTool::degenerated(&typed)?
                && !BRepTool::is_closed_on_face(&typed, original)?
                && !edges_to_check.add(edge.clone())
            {
                // A seam-like edge met twice is not checked.
                edges_to_check.remove(&edge);
            }
        }
        trimmer.set_glue(Glue::Shift);
        trimmer.options_mut().set_run_parallel(self.run_parallel);
        trimmer.set_non_destructive(true);
        trimmer.perform();
        if trimmer.options().has_errors() {
            return Ok(());
        }
        let trimmed = select_trimmed(&trimmer, &extension_edges, &edges_to_check)?;
        if !trimmed.is_empty() {
            remove_internal_wires(&trimmed, None)?;
            self.faces.add(
                original.clone().into_shape(),
                trimmed.iter().cloned().collect(),
            );
        }
        if let Some(trim_history) = trimmer.history() {
            history.merge_handle(&trim_history)?;
        }
        record_removed_splits(trimmer.shape(), &trimmed, history)
    }
}

/// The splits of the boundary edges of the extended face, bounding one face of the
/// intersection result.
fn extension_boundary(extension: &Face, trim: &TrimInputs<'_>) -> Result<MapOfShape, Failure> {
    let mut boundary = MapOfShape::default();
    for edge in explore(extension, ShapeEnum::Edge)? {
        let typed = topods::edge(&edge)?;
        // Degenerated and seam edges bound nothing.
        if BRepTool::degenerated(&typed)? || BRepTool::is_closed_on_face(&typed, extension)? {
            continue;
        }
        for image in take_modified(&edge, trim.intersector)? {
            if trim.extended_edge_faces.find_from_key(&image)?.extent() == 1 {
                boundary.add(image);
            }
        }
    }
    Ok(boundary)
}

/// The trimmed splits without an extension edge; several are filtered by the orientation
/// of the original edges and by connected blocks, none falls back to every split.
fn select_trimmed(
    trimmer: &BOPAlgoBuilder,
    extension_edges: &MapOfShape,
    edges_to_check: &MapOfShape,
) -> Result<Vec<Shape>, Failure> {
    let splits = explore(trimmer.shape(), ShapeEnum::Face)?;
    let mut trimmed = Vec::new();
    for split in &splits {
        if !explore(split, ShapeEnum::Edge)?
            .iter()
            .any(|edge| extension_edges.contains(edge))
        {
            trimmed.push(split.clone());
        }
    }
    if trimmed.is_empty() {
        // Use all splits, including those having the bounds of the extended face.
        return Ok(splits);
    }
    if trimmed.len() == 1 {
        return Ok(trimmed);
    }
    let to_avoid = faces_to_avoid(trimmer, &trimmed, edges_to_check)?;
    trimmed.retain(|face| !to_avoid.contains(face));
    Ok(trimmed)
}

/// The trimmed faces whose original edges run reversed, with the connected blocks of the
/// others that hold no validated face.
fn faces_to_avoid(
    trimmer: &BOPAlgoBuilder,
    trimmed: &[Shape],
    edges_to_check: &MapOfShape,
) -> Result<MapOfShape, Failure> {
    let mut edge_faces = IndexedDataMapOfShapeListOfShape::default();
    for face in trimmed {
        topexp::map_shapes_and_ancestors(face, ShapeEnum::Edge, ShapeEnum::Face, &mut edge_faces)?;
    }
    let (mut to_avoid, mut valid) = (MapOfShape::default(), MapOfShape::default());
    find_extra_shapes(
        &edge_faces,
        edges_to_check,
        trimmer,
        &mut to_avoid,
        Some(&mut valid),
    )?;
    if trimmed.len().saturating_sub(to_avoid.extent()) <= 1 {
        return Ok(to_avoid);
    }
    let kept: Vec<Shape> = trimmed
        .iter()
        .filter(|face| !to_avoid.contains(face))
        .cloned()
        .collect();
    let mut blocks = Vec::new();
    let mut ancestors = IndexedDataMapOfShapeListOfShape::default();
    make_connexity_blocks(
        &compound_of(&kept)?,
        ShapeEnum::Edge,
        ShapeEnum::Face,
        &mut blocks,
        &mut ancestors,
    )?;
    if blocks.len() > 1 {
        for block in blocks
            .iter()
            .filter(|block| !block.iter().any(|face| valid.contains(face)))
        {
            // A block without a valid face is invalid.
            for face in block {
                to_avoid.add(face.clone());
            }
        }
    }
    Ok(to_avoid)
}

/// Every sub-shape of the trimmer result outside the kept splits is removed.
fn record_removed_splits(
    splits: &Shape,
    trimmed: &[Shape],
    history: &History,
) -> Result<(), Failure> {
    let mut kept = IndexedMapOfShape::default();
    for face in trimmed {
        topexp::map_shapes(face, &mut kept, true, true);
    }
    let removals = History::new();
    make_removed(std::slice::from_ref(splits), &removals, &kept)?;
    history.merge(&removals)
}
