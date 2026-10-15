// Rust translation of BRepOffset_BuildOffsetFaces::BuildSplitsOfFaces, GetEdges and
// CheckIfArtificial (with the static checkConnectionsOfFace) of
// ModelingAlgorithms/TKOffset/BRepOffset/BRepOffset_MakeOffset_1.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The splits of the offset faces and the search of their invalid parts.
use super::{
    BuildOffsetFaces, ShapeIndexedMapMap, ShapeListMap, ShapeMapMap, build_splits_of_face,
    process_micro_edge,
};
use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::topexp::{self, Explorer};
use rustycad_tkbrep::topods::{self, Compound, Face, Shape};
use rustycad_tkbrep::toptools::{IndexedDataMapOfShapeListOfShape, IndexedMapOfShape, MapOfShape};
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::ShapeEnum;

/// `checkConnectionsOfFace(theFace, theLF)`: whether the face shares more than one vertex
/// with the other faces of the list.
fn check_connections_of_face(face: &Shape, lf: &List<Shape>) -> Result<bool, Failure> {
    let mut shape_vert = IndexedMapOfShape::default();
    for s in lf.iter() {
        if s.is_same(face) {
            continue;
        }
        topexp::map_shapes_of_type(s, ShapeEnum::Vertex, &mut shape_vert)?;
    }
    let mut nb_connections = 0;
    let mut face_vertices = IndexedMapOfShape::default();
    topexp::map_shapes_of_type(face, ShapeEnum::Vertex, &mut face_vertices)?;
    for i in 1..=face_vertices.extent() {
        if shape_vert.contains(face_vertices.find_key(i)?) {
            nb_connections += 1;
        }
        if nb_connections > 1 {
            return Ok(true);
        }
    }
    Ok(false)
}

impl BuildOffsetFaces<'_> {
    /// `BuildSplitsOfFaces(theRange)`: the splits of the faces with new intersection
    /// edges, the invalid edges and faces, the removal of the invalid splits that can be
    /// removed safely, and the invalid edges to avoid on the next step.
    pub(super) fn build_splits_of_faces(&mut self) -> Result<(), Failure> {
        let bb = BRepBuilder::default();
        // processed faces
        let mut a_lf_done: List<Shape> = List::default();
        // extended face - map of neutral edges, i.e. in one split - valid and in other - invalid
        let mut a_dmfmne = ShapeMapMap::default();
        // map of valid edges for each face
        let mut a_dmfmve = ShapeMapMap::default();
        // map of invalid edges for each face
        let mut a_dmfmie = ShapeIndexedMapMap::default();
        // map of valid inverted edges for the face
        let mut a_dmfmvie = ShapeMapMap::default();
        // map of splits to check for internals
        let mut a_mf_to_check_int = IndexedMapOfShape::default();
        // map of edges created from vertex and marked as invalid
        let mut a_m_edge_invalid_by_vertex = MapOfShape::default();
        // map of edges created from vertex and marked as valid
        let mut a_m_edge_valid_by_vertex = MapOfShape::default();
        // connection map from old edges to new ones
        let mut a_dme_or_le_im = ShapeListMap::default();

        // build splits of faces
        for a_f in self.faces().iter() {
            let a_f = topods::face(a_f)?;
            let has_images = self.my_of_images.contains(&a_f);
            if has_images && self.my_of_images.find_from_key(&a_f)?.is_empty() {
                continue;
            }
            // get edges by which the face should be split
            let mut a_ce = Shape::default();
            let mut a_map_e_inv = IndexedMapOfShape::default();
            let b_found = self.get_edges(&a_f, &mut a_ce, Some(&mut a_map_e_inv))?;
            if !b_found {
                continue;
            }
            // build splits
            let mut a_lf_images = List::default();
            build_splits_of_face(&a_f, &a_ce, self.faces_origins()?, &mut a_lf_images)?;

            if a_map_e_inv.extent() > 0 {
                // check if all possible faces are built
                let mut a_me_n_inv = MapOfShape::default();
                let mut b_artificial_case = a_lf_images.is_empty()
                    || self.check_if_artificial(
                        &a_f,
                        &a_lf_images,
                        &a_ce,
                        &a_map_e_inv,
                        &mut a_me_n_inv,
                    )?;

                // try to build splits using invalid edges
                let mut a_ce1 = Compound::default();
                bb.make_compound(&mut a_ce1);
                bb.add(&mut a_ce1, &a_ce)?;
                for i in 1..=a_map_e_inv.extent() {
                    bb.add(&mut a_ce1, a_map_e_inv.find_key(i)?)?;
                }

                let mut a_lf_images1 = List::default();
                build_splits_of_face(&a_f, &a_ce1, self.faces_origins()?, &mut a_lf_images1)?;

                // check if the rebuilding has added some new faces to the splits
                let mut kept = List::default();
                let images1: Vec<Shape> = a_lf_images1.iter().cloned().collect();
                for f_im in &images1 {
                    let mut b_all_inv = true;
                    // Additional check for artificial case
                    // if current image face consist only of edges from aMapEInv and aMENInv
                    // then recheck current face for the further processing
                    let mut a_to_re_check_face = b_artificial_case;
                    let mut exhausted = true;
                    for e in Explorer::new(f_im, ShapeEnum::Edge, ShapeEnum::Shape)? {
                        if !a_map_e_inv.contains(&e) {
                            b_all_inv = false;
                            if !a_me_n_inv.contains(&e) {
                                a_to_re_check_face = false;
                                exhausted = false;
                                break;
                            }
                        }
                    }
                    // if current image face is to recheck then check number of connections for this face
                    // with other image faces for current face (the list as it stands natively)
                    if exhausted && a_to_re_check_face {
                        let mut current = kept.clone();
                        let pos = images1.iter().position(|x| x.is_equal(f_im)).unwrap_or(0);
                        for rest in &images1[pos..] {
                            current.append(rest.clone());
                        }
                        a_to_re_check_face = check_connections_of_face(f_im, &current)?;
                    }
                    // do not delete image face from further processing if aToReCheckFace is true
                    if exhausted && !a_to_re_check_face {
                        if b_all_inv {
                            a_mf_to_check_int.add(f_im.clone());
                        }
                    } else {
                        kept.append(f_im.clone());
                    }
                }
                a_lf_images1 = kept;

                if b_artificial_case {
                    if a_lf_images.len() == a_lf_images1.len() {
                        b_artificial_case = false;
                    } else {
                        a_lf_images = a_lf_images1;
                    }
                }

                if b_artificial_case {
                    // make the face invalid
                    let mut a_me_inv = IndexedMapOfShape::default();
                    // (a missing list of images is a null dereference natively)
                    if !has_images {
                        return Err(Failure::with_message(
                            "Standard_NullObject",
                            "BRepOffset_BuildOffsetFaces::BuildSplitsOfFaces: no images of the face",
                        ));
                    }
                    *self.my_of_images.change_from_key(&a_f)? = a_lf_images.clone();
                    for f_im in a_lf_images.iter() {
                        for e in Explorer::new(f_im, ShapeEnum::Edge, ShapeEnum::Shape)? {
                            if a_map_e_inv.contains(&e) {
                                self.my_invalid_edges.add(e.clone());
                                a_me_inv.add(e);
                            } else {
                                self.my_valid_edges.add(e);
                            }
                        }
                    }
                    self.my_art_invalid_faces
                        .bind(a_f.clone().into_shape(), a_me_inv.clone());
                    a_dmfmie.bind(a_f.clone().into_shape(), a_me_inv);
                    a_lf_done.append(a_f.into_shape());
                    continue;
                }
            }

            // find invalid edges
            self.find_invalid_edges(
                &a_f,
                &a_lf_images,
                &mut a_dmfmve,
                &mut a_dmfmne,
                &mut a_dmfmie,
                &mut a_dmfmvie,
                &mut a_dme_or_le_im,
                &mut a_m_edge_invalid_by_vertex,
                &mut a_m_edge_valid_by_vertex,
            )?;

            // save the new splits
            if !self.my_of_images.contains(&a_f) {
                self.my_of_images
                    .add(a_f.clone().into_shape(), List::default());
            }
            let images = self.my_of_images.change_from_key(&a_f)?;
            images.clear();
            for f in a_lf_images.iter() {
                images.append(f.clone());
            }
            a_lf_done.append(a_f.into_shape());
        }

        if self.my_invalid_edges.is_empty()
            && self.my_art_invalid_faces.is_empty()
            && a_dmfmie.is_empty()
        {
            return Ok(());
        }

        // Additional step to find invalid edges by checking unclassified edges
        // in the splits of SD faces
        self.find_invalid_edges_sd(&a_lf_done, &mut a_dmfmie, &mut a_dmfmve, &mut a_dmfmne)?;

        // Additional step to mark inverted edges located inside loops
        // of invalid edges as invalid as well
        self.make_inverted_edges_invalid(&a_lf_done)?;

        // Build Edge-Face connectivity map to find faces which removal
        // may potentially lead to creation of the holes in the faces
        // preventing from obtaining closed volume in the result
        let mut an_ef_map = IndexedDataMapOfShapeListOfShape::default();
        for i in 1..=self.my_of_images.extent() {
            for f in self.my_of_images.find_from_index(i)?.iter() {
                topexp::map_shapes_and_ancestors(
                    f,
                    ShapeEnum::Edge,
                    ShapeEnum::Face,
                    &mut an_ef_map,
                )?;
            }
        }

        let an_empty_map = MapOfShape::default();
        // invalid faces inside the holes
        let mut a_mf_inv_in_hole = IndexedMapOfShape::default();
        // all hole faces
        let mut a_f_holes = Compound::default();
        bb.make_compound(&mut a_f_holes);
        // Find the faces containing only the inverted edges and the invalid ones
        let mut an_inverted_faces = List::default();

        // find invalid faces
        // considering faces containing only invalid edges as invalid
        for a_f in a_lf_done.iter() {
            let a_f = topods::face(a_f)?;
            let mut a_lf_images = self.my_of_images.find_from_key(&a_f)?.clone();

            let mut a_lf_inv = List::default();
            let b_artificial_case = self.my_art_invalid_faces.is_bound(&a_f);
            if b_artificial_case {
                a_lf_inv = a_lf_images.clone();
            } else {
                // neutral edges
                let p_mne = a_dmfmne
                    .seek(&a_f)
                    .cloned()
                    .unwrap_or_else(|| an_empty_map.clone());
                // find faces inside holes wires
                let mut a_mf_holes = MapOfShape::default();
                let a_f_or =
                    topods::face(self.faces_origins_ref()?.seek(&a_f).ok_or_else(|| {
                        Failure::with_message("Standard_NoSuchObject", "NCollection_DataMap::Find")
                    })?)?;
                self.find_faces_inside_hole_wires(
                    &a_f_or,
                    &a_f,
                    &a_lf_images,
                    &a_dme_or_le_im,
                    &an_ef_map,
                    &mut a_mf_holes,
                )?;
                for h in a_mf_holes.iter() {
                    bb.add(&mut a_f_holes, h)?;
                }
                // find invalid faces
                self.find_invalid_faces(
                    &mut a_lf_images,
                    &a_dmfmve,
                    &a_dmfmie,
                    &p_mne,
                    &a_m_edge_invalid_by_vertex,
                    &a_m_edge_valid_by_vertex,
                    &a_mf_holes,
                    &mut a_mf_inv_in_hole,
                    &mut a_lf_inv,
                    &mut an_inverted_faces,
                )?;
                *self.my_of_images.change_from_key(&a_f)? = a_lf_images.clone();
            }

            if !a_lf_inv.is_empty() {
                if let Some(count) = self.my_already_inv_faces.seek(&a_f)
                    && *count > 2
                {
                    if a_lf_inv.len() == a_lf_images.len() && !b_artificial_case {
                        self.my_of_images.change_from_key(&a_f)?.clear();
                    }
                    a_lf_inv.clear();
                }
                self.my_invalid_faces.add(a_f.into_shape(), a_lf_inv);
            }
        }

        if self.my_invalid_faces.is_empty() {
            self.my_invalid_edges.clear(true);
            return Ok(());
        }

        let mut a_me_removed = IndexedMapOfShape::default();
        // remove invalid splits of faces using inverted edges
        self.remove_invalid_splits_by_inverted_edges(&mut a_me_removed)?;
        if self.my_invalid_faces.is_empty() {
            self.my_invalid_edges.clear(true);
            return Ok(());
        }

        // remove invalid splits from valid splits
        self.remove_invalid_splits_from_valid(&a_dmfmvie)?;

        // remove inside faces
        let mut inside_edges = std::mem::take(&mut self.my_inside_edges);
        let r = self.remove_inside_faces(
            &an_inverted_faces,
            &a_mf_to_check_int,
            &a_mf_inv_in_hole,
            a_f_holes.as_ref(),
            &mut a_me_removed,
            &mut inside_edges,
        );
        self.my_inside_edges = inside_edges;
        r?;

        // make compound of valid splits
        let mut a_cf_im = Compound::default();
        bb.make_compound(&mut a_cf_im);
        for i in 1..=self.my_of_images.extent() {
            for f_im in self.my_of_images.find_from_index(i)?.iter() {
                bb.add(&mut a_cf_im, f_im)?;
            }
        }

        let mut a_dmef = IndexedDataMapOfShapeListOfShape::default();
        topexp::map_shapes_and_ancestors(&a_cf_im, ShapeEnum::Edge, ShapeEnum::Face, &mut a_dmef)?;

        // filter maps of images and origins
        self.filter_edges_images(&a_cf_im)?;

        // filter invalid faces
        let removed_or_inside = if a_me_removed.extent() > 0 {
            self.my_inside_edges.clone()
        } else {
            a_me_removed.clone()
        };
        self.filter_invalid_faces(&a_dmef, &removed_or_inside)?;
        if self.my_invalid_faces.extent() == 0 {
            self.my_invalid_edges.clear(true);
            return Ok(());
        }

        // filter invalid edges
        let mut a_me_use_in_rebuild = MapOfShape::default();
        self.filter_invalid_edges(
            &a_dmfmie,
            &a_me_removed,
            &removed_or_inside,
            &mut a_me_use_in_rebuild,
        )?;

        // Check additionally validity of edges originated from vertices.
        self.check_edges_created_by_vertex()?;

        self.my_last_inv_edges.clear(true);
        for i in 1..=self.my_invalid_edges.extent() {
            let e = self.my_invalid_edges.find_key(i)?.clone();
            self.my_last_inv_edges.add(e.clone());
            if !a_me_use_in_rebuild.contains(&e) {
                self.my_edges_to_avoid.add(e);
            }
        }

        for i in 1..=self.my_invalid_faces.extent() {
            let f = self.my_invalid_faces.find_key(i)?.clone();
            if let Some(count) = self.my_already_inv_faces.change_seek(&f) {
                *count += 1;
            } else {
                self.my_already_inv_faces.bind(f, 1);
            }
        }
        Ok(())
    }

    /// `GetEdges(theFace, theEdges, theInvMap)`: the images of the descendants of the
    /// face to split it with (not the edges to avoid, collected in `theInvMap`, nor the
    /// micro edges); true when some edge is not a boundary of the face and some edge is
    /// new.
    pub(super) fn get_edges(
        &mut self,
        face: &Face,
        edges: &mut Shape,
        mut inv_map: Option<&mut IndexedMapOfShape>,
    ) -> Result<bool, Failure> {
        // get boundary edges
        let mut a_mf_bounds = MapOfShape::default();
        for e in Explorer::new(face, ShapeEnum::Edge, ShapeEnum::Shape)? {
            if let Some(le_im) = self.my_oe_images.seek(&e) {
                for x in le_im.iter() {
                    a_mf_bounds.add(x.clone());
                }
            } else {
                a_mf_bounds.add(e);
            }
        }

        let bb = BRepBuilder::default();
        let (mut b_found, mut b_update) = (false, false);
        // the resulting edges
        let mut an_edges = Compound::default();
        bb.make_compound(&mut an_edges);
        // Fence map
        let mut a_me_fence = MapOfShape::default();
        // the edges by which the offset face should be split
        for e in self.descendants(face)? {
            let e = topods::edge(&e)?;
            if !b_update {
                b_update = self.my_modified_edges.contains(&e);
            }
            if let Some(le_im) = self.my_oe_images.seek(&e).cloned() {
                for e_im in le_im.iter() {
                    let e_im = topods::edge(e_im)?;
                    if !a_me_fence.add(e_im.clone().into_shape()) {
                        continue;
                    }
                    if self.my_edges_to_avoid.contains(&e_im) {
                        if let Some(m) = inv_map.as_deref_mut() {
                            m.add(e_im.clone().into_shape());
                        }
                        if !b_update {
                            b_update = self.my_last_inv_edges.contains(&e_im);
                        }
                        continue;
                    }
                    // check for micro edge
                    if process_micro_edge(&e_im, &self.my_context)? {
                        continue;
                    }
                    bb.add(&mut an_edges, &e_im)?;
                    if !b_found {
                        b_found = !a_mf_bounds.contains(&e_im);
                    }
                    if !b_update {
                        b_update = self.my_modified_edges.contains(&e_im);
                    }
                }
            } else {
                if self.my_edges_to_avoid.contains(&e) {
                    if let Some(m) = inv_map.as_deref_mut() {
                        m.add(e.clone().into_shape());
                    }
                    if !b_update {
                        b_update = self.my_last_inv_edges.contains(&e);
                    }
                    continue;
                }
                if process_micro_edge(&e, &self.my_context)? {
                    continue;
                }
                bb.add(&mut an_edges, &e)?;
                if !b_found {
                    b_found = !a_mf_bounds.contains(&e);
                }
            }
        }
        *edges = an_edges.into_shape();
        Ok(b_found && b_update)
    }

    /// `CheckIfArtificial(theF, theLFImages, theCE, theMapEInv, theMENInv)`: whether some
    /// splitting edge is unused by the splits although the invalid edges are left out
    /// (the edges at the vertices of the invalid ones, unused, go to `theMENInv`).
    pub(super) fn check_if_artificial(
        &self,
        f: &Shape,
        lf_images: &List<Shape>,
        ce: &Shape,
        map_e_inv: &IndexedMapOfShape,
        me_n_inv: &mut MapOfShape,
    ) -> Result<bool, Failure> {
        // all boundary edges should be used
        let mut a_me_used = IndexedMapOfShape::default();
        for f_im in lf_images.iter() {
            topexp::map_shapes_of_type(f_im, ShapeEnum::Edge, &mut a_me_used)?;
            topexp::map_shapes_of_type(f_im, ShapeEnum::Vertex, &mut a_me_used)?;
        }

        let mut a_mve = IndexedDataMapOfShapeListOfShape::default();
        topexp::map_shapes_and_ancestors(ce, ShapeEnum::Vertex, ShapeEnum::Edge, &mut a_mve)?;

        for i in 1..=map_e_inv.extent() {
            let e_inv = map_e_inv.find_key(i)?;
            for v_e_inv in Explorer::new(e_inv, ShapeEnum::Vertex, ShapeEnum::Shape)? {
                if a_mve.contains(&v_e_inv) {
                    for e_n_inv in a_mve.find_from_key(&v_e_inv)?.iter() {
                        if !a_me_used.contains(e_n_inv) {
                            me_n_inv.add(e_n_inv.clone());
                        }
                    }
                }
            }
        }

        if me_n_inv.is_empty() {
            return Ok(false);
        }

        let mut a_me_found = IndexedMapOfShape::default();
        topexp::map_shapes_of_type(ce, ShapeEnum::Edge, &mut a_me_found)?;

        for e in self.descendants(f)? {
            let e = topods::edge(&e)?;
            if let Some(le_im) = self.my_oe_images.seek(&e) {
                let mut b_checked = false;
                let mut exhausted = true;
                for e_im in le_im.iter() {
                    let e_im = topods::edge(e_im)?;
                    if !a_me_found.contains(&e_im) || me_n_inv.contains(&e_im) {
                        continue;
                    }
                    b_checked = true;
                    if a_me_used.contains(&e_im) {
                        exhausted = false;
                        break;
                    }
                }
                if b_checked && exhausted {
                    return Ok(true);
                }
            } else if a_me_found.contains(&e) && !me_n_inv.contains(&e) && !a_me_used.contains(&e) {
                return Ok(true);
            }
        }
        Ok(false)
    }
}
