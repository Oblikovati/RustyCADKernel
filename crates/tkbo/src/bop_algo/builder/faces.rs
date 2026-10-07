//! `BOPAlgo_Builder::FillImagesFaces` and `BuildSplitFaces` (BOPAlgo_Builder_2.cxx).
use super::{BOPAlgoBuilder, required_context};
use crate::bop_algo::AlertUnableToMakeClosedEdgeOnFace;
use crate::bop_algo::BuilderFace;
use crate::bop_ds::DataStructure;
use crate::bop_tools::algo_tools2d::is_edge_isoline;
use crate::bop_tools::algo_tools3d::{split_seam_from_origin, split_seam_on_face};
use crate::bop_tools::is_split_to_reverse_with_warning;
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{Compound, Edge, Face, Shape, ShapeIterator};
use rustycad_tkbrep::toptools::ShapeMapHasher;
use rustycad_tkernel::message::AlertHandle;
use rustycad_tkernel::ncollection::{IndexedDataMap, List, Map};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkgeombase::geom_lib;
use rustycad_tktopalgo::brep_lib::build_pcurve_for_edges_on_plane;
use std::sync::Arc;

/// The splits of each source face by DS index (`aFacesIm`).
type FaceSplits = IndexedDataMap<i32, List<Shape>>;

impl BOPAlgoBuilder {
    /// Splits the faces, fills the same-domain faces and puts the alone vertices into the
    /// splits (`FillImagesFaces`); `builder.fill_images_faces(&mut ds)?`.
    pub fn fill_images_faces(&mut self, structure: &mut DataStructure) -> Result<(), Failure> {
        self.build_split_faces(structure)?;
        if self.options().has_errors() {
            return Ok(());
        }
        self.fill_same_domain_faces(structure)?;
        if self.options().has_errors() {
            return Ok(());
        }
        self.fill_internal_vertices(structure)
    }

    /// Builds the splits of every face touched by the intersection: draft faces when only
    /// the boundary changed, else the face builder on its edges (`BuildSplitFaces`).
    pub fn build_split_faces(&mut self, structure: &DataStructure) -> Result<(), Failure> {
        let mut splits: FaceSplits = IndexedDataMap::new(1);
        let mut builders: Vec<BuilderFace> = Vec::new();
        for index in 0..structure.nb_source_shapes() {
            let info = structure.shape_info(index)?;
            if info.shape_type() != ShapeEnum::Face || !structure.has_face_info(index)? {
                continue;
            }
            let face = Face::unchecked(info.shape().clone());
            match self.face_split_input(structure, index, &face)? {
                FaceSplitInput::Untouched => {}
                FaceSplitInput::Draft(draft) => {
                    let mut list = List::default();
                    list.append(draft.into_shape());
                    splits.add(index, list);
                }
                FaceSplitInput::Edges(edges) => {
                    let mut builder = BuilderFace::new();
                    builder.set_face(&face);
                    builder.set_shapes(&edges);
                    let parallel = self.options().run_parallel();
                    builder.options_mut().set_run_parallel(parallel);
                    builders.push(builder);
                }
            }
        }
        // Native: the face builders run without the shared context and create their own.
        for builder in builders.iter_mut() {
            builder.perform()?;
        }
        for builder in &builders {
            splits.add(structure.index(builder.face()), builder.areas().clone());
            self.options().report().merge(builder.options().report());
        }
        self.bind_face_splits(structure, &splits)
    }

    /// Binds the splits as images of the faces, a REVERSED face giving REVERSED splits.
    fn bind_face_splits(
        &mut self,
        structure: &DataStructure,
        splits: &FaceSplits,
    ) -> Result<(), Failure> {
        for position in 1..=splits.extent() {
            let face = structure.shape(*splits.find_key(position)?)?.clone();
            let reversed = face.orientation() == Orientation::Reversed;
            let mut images = List::default();
            for split in splits.find_from_index(position)?.iter() {
                images.append(if reversed {
                    split.oriented(Orientation::Reversed)
                } else {
                    split.clone()
                });
            }
            self.images.bound(face, images);
        }
        Ok(())
    }

    /// What splitting the face needs: nothing, a draft face, or the face builder on edges.
    fn face_split_input(
        &mut self,
        structure: &DataStructure,
        index: i32,
        face: &Face,
    ) -> Result<FaceSplitInput, Failure> {
        let info = structure.face_info(index)?;
        let (inner, on, section) = (
            info.pave_blocks_in().extent(),
            info.pave_blocks_on().extent(),
            info.pave_blocks_section().extent(),
        );
        let alone = structure.alone_vertices(index)?.len();
        if inner == 0 && on == 0 && section == 0 && alone == 0 {
            return Ok(FaceSplitInput::Untouched);
        }
        if inner == 0 && section == 0 {
            let has_internals = if alone == 0 {
                let (has_internals, has_modified) = self.wire_changes(face);
                if !has_internals && !has_modified {
                    return Ok(FaceSplitInput::Untouched);
                }
                has_internals
            } else {
                false
            };
            if !has_internals {
                // A null draft means internal edges or multi-connected vertices may split
                // the face (as in "bugs modalg_5 bug25245_1"): the face builder decides.
                if let Some(draft) = self.build_draft_face(face)? {
                    return Ok(FaceSplitInput::Draft(draft));
                }
            }
        }
        Ok(FaceSplitInput::Edges(
            self.face_edges(structure, index, face)?,
        ))
    }

    /// Whether the face has a wire starting with an INTERNAL edge (checked wire by wire
    /// until one is found), and whether a wire before it has images.
    fn wire_changes(&self, face: &Face) -> (bool, bool) {
        let mut has_modified = false;
        for wire in ShapeIterator::new(face, true, true) {
            let first = Iterator::next(&mut ShapeIterator::new(&wire, true, true));
            if first.is_some_and(|edge| edge.orientation() == Orientation::Internal) {
                return (true, has_modified);
            }
            has_modified |= self.images.is_bound(&wire);
        }
        (false, has_modified)
    }

    /// The edges to split the face with: the splits of its boundary edges, then its In and
    /// section edges in both orientations.
    fn face_edges(
        &mut self,
        structure: &DataStructure,
        index: i32,
        face: &Face,
    ) -> Result<List<Shape>, Failure> {
        let forward = Face::unchecked(face.oriented(Orientation::Forward));
        let mut edges = List::default();
        let mut closedness = None;
        let mut fence: Map<Shape, ShapeMapHasher> = Map::with_hasher(ShapeMapHasher, 1);
        for edge in Explorer::new(&forward, ShapeEnum::Edge, ShapeEnum::Shape)? {
            let edge = Edge::unchecked(edge);
            self.add_boundary_edge(&edge, face, &mut closedness, &mut fence, &mut edges)?;
        }
        let info = structure.face_info(index)?;
        for block in info.pave_blocks_in().iter() {
            let split = block.lock().expect("pave-block lock").edge();
            // Native: "Face information is not up to date" assertion, then skipped.
            if split < 0 {
                continue;
            }
            append_both(&mut edges, structure.shape(split)?);
        }
        for block in info.pave_blocks_section().iter() {
            let split = block.lock().expect("pave-block lock").edge();
            append_both(&mut edges, structure.shape(split)?);
        }
        if !self.non_destructive {
            // Speed up for planar faces.
            build_pcurve_for_edges_on_plane(edges.iter(), &forward)?;
        }
        Ok(edges)
    }

    /// Appends a boundary edge, or its splits oriented as it; the splits of a seam edge go in
    /// both orientations with a second p-curve made when they lost it.
    fn add_boundary_edge(
        &mut self,
        edge: &Edge,
        face: &Face,
        closedness: &mut Option<(bool, bool)>,
        fence: &mut Map<Shape, ShapeMapHasher>,
        edges: &mut List<Shape>,
    ) -> Result<(), Failure> {
        let orientation = edge.orientation();
        let Some(images) = self.images.seek(edge).cloned() else {
            if orientation == Orientation::Internal {
                append_both(edges, edge);
            } else {
                edges.append(edge.clone().into_shape());
            }
            return Ok(());
        };
        if closedness.is_none() {
            let surface = BRepTool::surface(face)?;
            *closedness = Some(geom_lib::is_closed(
                &surface,
                BRepTool::edge_tolerance(edge)?,
            )?);
        }
        let is_seam = self.is_seam_on_closed_surface(edge, face, closedness.unwrap_or_default())?;
        let degenerated = BRepTool::degenerated(edge)?;
        for split in images.iter() {
            let split = Edge::unchecked(split.clone());
            if degenerated {
                edges.append(split.oriented(orientation));
            } else if orientation == Orientation::Internal {
                append_both(edges, &split);
            } else if is_seam {
                self.add_seam_split(edge, &split, face, fence, edges)?;
            } else {
                let mut split = split.oriented(orientation);
                let context = required_context(&self.context)?;
                if is_split_to_reverse_with_warning(
                    &split,
                    edge,
                    context,
                    Some(self.options().report()),
                )? {
                    split.reverse();
                }
                edges.append(split);
            }
        }
        Ok(())
    }

    /// Whether the edge is a seam along a closed direction of the surface.
    fn is_seam_on_closed_surface(
        &self,
        edge: &Edge,
        face: &Face,
        (u_closed, v_closed): (bool, bool),
    ) -> Result<bool, Failure> {
        if !(u_closed || v_closed) || !BRepTool::is_closed_on_face(edge, face)? {
            return Ok(false);
        }
        let (mut u_iso, mut v_iso) = (false, false);
        is_edge_isoline(edge, face, &mut u_iso, &mut v_iso)?;
        Ok((u_closed && u_iso) || (v_closed && v_iso))
    }

    /// Appends a split of a seam once, in both orientations, giving it its second p-curve
    /// when it lost it.
    fn add_seam_split(
        &mut self,
        edge: &Edge,
        split: &Edge,
        face: &Face,
        fence: &mut Map<Shape, ShapeMapHasher>,
        edges: &mut List<Shape>,
    ) -> Result<(), Failure> {
        if !fence.add(split.clone().into_shape()) {
            return Ok(());
        }
        if !BRepTool::is_closed_on_face(split, face)?
            && !split_seam_on_face(split, face)?
            && !split_seam_from_origin(edge, split, face)?
        {
            self.warn_unclosed_seam(face, split)?;
        }
        append_both(edges, split);
        Ok(())
    }

    fn warn_unclosed_seam(&mut self, face: &Face, split: &Edge) -> Result<(), Failure> {
        let builder = BRepBuilder::default();
        let mut warning = Compound::default();
        builder.make_compound(&mut warning);
        let mut warning = warning.into_shape();
        builder.add(&mut warning, face)?;
        builder.add(&mut warning, split)?;
        let alert: AlertHandle = Arc::new(AlertUnableToMakeClosedEdgeOnFace::new(&warning));
        self.options().add_warning(&alert);
        Ok(())
    }
}

/// What splitting one face needs.
enum FaceSplitInput {
    /// Nothing changed on the face.
    Untouched,
    /// Only the boundary changed: the face rebuilt on the splits of its edges.
    Draft(Face),
    /// The edges for the face builder.
    Edges(List<Shape>),
}

/// Appends the edge FORWARD then REVERSED.
fn append_both(edges: &mut List<Shape>, edge: &Shape) {
    edges.append(edge.oriented(Orientation::Forward));
    edges.append(edge.oriented(Orientation::Reversed));
}
