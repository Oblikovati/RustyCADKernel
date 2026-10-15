// Rust translation of BRepOffset_BuildOffsetFaces::GetBounds, GetBoundsToUpdate,
// GetInvalidEdgesByBounds and UpdateNewIntersectionEdges of
// ModelingAlgorithms/TKOffset/BRepOffset/BRepOffset_MakeOffset_1.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The bounds of the splits of the offset faces, the new splits filtered by their
//! intersection with the bounds, and the maps updated with the new intersection edges.
use super::trim_edges::{ds_of, not_found};
use super::{BuildOffsetFaces, ShapeListMap, append_to_list};
use rustycad_tkbo::bop_algo::BOPAlgoSection;
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::topexp::{self, Explorer};
use rustycad_tkbrep::topods::{self, Compound, Shape, ShapeIterator, Vertex};
use rustycad_tkbrep::toptools::{IndexedDataMapOfShapeListOfShape, IndexedMapOfShape, MapOfShape};
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::{ShapeEnum, State};
use rustycad_tkmath::gp::Point2;

/// A vertex in the middle of the 3D curve of the edge, with the confusion tolerance.
fn middle_vertex(e: &Shape) -> Result<Shape, Failure> {
    let (c, f, l) = BRepTool::curve(&topods::edge(e)?)?;
    let c = c
        .get()
        .ok_or_else(|| Failure::with_message("Standard_NullObject", "BRep_Tool::Curve"))?;
    let mut v = Vertex::default();
    BRepBuilder::default().make_vertex_at(
        &mut v,
        c.value((f + l) * 0.5)?,
        precision::confusion(),
    )?;
    Ok(v.into_shape())
}

impl BuildOffsetFaces<'_> {
    /// `GetBounds(theLFaces, theMEB, theBounds)`: the edges of the splits of the faces, but
    /// the bounding edges to replace.
    pub(super) fn get_bounds(
        &self,
        l_faces: &List<Shape>,
        meb: &MapOfShape,
    ) -> Result<Shape, Failure> {
        let bb = BRepBuilder::default();
        // Make compound of edges contained in the splits of faces
        let mut bounds = Compound::default();
        bb.make_compound(&mut bounds);
        // Fence map
        let mut m_fence = MapOfShape::default();

        for f in l_faces.iter() {
            let Some(lf_im) = self.my_of_images.seek(f) else {
                continue;
            };
            for f_im in lf_im.iter() {
                for e_im in Explorer::new(f_im, ShapeEnum::Edge, ShapeEnum::Shape)? {
                    if !meb.contains(&e_im) && m_fence.add(e_im.clone()) {
                        bb.add(&mut bounds, &e_im)?;
                    }
                }
            }
        }
        Ok(bounds.into_shape())
    }

    /// `GetBoundsToUpdate(theLF, theMEB, theLABounds, theLAValid, theBounds)`: the edges of
    /// the faces in the AsDes (or their images) to update, with their origins.
    pub(super) fn get_bounds_to_update(
        &self,
        lf: &List<Shape>,
        meb: &MapOfShape,
        la_bounds: &mut List<Shape>,
        la_valid: &mut List<Shape>,
    ) -> Result<Shape, Failure> {
        // get all edges
        let bb = BRepBuilder::default();
        let mut bounds = Compound::default();
        bb.make_compound(&mut bounds);

        let mut ma_valid = MapOfShape::default();
        let mut m_fence = MapOfShape::default();

        for f in lf.iter() {
            let mut mde = IndexedMapOfShape::default();
            for ed in self.descendants(f)? {
                let Some(led_im) = self.my_oe_images.seek(&ed) else {
                    mde.add(ed);
                    continue;
                };
                for ed_im in led_im.iter() {
                    mde.add(ed_im.clone());
                }
            }

            for j in 1..=mde.extent() {
                let e_im = topods::edge(mde.find_key(j)?)?.into_shape();

                if !meb.contains(&e_im) && m_fence.add(e_im.clone()) {
                    bb.add(&mut bounds, &e_im)?;
                    la_bounds.append(e_im.clone());
                }

                if let Some(lo) = self.my_oe_origins.seek(&e_im) {
                    for eo in lo.iter() {
                        if ma_valid.add(eo.clone()) {
                            la_valid.append(eo.clone());
                        }
                    }
                } else if ma_valid.add(e_im.clone()) {
                    la_valid.append(e_im);
                }
            }
        }
        Ok(bounds.into_shape())
    }

    /// `GetInvalidEdgesByBounds(...)`: the splits sectioned with the bounds; the vertices of
    /// the splits intersected by invalid edges, of the partially sectioned splits and of the
    /// extended checks, not lying on the splits of their faces nor inside the solids, made
    /// vertices to avoid, their edges invalid.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn get_invalid_edges_by_bounds(
        &self,
        splits: &Shape,
        bounds: &Shape,
        mv_old: &MapOfShape,
        me_new: &MapOfShape,
        dme_or: &ShapeListMap,
        melf: &ShapeListMap,
        e_images: &ShapeListMap,
        me_check_ext: &MapOfShape,
        me_inv_on_art: &MapOfShape,
        verts_to_avoid: &mut MapOfShape,
        me_inv: &mut MapOfShape,
    ) -> Result<(), Failure> {
        // map splits to check the vertices of edges
        let mut dmve = IndexedDataMapOfShapeListOfShape::default();
        topexp::map_shapes_and_ancestors(splits, ShapeEnum::Vertex, ShapeEnum::Edge, &mut dmve)?;

        let mut sec = BOPAlgoSection::new();
        sec.add_argument(splits);
        sec.add_argument(bounds);

        sec.perform();

        // invalid vertices
        let mut mv_inv = IndexedMapOfShape::default();
        // vertices to check additionally by classification relatively to solid
        let mut mv_check_add = MapOfShape::default();
        // collect parts for removal
        let ds = ds_of(&sec)?;

        // check edge/edge intersections
        for ee in ds.interf_ee_records().iter() {
            let e1 = ds.shape(ee.index1())?;
            let e2 = ds.shape(ee.index2())?;

            if !ee.has_index_new() {
                if me_check_ext.contains(e1) && ee.common_part().shape_type() == ShapeEnum::Edge {
                    me_inv.add(e1.clone());
                }
                continue;
            }

            if self.my_invalid_edges.contains(e2) {
                me_inv.add(e1.clone());
            }

            if me_inv_on_art.contains(e2) {
                // avoid checking of the vertices of the split edge intersected by
                // the invalid edge from artificial face
                let (v1, v2) = topexp::vertices(&topods::edge(e2)?, false);
                if dmve.contains(&v1) && dmve.contains(&v2) {
                    continue;
                }
            }

            // add vertices of all images of the edge from splits for checking
            for e_or in dme_or.seek(e1).ok_or_else(not_found)?.iter() {
                let Some(le_im) = e_images.seek(e_or) else {
                    continue;
                };
                for e_im in le_im.iter() {
                    for v in ShapeIterator::new(e_im, true, true) {
                        if !mv_old.contains(&v) {
                            mv_inv.add(v.clone());
                            mv_check_add.add(v);
                        }
                    }
                }
            }
        }

        // to avoid unnecessary filling of parts due to extra trim of the edges
        // process Edge/Edge interferences of type EDGE, i.e. common blocks and check
        // not the bounding vertices of the edges, but check the edge itself
        // to be lying on some face
        //
        // all common blocks are contained in the result of SECTION operation
        // between sets of edges
        let sec_r = sec.shape().clone();

        let mut ms_sec = IndexedMapOfShape::default();
        topexp::map_shapes(&sec_r, &mut ms_sec, true, true);

        let im = sec.images();
        for e in Explorer::new(splits, ShapeEnum::Edge, ShapeEnum::Shape)? {
            if sec.is_deleted(&e)? {
                // no common blocks for this edge
                continue;
            }

            let Some(le_im) = im.seek(&e) else {
                // no splits, i.e. completely coincides with some edge from boundary
                continue;
            };

            for e_im in le_im.iter() {
                if !ms_sec.contains(e_im) {
                    // the edge included in section only partially.
                    // the part not included in section may be excessive
                    //
                    // check vertices of this edge - if one of them is new
                    // the edge might be removed
                    let (v1, v2) = topexp::vertices(&topods::edge(e_im)?, false);
                    if !mv_old.contains(&v1) || !mv_old.contains(&v2) {
                        // add this edge for checking by making new vertex in the middle of the edge
                        let v = middle_vertex(e_im)?;
                        // and adding this vertex for checking
                        let idx = dmve.add(v.clone(), List::default());
                        dmve.change_from_index(idx)?.append(e.clone());
                        mv_inv.add(v);
                        break;
                    }
                }
            }
        }

        // Add for check also the edges created from common between splits
        // of offset faces edges not connected to any invalidity.
        // These edges may also accidentally fill some part.
        for e in me_check_ext.iter() {
            // make new vertex in the middle of the edge
            let v = middle_vertex(e)?;
            // add this vertex for checking
            let idx = dmve.add(v.clone(), List::default());
            dmve.change_from_index(idx)?.append(e.clone());
            mv_inv.add(v);
        }

        // add for check also the vertices connected only to new or old edges
        for i in 1..=dmve.extent() {
            let v = dmve.find_key(i)?;
            if mv_old.contains(v) {
                continue;
            }

            let (mut b_new, mut b_old) = (false, false);
            for e in dmve.find_from_index(i)?.iter() {
                if me_check_ext.contains(e) {
                    continue;
                }

                if me_new.contains(e) {
                    b_new = true;
                } else {
                    b_old = true;
                }

                if b_new && b_old {
                    break;
                }
            }

            if !b_new || !b_old {
                mv_inv.add(v.clone());
                mv_check_add.remove(v);
            }
        }

        // perform the checking of the vertices
        let context = self.context()?;
        for iv in 1..=mv_inv.extent() {
            let v = topods::vertex(mv_inv.find_key(iv)?)?;
            if mv_old.contains(&v) {
                continue;
            }

            let Some(le_inv) = dmve.seek(&v) else {
                continue;
            };
            // find faces by the edges to check the vertex
            let mut mf = IndexedMapOfShape::default();
            for e in le_inv.iter() {
                for f in melf.seek(e).ok_or_else(not_found)?.iter() {
                    mf.add(f.clone());
                }
            }

            // check the vertex to belong to some split of the faces
            let mut b_invalid = true;

            let mut i = 1;
            while i <= mf.extent() && b_invalid {
                let f = topods::face(mf.find_key(i)?)?;
                let lf_im = self.my_of_images.find_from_key(&f)?;

                'splits: for f_im in lf_im.iter() {
                    for vf in Explorer::new(f_im, ShapeEnum::Vertex, ShapeEnum::Shape)? {
                        b_invalid = !vf.is_same(&v);
                        if !b_invalid {
                            break 'splits;
                        }
                    }
                }

                if b_invalid {
                    let (mut u, mut vv, mut tol) = (0.0, 0.0, 0.0);
                    let i_status = context.compute_vf(&v, &f, &mut u, &mut vv, &mut tol, 0.0)?;
                    if i_status == 0 {
                        // classify the point relatively faces
                        let p2d = Point2::new(u, vv);
                        for f_im in lf_im.iter() {
                            b_invalid = !context.is_point_in_on_face(&topods::face(f_im)?, p2d)?;
                            if !b_invalid {
                                break;
                            }
                        }
                    }
                }
                i += 1;
            }

            if b_invalid && mv_check_add.contains(&v) {
                // the vertex is invalid for all faces
                // check the same vertex for the solids
                let p = BRepTool::pnt(&v)?;
                let tol_v = BRepTool::vertex_tolerance(&v)?;

                if !self.my_solids.is_null() {
                    for sol in Explorer::new(&self.my_solids, ShapeEnum::Solid, ShapeEnum::Shape)? {
                        if !b_invalid {
                            break;
                        }
                        let cached = context.solid_classifier(&topods::solid(&sol)?)?;
                        let mut sc = cached.write();
                        sc.perform(&p, tol_v)?;
                        b_invalid = sc.state() == State::Out;
                    }
                }
            }

            if b_invalid {
                verts_to_avoid.add(v.into_shape());
                for e in le_inv.iter() {
                    me_inv.add(e.clone());
                }
            }
        }
        Ok(())
    }

    /// `UpdateNewIntersectionEdges(theLE, theMELF, theEImages, theEETrim)`: the new
    /// intersection edges and their splits recorded in the AsDes, the images, the origins and
    /// the trimmed-to-untrimmed map; the trimmed edges they replace updated.
    pub(super) fn update_new_intersection_edges(
        &mut self,
        le: &List<Shape>,
        melf: &ShapeListMap,
        e_images: &ShapeListMap,
        ee_trim: &mut ShapeListMap,
    ) -> Result<(), Failure> {
        let le_im_empty: List<Shape> = List::default();
        // update global maps of images and origins with new splits
        for e in le.iter() {
            if !e_images.is_bound(e) {
                let Some(let_) = ee_trim.change_seek(e) else {
                    continue;
                };

                let (invalid_edges, inverted_edges) =
                    (&self.my_invalid_edges, &self.my_inverted_edges);
                let_.retain(|et| invalid_edges.contains(et) || inverted_edges.contains(et));

                if let_.is_empty() {
                    continue;
                }
            }
            // new images
            let le_new = e_images.seek(e).unwrap_or(&le_im_empty).clone();

            // save connection to untrimmed edge for the next steps
            for et in le_new.iter() {
                self.e_trim_e_inf()?.bind(et.clone(), e.clone());
                self.my_modified_edges.add(et.clone());
            }

            // check if it is existing edge
            if !ee_trim.is_bound(e) {
                let lf = melf.seek(e).ok_or_else(not_found)?;
                // the edge is new
                // add this edge to AsDes
                let as_des = self.as_des()?;
                for f in lf.iter() {
                    as_des.add_shared(f, e);
                }

                // add aE to the images
                self.my_oe_images.bind(e.clone(), le_new.clone());
                self.my_modified_edges.add(e.clone());

                // add to origins
                for e_new in le_new.iter() {
                    if let Some(origins) = self.my_oe_origins.change_seek(e_new) {
                        append_to_list(origins, e);
                    } else {
                        let mut origins = List::default();
                        origins.append(e.clone());
                        self.my_oe_origins.bind(e_new.clone(), origins);
                    }
                }

                // update connection to initial origins
                if let Some(le_or_init) = self.edges_origins().seek(e).cloned() {
                    for e_new in le_new.iter() {
                        let origins = self.edges_origins();
                        if let Some(le_new_or) = origins.change_seek(e_new) {
                            for e_or in le_or_init.iter() {
                                append_to_list(le_new_or, e_or);
                            }
                        } else {
                            origins.bind(e_new.clone(), le_or_init.clone());
                        }
                    }
                }

                continue;
            }

            // old images
            let le_old = ee_trim.seek(e).ok_or_else(not_found)?.clone();

            // list of initial origins
            let mut init_origins: List<Shape> = List::default();

            // it is necessary to replace the old edges with new ones
            for e_old in le_old.iter() {
                if let Some(e_origins) = self.my_oe_origins.seek(e_old).cloned() {
                    // get its origins
                    for e_or in e_origins.iter() {
                        self.my_modified_edges.add(e_or.clone());

                        let e_imgs = self.my_oe_images.change_seek(e_or).ok_or_else(not_found)?;

                        // remove old edge from images
                        e_imgs.retain(|e_im| !e_im.is_same(e_old));

                        // add new images
                        for e_new in le_new.iter() {
                            append_to_list(e_imgs, e_new);
                        }
                        for e_new in le_new.iter() {
                            if let Some(new_origins) = self.my_oe_origins.change_seek(e_new) {
                                append_to_list(new_origins, e_or);
                            } else {
                                let mut new_origins = List::default();
                                new_origins.append(e_or.clone());
                                self.my_oe_origins.bind(e_new.clone(), new_origins);
                            }
                        }
                    }
                } else {
                    // add to images
                    self.my_oe_images.bind(e_old.clone(), le_new.clone());

                    self.my_modified_edges.add(e_old.clone());

                    // add to origins
                    for e_new in le_new.iter() {
                        if let Some(origins) = self.my_oe_origins.change_seek(e_new) {
                            append_to_list(origins, e_old);
                        } else {
                            let mut origins = List::default();
                            origins.append(e_old.clone());
                            self.my_oe_origins.bind(e_new.clone(), origins);
                        }
                    }
                }

                // update connection to initial shape
                if let Some(le_or_init) = self.edges_origins().seek(e_old) {
                    for e_or_init in le_or_init.iter() {
                        append_to_list(&mut init_origins, e_or_init);
                    }
                }
            }

            if !init_origins.is_empty() {
                for e_new in le_new.iter() {
                    let origins = self.edges_origins();
                    if let Some(le_new_or) = origins.change_seek(e_new) {
                        for e_or in init_origins.iter() {
                            append_to_list(le_new_or, e_or);
                        }
                    } else {
                        origins.bind(e_new.clone(), init_origins.clone());
                    }
                }
            }
        }
        Ok(())
    }
}
