// Rust translation of BRepOffset_BuildOffsetFaces::FilterEdgesImages, FilterInvalidFaces,
// CheckEdgesCreatedByVertex, FilterInvalidEdges and FindFacesToRebuild of
// ModelingAlgorithms/TKOffset/BRepOffset/BRepOffset_MakeOffset_1.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The images, invalid faces and invalid edges filtered after the removal of splits, and
//! the offset faces to rebuild.
use super::{BuildOffsetFaces, ShapeIndexedMapMap, ShapeListMap, append_to_list};
use rustycad_tkbo::bop_tools;
use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::topexp::{self, Explorer};
use rustycad_tkbrep::topods::{Compound, Shape};
use rustycad_tkbrep::toptools::{IndexedDataMapOfShapeListOfShape, IndexedMapOfShape, MapOfShape};
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::ShapeEnum;

impl BuildOffsetFaces<'_> {
    /// `FilterEdgesImages(theS)`: the images of the offset edges reduced to the edges of the
    /// shape, the origins rebuilt from them.
    ///
    /// The native loop runs over the hashed map of images: the order of the origins of an
    /// image edge with several origins follows the addresses of the shapes.
    pub(super) fn filter_edges_images(&mut self, s: &Shape) -> Result<(), Failure> {
        // map edges
        let mut me = IndexedMapOfShape::default();
        topexp::map_shapes_of_type(s, ShapeEnum::Edge, &mut me)?;

        self.my_oe_origins.clear(true);
        let keys: Vec<Shape> = self.my_oe_images.items().map(|(k, _)| k.clone()).collect();
        for e in &keys {
            let Some(le_im) = self.my_oe_images.change_seek(e) else {
                continue;
            };
            // filter images
            // (edges with no images left should be kept in the map
            // to avoid their usage when building the splits of faces)
            le_im.retain(|e_im| me.contains(e_im));
            let kept: Vec<Shape> = le_im.iter().cloned().collect();
            // save origins
            for e_im in &kept {
                if let Some(l_or) = self.my_oe_origins.change_seek(e_im) {
                    append_to_list(l_or, e);
                } else {
                    let mut l_or = List::default();
                    l_or.append(e.clone());
                    self.my_oe_origins.bind(e_im.clone(), l_or);
                }
            }
        }
        Ok(())
    }

    /// `FilterInvalidFaces(theDMEF, theMERemoved)`: the invalid faces kept only when their
    /// invalid splits are among the valid ones, or the valid splits have free edges, unless
    /// all invalid edges are inside the splits of the face (then the invalid splits are
    /// returned to the images).
    pub(super) fn filter_invalid_faces(
        &mut self,
        dmef: &IndexedDataMapOfShapeListOfShape,
        me_removed: &IndexedMapOfShape,
    ) -> Result<(), Failure> {
        // filter invalid faces, considering faces having only valid
        // images left with non-free edges as valid
        // do not remove invalid faces if it creates free edges
        let mut really_inv_faces = IndexedDataMapOfShapeListOfShape::default();
        // Edge-Face connexity map of all splits, both invalid and valid
        let mut dmef_all = IndexedDataMapOfShapeListOfShape::default();

        let nb = self.my_invalid_faces.extent();
        for i in 1..=nb {
            let f = self.my_invalid_faces.find_key(i)?.clone();
            let lf_inv = self.my_invalid_faces.find_from_index(i)?.clone();

            if self.my_art_invalid_faces.is_bound(&f) {
                if lf_inv.is_empty() {
                    self.my_art_invalid_faces.unbind(&f);
                } else {
                    really_inv_faces.add(f, lf_inv);
                }
                continue;
            }

            if lf_inv.is_empty() {
                continue;
            }

            let lf_im = self.my_of_images.find_from_key(&f)?.clone();
            let mut b_invalid = lf_im.is_empty();

            if !b_invalid {
                // check two lists on common splits
                b_invalid = lf_inv
                    .iter()
                    .any(|f_inv| lf_im.iter().any(|f_im| f_inv.is_same(f_im)));
            }

            if !b_invalid {
                // check for free edges
                for li in [&lf_im, &lf_inv] {
                    if b_invalid {
                        break;
                    }
                    'faces: for f_im in li.iter() {
                        for e in Explorer::new(f_im, ShapeEnum::Edge, ShapeEnum::Shape)? {
                            if !me_removed.contains(&e)
                                && dmef.seek(&e).is_some_and(|lef| lef.len() == 1)
                            {
                                b_invalid = true;
                                break 'faces;
                            }
                        }
                    }
                }
            }
            if b_invalid {
                if dmef_all.is_empty() {
                    dmef_all = dmef.clone();
                    for i_f in 1..=nb {
                        for f_inv in self.my_invalid_faces.find_from_index(i_f)?.iter() {
                            topexp::map_shapes_and_ancestors(
                                f_inv,
                                ShapeEnum::Edge,
                                ShapeEnum::Face,
                                &mut dmef_all,
                            )?;
                        }
                    }
                }

                let mut local_splits = MapOfShape::default();
                for x in lf_im.iter().chain(lf_inv.iter()) {
                    local_splits.add(x.clone());
                }

                // Check if all invalid edges are located inside the split and do not touch
                // any other faces both invalid and valid
                b_invalid = false;
                'splits: for f_im in lf_inv.iter() {
                    for e in Explorer::new(f_im, ShapeEnum::Edge, ShapeEnum::Shape)? {
                        if self.my_invalid_edges.contains(&e) && !me_removed.contains(&e) {
                            let lf = dmef_all.find_from_key(&e)?;
                            if lf.iter().any(|x| !local_splits.contains(x)) {
                                b_invalid = true;
                                break 'splits;
                            }
                        }
                    }
                }
                if !b_invalid {
                    let images = self.my_of_images.change_from_key(&f)?;
                    for x in lf_inv.iter() {
                        append_to_list(images, x);
                    }
                }
            }

            if b_invalid {
                really_inv_faces.add(f, lf_inv);
            }
        }

        self.my_invalid_faces = really_inv_faces;
        Ok(())
    }

    /// `CheckEdgesCreatedByVertex()`: the unchecked edges of the invalid splits originated
    /// only from vertices made invalid.
    pub(super) fn check_edges_created_by_vertex(&mut self) -> Result<(), Failure> {
        // Mark the unchecked edges contained in invalid faces as invalid
        for i in 1..=self.my_invalid_faces.extent() {
            let f = self.my_invalid_faces.find_key(i)?.clone();
            if self.my_art_invalid_faces.is_bound(&f) {
                continue;
            }

            let lf_im = self.my_invalid_faces.find_from_index(i)?.clone();
            for f_im in lf_im.iter() {
                for e in Explorer::new(f_im, ShapeEnum::Edge, ShapeEnum::Shape)? {
                    if self.my_invalid_edges.contains(&e) || self.my_valid_edges.contains(&e) {
                        continue;
                    }

                    // check if this edges is not created from vertex and mark it as invalid
                    let Some(le_or) = self.edges_origins().seek(&e) else {
                        continue;
                    };
                    let mut all_vertices = true;
                    for x in le_or.iter() {
                        if x.shape_type()? != ShapeEnum::Vertex {
                            all_vertices = false;
                            break;
                        }
                    }
                    if all_vertices {
                        self.my_invalid_edges.add(e);
                    }
                }
            }
        }
        Ok(())
    }

    /// `FilterInvalidEdges(theDMFMIE, theMERemoved, theMEInside, theMEUseInRebuild)`: the
    /// invalid edges reduced to those of the invalid splits not in blocks of removed edges;
    /// the inside ones usable for rebuilding collected.
    pub(super) fn filter_invalid_edges(
        &mut self,
        dmfmie: &ShapeIndexedMapMap,
        me_removed: &IndexedMapOfShape,
        me_inside: &IndexedMapOfShape,
        me_use_in_rebuild: &mut MapOfShape,
    ) -> Result<(), Failure> {
        let bb = BRepBuilder::default();
        let mut ce_inv = Compound::default();
        let mut me_inv = IndexedMapOfShape::default();
        bb.make_compound(&mut ce_inv);

        for i in 1..=self.my_invalid_faces.extent() {
            for f_im in self.my_invalid_faces.find_from_index(i)?.iter() {
                topexp::map_shapes_of_type(f_im, ShapeEnum::Edge, &mut me_inv)?;

                for e in Explorer::new(f_im, ShapeEnum::Edge, ShapeEnum::Shape)? {
                    if self.my_invalid_edges.contains(&e) {
                        bb.add(&mut ce_inv, &e)?;
                    }
                }
            }
        }

        // remove edges which have been marked for removal
        let mut me_inv_to_avoid = IndexedMapOfShape::default();
        let mut lcbe = Vec::new();
        bop_tools::make_connexity_compounds(
            &ce_inv,
            ShapeEnum::Vertex,
            ShapeEnum::Edge,
            &mut lcbe,
        )?;

        for cbe in &lcbe {
            let mut all_removed = true;
            for e in Explorer::new(cbe, ShapeEnum::Edge, ShapeEnum::Shape)? {
                if !me_removed.contains(&e) {
                    all_removed = false;
                    break;
                }
            }

            if all_removed {
                topexp::map_shapes_of_type(cbe, ShapeEnum::Edge, &mut me_inv_to_avoid)?;
            }
        }

        let mut really_inv_edges = IndexedMapOfShape::default();

        for i in 1..=self.my_invalid_faces.extent() {
            let f = self.my_invalid_faces.find_key(i)?;
            if self.my_art_invalid_faces.is_bound(f) {
                if let Some(mie) = dmfmie.seek(f) {
                    for i_e in 1..=mie.extent() {
                        let e = mie.find_key(i_e)?;
                        if me_inv.contains(e) && !me_inv_to_avoid.contains(e) {
                            really_inv_edges.add(e.clone());
                        }
                    }
                }
            } else {
                for f_im in self.my_invalid_faces.find_from_index(i)?.iter() {
                    for e in Explorer::new(f_im, ShapeEnum::Edge, ShapeEnum::Shape)? {
                        if self.my_invalid_edges.contains(&e) && !me_inv_to_avoid.contains(&e) {
                            really_inv_edges.add(e);
                        }
                    }
                }
            }
        }

        self.my_invalid_edges = really_inv_edges;

        // Check if any of the currently invalid edges may be used for
        // rebuilding splits of invalid faces.
        // For that the edge should be inside and not connected to invalid
        // boundary edges of the same origin.
        for i in 1..=self.my_invalid_edges.extent() {
            let e = self.my_invalid_edges.find_key(i)?;
            if !me_inside.contains(e) || !self.my_valid_edges.contains(e) {
                continue;
            }

            let Some(e_origins) = self.my_oe_origins.seek(e) else {
                continue;
            };

            let mut b_has_inv_outside = false;
            for e_or in e_origins.iter() {
                if b_has_inv_outside {
                    break;
                }
                if let Some(e_ims) = self.my_oe_images.seek(e_or) {
                    for e_im in e_ims.iter() {
                        if b_has_inv_outside {
                            break;
                        }
                        b_has_inv_outside =
                            self.my_invalid_edges.contains(e_im) && !me_inside.contains(e_im);
                    }
                }
            }
            if !b_has_inv_outside {
                me_use_in_rebuild.add(e.clone());
            }
        }
        Ok(())
    }

    /// `FindFacesToRebuild()`: the offset faces to rebuild (interfering with invalid faces,
    /// sharing edges or vertices of invalid splits, or invalid themselves), with their valid
    /// edges.
    pub(super) fn find_faces_to_rebuild(&mut self) -> Result<(), Failure> {
        if self.my_of_images.extent() == 0 {
            return Ok(());
        }

        let mut m_fence = MapOfShape::default();
        let mut me_reb = MapOfShape::default();
        let mut mf_reb = MapOfShape::default();

        let mut dmflv = ShapeListMap::default();
        // get edges from invalid faces
        for i in 1..=self.my_invalid_faces.extent() {
            let f = self.my_invalid_faces.find_key(i)?;
            m_fence.clear(true);
            let mut lv_avoid: List<Shape> = List::default();
            for f_im in self.my_invalid_faces.find_from_index(i)?.iter() {
                for e in Explorer::new(f_im, ShapeEnum::Edge, ShapeEnum::Shape)? {
                    me_reb.add(e.clone());
                    if self.my_invalid_edges.contains(&e) {
                        for v in Explorer::new(&e, ShapeEnum::Vertex, ShapeEnum::Shape)? {
                            if m_fence.add(v.clone()) {
                                lv_avoid.append(v.clone());
                                me_reb.add(v);
                            }
                        }
                    }
                }
            }

            if !lv_avoid.is_empty() {
                dmflv.bind(f.clone(), lv_avoid);
            }

            let p_lf = if !self.my_art_invalid_faces.is_bound(f) {
                self.my_ss_interfs.seek(f)
            } else {
                self.my_ss_interfs_art.seek(f)
            };
            if let Some(lf) = p_lf {
                for fe in lf.iter() {
                    mf_reb.add(fe.clone());
                }
            }
        }

        // get face to rebuild
        for i in 1..=self.my_of_images.extent() {
            let f = self.my_of_images.find_key(i)?.clone();
            let lf_im = self.my_of_images.find_from_index(i)?.clone();
            let mut mv_avoid = MapOfShape::default();
            if let Some(lv_avoid) = dmflv.seek(&f) {
                for v in lv_avoid.iter() {
                    mv_avoid.add(v.clone());
                }
            }

            let mut b_rebuild = mf_reb.contains(&f);
            let mut le_valid: List<Shape> = List::default();
            m_fence.clear(true);

            for f_im in lf_im.iter() {
                for e_im in Explorer::new(f_im, ShapeEnum::Edge, ShapeEnum::Shape)? {
                    if !self.my_invalid_edges.contains(&e_im) && m_fence.add(e_im.clone()) {
                        le_valid.append(e_im.clone());
                    }

                    if !b_rebuild {
                        b_rebuild = me_reb.contains(&e_im);
                    }

                    if !b_rebuild {
                        // check vertices
                        for v in Explorer::new(&e_im, ShapeEnum::Vertex, ShapeEnum::Shape)? {
                            if !mv_avoid.contains(&v) {
                                b_rebuild = me_reb.contains(&v);
                            }
                            if b_rebuild {
                                break;
                            }
                        }
                    }
                }
            }

            if !b_rebuild {
                b_rebuild = !lf_im.is_empty() && self.my_invalid_faces.contains(&f);
                if b_rebuild {
                    self.my_f_self_reb_avoid.add(f.clone());
                }
            }

            if b_rebuild {
                self.my_faces_to_rebuild.add(f, le_valid);
            }
        }
        Ok(())
    }
}
