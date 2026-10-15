// Rust translation of BRepOffset_BuildOffsetFaces::IntersectFaces (with the static
// mapShapes), PrepareFacesForIntersection, FindVerticesToAvoid, FindFacesForIntersection,
// ProcessCommonEdges, UpdateIntersectedFaces (with the static FindOrigins) and the pair
// IntersectFaces of ModelingAlgorithms/TKOffset/BRepOffset/BRepOffset_MakeOffset_1.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The intersection of the offset faces around each invalidity: the new intersection edges
//! between the faces to rebuild, trimmed and filtered into new images of the faces.
use super::{BuildOffsetFaces, ShapeListMap, ShapeShapeMap, append_to_list, find_common_parts};
use crate::brep_offset::BRepOffsetTool;
use rustycad_tkbo::bop_tools;
use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::topexp::{self, Explorer};
use rustycad_tkbrep::topods::{self, Compound, Edge, Face, Shape, ShapeIterator};
use rustycad_tkbrep::toptools::{IndexedDataMapOfShapeListOfShape, IndexedMapOfShape, MapOfShape};
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::{ShapeEnum, State};

/// Standard_NoSuchObject of a missing data map key.
fn not_found() -> Failure {
    Failure::with_message("Standard_NoSuchObject", "NCollection_DataMap::Find")
}

/// `mapShapes(theVecShapes, theType, theMap)`: the sub-shapes of the type collected.
fn map_shapes(shapes: &List<Shape>, ty: ShapeEnum, map: &mut MapOfShape) -> Result<(), Failure> {
    for s in shapes.iter() {
        for x in Explorer::new(s, ty, ShapeEnum::Shape)? {
            map.add(x);
        }
    }
    Ok(())
}

/// `FindOrigins(theLFIm1, theLFIm2, theME, theOrigins, theLEOr)`: the original edges of the
/// edges of the map in the splits.
fn find_origins(
    lf_im1: &List<Shape>,
    lf_im2: &List<Shape>,
    me: &IndexedMapOfShape,
    origins: &ShapeListMap,
    le_or: &mut List<Shape>,
) -> Result<(), Failure> {
    let mut m_fence = MapOfShape::default();
    for lf in [lf_im1, lf_im2] {
        for f in lf.iter() {
            for e in Explorer::new(f, ShapeEnum::Edge, ShapeEnum::Shape)? {
                if me.contains(&e)
                    && let Some(l) = origins.seek(&e)
                {
                    for e_or in l.iter() {
                        if m_fence.add(e_or.clone()) && e_or.shape_type()? == ShapeEnum::Edge {
                            le_or.append(e_or.clone());
                        }
                    }
                }
            }
        }
    }
    Ok(())
}

/// The maps prepared by `PrepareFacesForIntersection`.
#[derive(Default)]
struct IntersectionMaps {
    fle: IndexedDataMapOfShapeListOfShape,
    m_done: ShapeListMap,
    dmsf: ShapeListMap,
    me_inf_e_trim: ShapeListMap,
    dmve_full: ShapeListMap,
    dmef_inv: IndexedDataMapOfShapeListOfShape,
}

/// `AppendToList` into the list bound to the key (created when absent).
fn append_bound(map: &mut ShapeListMap, key: &Shape, value: &Shape) {
    if !map.is_bound(key) {
        map.bind(key.clone(), List::default());
    }
    if let Some(l) = map.change_seek(key) {
        append_to_list(l, value);
    }
}

/// `AppendToList` into the list bound to the key in an indexed map (created when absent).
fn append_indexed(
    map: &mut IndexedDataMapOfShapeListOfShape,
    key: &Shape,
    value: &Shape,
) -> Result<(), Failure> {
    if !map.contains(key) {
        map.add(key.clone(), List::default());
    }
    append_to_list(map.change_from_key(key)?, value);
    Ok(())
}

impl BuildOffsetFaces<'_> {
    /// `IntersectFaces(theVertsToAvoid, theRange)`: the faces to rebuild intersected around
    /// each block of invalid edges of the invalid faces, the intersection edges trimmed and
    /// filtered into new images.
    pub(super) fn intersect_faces(
        &mut self,
        verts_to_avoid: &mut MapOfShape,
    ) -> Result<(), Failure> {
        if self.my_faces_to_rebuild.extent() == 0 {
            return Ok(());
        }

        // get vertices from invalid edges
        let mut mv_inv = MapOfShape::default();
        let mut mv_inv_all = MapOfShape::default();
        for i in 1..=self.my_invalid_edges.extent() {
            let e_inv = self.my_invalid_edges.find_key(i)?;
            let b_valid = self.my_valid_edges.contains(e_inv);
            for v in Explorer::new(e_inv, ShapeEnum::Vertex, ShapeEnum::Shape)? {
                mv_inv_all.add(v.clone());
                if !b_valid {
                    mv_inv.add(v);
                }
            }
        }

        let b_look_vert_to_avoid = mv_inv.extent() > 0;

        // Add all faces to rebuild to outgoing map <aFLE>,
        // plus link edges and vertices to the faces to
        // define intersection faces
        let mut maps = IntersectionMaps::default();
        self.prepare_faces_for_intersection(b_look_vert_to_avoid, &mut maps)?;

        // Find vertices to avoid while trimming the edges.
        // These vertices are taken from the invalid edges common between
        // splits of different invalid, but not artificially, faces.
        // Additional condition for these vertices is that all
        // edges adjacent to this vertex must be either invalid
        // or contained in invalid faces
        let mut mvr_inv = verts_to_avoid.clone();
        self.find_vertices_to_avoid(&maps.dmef_inv, &maps.dmve_full, &mut mvr_inv)?;

        // The faces should be intersected selectively -
        // intersect only faces neighboring to the same invalid face
        // and connected to its invalid edges;
        // when dealing with artificially invalid faces for intersection to be
        // complete we need to use not only invalid edges, but also the
        // edges connected to invalid ones

        // find blocks of artificially invalid faces
        let mut dmf_im_f = ShapeShapeMap::default();
        let mut cf_art = Compound::default();
        BRepBuilder::default().make_compound(&mut cf_art);
        for (f, _) in self.my_art_invalid_faces.items() {
            for f_inv in self.my_invalid_faces.find_from_key(f)?.iter() {
                BRepBuilder::default().add(&mut cf_art, f_inv)?;
                dmf_im_f.bind(f_inv.clone(), f.clone());
            }
        }

        // make connexity blocks
        let mut lcb_art = Vec::new();
        bop_tools::make_connexity_compounds(
            &cf_art,
            ShapeEnum::Vertex,
            ShapeEnum::Face,
            &mut lcb_art,
        )?;

        // alone edges
        let mut me_alone = MapOfShape::default();
        let mut me_inv_on_art = MapOfShape::default();

        for cb in &lcb_art {
            // check if aCB contains splits of only one offset face
            let mut mf_art = MapOfShape::default();
            for f in Explorer::new(cb, ShapeEnum::Face, ShapeEnum::Shape)? {
                mf_art.add(dmf_im_f.seek(&f).ok_or_else(not_found)?.clone());
            }

            let b_alone = mf_art.extent() == 1;

            // vertices on invalid edges
            let mut mve_inv = MapOfShape::default();
            let mut m_fence = MapOfShape::default();
            // edges that should not be marked as alone - edges having same origins as invalid ones
            let mut me_avoid = MapOfShape::default();
            // map to find alone edges by looking for free vertices
            let mut dmve_val = IndexedDataMapOfShapeListOfShape::default();

            for e in Explorer::new(cb, ShapeEnum::Edge, ShapeEnum::Shape)? {
                if self.my_invalid_edges.contains(&e) {
                    me_inv_on_art.add(e.clone());
                    for v in ShapeIterator::new(&e, true, true) {
                        mve_inv.add(v);
                    }

                    if b_alone && let Some(le_or) = self.my_oe_origins.seek(&e) {
                        for e_or in le_or.iter() {
                            for e_im in self.my_oe_images.seek(e_or).ok_or_else(not_found)?.iter() {
                                me_avoid.add(e_im.clone());
                            }
                        }
                    }
                    continue;
                }

                if m_fence.add(e.clone()) {
                    topexp::map_shapes_and_ancestors(
                        &e,
                        ShapeEnum::Vertex,
                        ShapeEnum::Edge,
                        &mut dmve_val,
                    )?;
                }
            }

            // find edges with free vertices
            for i in 1..=dmve_val.extent() {
                let v = dmve_val.find_key(i)?;
                if !mve_inv.contains(v) {
                    continue;
                }

                let lev = dmve_val.find_from_index(i)?;
                if lev.len() > 1 {
                    continue;
                }

                let e = lev.first().ok_or_else(not_found)?.clone();
                if me_avoid.contains(&e) {
                    continue;
                }

                me_alone.add(e.clone());

                // if this alone edge adds nothing to the intersection list
                // it means that the origin of this edge has been split and we need to
                // add the neighboring images of the same origins
                if maps.dmsf.seek(&e).ok_or_else(not_found)?.len() > 1 {
                    continue;
                }

                // check also its vertices
                let mut used = false;
                for ve in ShapeIterator::new(&e, true, true) {
                    if maps.dmsf.seek(&ve).ok_or_else(not_found)?.len() > 2 {
                        used = true;
                        break;
                    }
                }

                if used {
                    continue;
                }

                // the edge is useless - look for other images
                let Some(le_or) = self.my_oe_origins.seek(&e) else {
                    continue;
                };

                for e_or in le_or.iter() {
                    for e_im in self.my_oe_images.seek(e_or).ok_or_else(not_found)?.iter() {
                        if m_fence.contains(e_im) {
                            me_alone.add(e_im.clone());
                        }
                    }
                }
            }
        }

        // Get all invalidities from all faces to be used for avoiding
        // repeated usage of the common edges
        let mut m_all_invs = MapOfShape::default();
        for k in 1..=self.my_invalid_faces.extent() {
            for f in self.my_invalid_faces.find_from_index(k)?.iter() {
                for e in Explorer::new(f, ShapeEnum::Edge, ShapeEnum::Shape)? {
                    if self.my_invalid_edges.contains(&e) || me_alone.contains(&e) {
                        m_all_invs.add(e.clone());
                        for v in ShapeIterator::new(&e, true, true) {
                            m_all_invs.add(v);
                        }
                    }
                }
            }
        }

        // Bounding vertices of not trimmed edges
        let mut mv_bounds = MapOfShape::default();

        let mut me_check_ext = MapOfShape::default();
        // Save connections between not trimmed edge and its trimmed parts
        let mut dmee_trim = ShapeListMap::default();
        // Splits of the new edges
        let mut e_images = ShapeListMap::default();
        let bb = BRepBuilder::default();

        // Keep connection between blocks of invalid edges to the lists of
        // found edges to be intersected for its treatment
        let mut dmoen_edges = IndexedDataMapOfShapeListOfShape::default();

        for k in 1..=self.my_invalid_faces.extent() {
            let f_inv = self.my_invalid_faces.find_key(k)?.clone();
            let b_self_reb_avoid = self.my_f_self_reb_avoid.contains(&f_inv);
            let lf_inv = self.my_invalid_faces.find_from_index(k)?.clone();

            let lcb: Vec<Shape> = if lf_inv.len() > 1 {
                // make compound of invalid faces
                let mut cf_inv = Compound::default();
                bb.make_compound(&mut cf_inv);
                for f_im in lf_inv.iter() {
                    bb.add(&mut cf_inv, f_im)?;
                }

                // make connexity blocks
                let mut lcb = Vec::new();
                bop_tools::make_connexity_compounds(
                    &cf_inv,
                    ShapeEnum::Edge,
                    ShapeEnum::Face,
                    &mut lcb,
                )?;
                lcb
            } else {
                lf_inv.iter().cloned().collect()
            };

            let b_artificial = self.my_art_invalid_faces.is_bound(&f_inv);
            for cb_inv in &lcb {
                let mut me_fence = MapOfShape::default();

                let mut cbe = Compound::default();
                bb.make_compound(&mut cbe);

                // remember inside edges and vertices to further check
                let mut inside_edges = MapOfShape::default();
                let mut inside_vertices = MapOfShape::default();
                for e in Explorer::new(cb_inv, ShapeEnum::Edge, ShapeEnum::Shape)? {
                    if (self.my_invalid_edges.contains(&e)
                        || (b_artificial && me_alone.contains(&e)))
                        && me_fence.add(e.clone())
                    {
                        bb.add(&mut cbe, &e)?;
                        if !self.my_edges_to_avoid.contains(&e)
                            && self.my_invalid_edges.contains(&e)
                        {
                            inside_edges.add(e.clone());
                            for v in ShapeIterator::new(&e, true, true) {
                                inside_vertices.add(v);
                            }
                        }
                    }
                }

                // make connexity blocks of edges
                let mut lcbe = Vec::new();
                bop_tools::make_connexity_compounds(
                    &cbe,
                    ShapeEnum::Vertex,
                    ShapeEnum::Edge,
                    &mut lcbe,
                )?;

                for cbe_loc in &lcbe {
                    // map of edges and vertices of processing invalidity
                    let mut me = IndexedMapOfShape::default();
                    topexp::map_shapes_of_type(cbe_loc, ShapeEnum::Edge, &mut me)?;
                    // map of vertices to trim the new edges
                    let mut mecv = me.clone();
                    topexp::map_shapes_of_type(cbe_loc, ShapeEnum::Vertex, &mut me)?;
                    let mut mf_int = IndexedMapOfShape::default();
                    // additional faces for intersection
                    let mut mf_int_ext = IndexedMapOfShape::default();
                    // splits of faces for intersection
                    let mut lf_int: List<Shape> = List::default();
                    // faces to avoid intersection
                    let mut mf_avoid = IndexedMapOfShape::default();

                    self.find_faces_for_intersection(
                        &f_inv,
                        &me,
                        &maps.dmsf,
                        &mv_inv_all,
                        b_artificial,
                        &mut mf_avoid,
                        &mut mf_int,
                        &mut mf_int_ext,
                        &mut lf_int,
                    )?;
                    if mf_int.extent() < 3 {
                        // nothing to intersect
                        continue;
                    }

                    let p_mf_inter = self.my_intersection_pairs.seek(&f_inv).cloned();
                    // intersect the faces, but do not intersect the invalid ones
                    // among each other (except for the artificially invalid faces)
                    let mut me_to_int = IndexedMapOfShape::default();
                    let nb = mf_int.extent();
                    for i in 1..=nb {
                        let fi = mf_int.find_key(i)?.clone();
                        if b_self_reb_avoid && fi.is_same(&f_inv) {
                            continue;
                        }

                        let Some(lf_imi) = self.my_of_images.seek(&fi).cloned() else {
                            continue;
                        };

                        let idx_i = maps.fle.find_index(&fi);
                        if idx_i == 0 {
                            continue;
                        }

                        if !maps.m_done.is_bound(&fi) {
                            return Err(not_found());
                        }

                        let p_inter_fi = p_mf_inter.as_ref().and_then(|m| m.seek(&fi).cloned());
                        if p_mf_inter.is_some() && p_inter_fi.is_none() {
                            continue;
                        }

                        // create map of edges and vertices for aLFImi
                        let mut mev_im = MapOfShape::default();
                        map_shapes(&lf_imi, ShapeEnum::Edge, &mut mev_im)?;
                        map_shapes(&lf_imi, ShapeEnum::Vertex, &mut mev_im)?;

                        let is_i_contains_e = mev_im.has_intersection(&inside_edges);
                        let is_i_contains_v = mev_im.has_intersection(&inside_vertices);

                        for j in (i + 1)..=nb {
                            let fj = mf_int.find_key(j)?.clone();
                            if b_self_reb_avoid && fj.is_same(&f_inv) {
                                continue;
                            }

                            if let Some(inter_fi) = &p_inter_fi
                                && !inter_fi.contains(&fj)
                            {
                                continue;
                            }

                            let Some(lf_imj) = self.my_of_images.seek(&fj).cloned() else {
                                continue;
                            };

                            let idx_j = maps.fle.find_index(&fj);
                            if idx_j == 0 {
                                continue;
                            }

                            // create map of edges and vertices for aLFImi
                            mev_im.clear(true);
                            map_shapes(&lf_imj, ShapeEnum::Edge, &mut mev_im)?;
                            map_shapes(&lf_imj, ShapeEnum::Vertex, &mut mev_im)?;
                            // check images of both faces contain anInsideEdges and anInsideVertices
                            // not process if false and true
                            let is_j_contains_e = mev_im.has_intersection(&inside_edges);
                            let is_j_contains_v = mev_im.has_intersection(&inside_vertices);

                            // Check if one face is connected to inside edge then
                            // the other must be also connected
                            if (is_i_contains_e && !is_j_contains_v)
                                || (is_j_contains_e && !is_i_contains_v)
                            {
                                let mut lvc = List::default();
                                // it is necessary to process the images if they already have
                                // common vertices
                                find_common_parts(&lf_imi, &lf_imj, &mut lvc, ShapeEnum::Vertex)?;

                                if lvc.is_empty() {
                                    continue;
                                }
                            }

                            // if there are some common edges between faces
                            // we should use these edges and do not intersect again.
                            let mut lec = List::default();
                            find_common_parts(&lf_imi, &lf_imj, &mut lec, ShapeEnum::Edge)?;

                            let mut lfei = std::mem::take(maps.fle.change_from_index(idx_i)?);
                            let mut lfej = std::mem::take(maps.fle.change_from_index(idx_j)?);
                            let r = self.intersect_pair(
                                &f_inv,
                                &fi,
                                &fj,
                                &lf_inv,
                                &lf_imi,
                                &lf_imj,
                                &lec,
                                &mut lfei,
                                &mut lfej,
                                &me,
                                &m_all_invs,
                                &mf_int_ext,
                                &mf_avoid,
                                &mut maps,
                                &mut mecv,
                                &mut me_check_ext,
                                &mut dmee_trim,
                                &mut me_to_int,
                            );
                            *maps.fle.change_from_index(idx_i)? = lfei;
                            *maps.fle.change_from_index(idx_j)? = lfej;
                            r?;
                        }
                    }

                    // intersect and trim edges for this chain
                    self.intersect_and_trim_edges(
                        &mf_int,
                        &me_to_int,
                        &dmee_trim,
                        &me,
                        &mecv,
                        &mv_inv,
                        &mvr_inv,
                        &me_check_ext,
                        b_artificial,
                        &mut mv_bounds,
                        &mut e_images,
                    )?;

                    for i_e in 1..=me_to_int.extent() {
                        let e_int = me_to_int.find_key(i_e)?;
                        for e in Explorer::new(cbe_loc, ShapeEnum::Edge, ShapeEnum::Shape)? {
                            append_indexed(&mut dmoen_edges, &e, e_int)?;
                        }
                    }
                }
            }
        }

        // filter the obtained edges
        self.update_valid_edges(
            &maps.fle,
            &dmoen_edges,
            &mv_bounds,
            &me_inv_on_art,
            &mut me_check_ext,
            verts_to_avoid,
            &mut e_images,
            &mut dmee_trim,
        )
    }

    /// The treatment of one pair of faces of the intersection loop of `IntersectFaces`: the
    /// common edges used, or the faces intersected (once), or the earlier intersection reused.
    #[allow(clippy::too_many_arguments)]
    fn intersect_pair(
        &mut self,
        f_inv: &Shape,
        fi: &Shape,
        fj: &Shape,
        lf_inv: &List<Shape>,
        lf_imi: &List<Shape>,
        lf_imj: &List<Shape>,
        lec: &List<Shape>,
        lfei: &mut List<Shape>,
        lfej: &mut List<Shape>,
        me: &IndexedMapOfShape,
        m_all_invs: &MapOfShape,
        mf_int_ext: &IndexedMapOfShape,
        mf_avoid: &IndexedMapOfShape,
        maps: &mut IntersectionMaps,
        mecv: &mut IndexedMapOfShape,
        me_check_ext: &mut MapOfShape,
        dmee_trim: &mut ShapeListMap,
        me_to_int: &mut IndexedMapOfShape,
    ) -> Result<(), Failure> {
        if !lec.is_empty() {
            // no need to intersect if we have common edges between faces
            let b_force_use = mf_int_ext.contains(fi) || mf_int_ext.contains(fj);
            self.process_common_edges(
                lec,
                me,
                &maps.me_inf_e_trim,
                m_all_invs,
                b_force_use,
                mecv,
                me_check_ext,
                dmee_trim,
                lfei,
                lfej,
                me_to_int,
            )?;

            // Add common vertices not belonging to the common edges for trimming the intersection
            // edges
            let mut mv_on_ce = IndexedMapOfShape::default();
            for e in lec.iter() {
                topexp::map_shapes_of_type(e, ShapeEnum::Vertex, &mut mv_on_ce)?;
            }

            let mut lev = List::default();
            find_common_parts(lf_imi, lf_imj, &mut lev, ShapeEnum::Vertex)?;
            for v in lev.iter() {
                if !mv_on_ce.contains(v) {
                    mecv.add(v.clone());
                }
            }
            return Ok(());
        }

        // check if both these faces are invalid and sharing edges
        if self.my_invalid_faces.contains(fi)
            && self.my_invalid_faces.contains(fj)
            && !self.my_art_invalid_faces.is_bound(fi)
            && !self.my_art_invalid_faces.is_bound(fj)
        {
            return Ok(());
        }

        // check if these two faces have already been treated
        let done = maps
            .m_done
            .seek(fi)
            .ok_or_else(not_found)?
            .iter()
            .any(|f| f.is_same(fj));
        if done {
            // use intersection line obtained on the previous steps
            // plus, find new origins for these lines
            return self.update_intersected_faces(
                f_inv, fi, fj, lf_inv, lf_imi, lf_imj, lfei, lfej, me_to_int,
            );
        }

        if mf_avoid.contains(fi) || mf_avoid.contains(fj) {
            return Ok(());
        }

        maps.m_done
            .change_seek(fi)
            .ok_or_else(not_found)?
            .append(fj.clone());
        maps.m_done
            .change_seek(fj)
            .ok_or_else(not_found)?
            .append(fi.clone());

        self.intersect_faces_pair(
            f_inv, fi, fj, lf_inv, lf_imi, lf_imj, lfei, lfej, mecv, me_to_int,
        )
    }

    /// `PrepareFacesForIntersection(...)`: the faces to rebuild with empty lists of
    /// intersection edges, and the connections of their edges and vertices to the faces, of
    /// the trimmed edges to the untrimmed ones and of the edges of the invalid splits.
    fn prepare_faces_for_intersection(
        &mut self,
        look_vert_to_avoid: bool,
        maps: &mut IntersectionMaps,
    ) -> Result<(), Failure> {
        for i in 1..=self.my_faces_to_rebuild.extent() {
            let f = self.my_faces_to_rebuild.find_key(i)?.clone();

            maps.fle.add(f.clone(), List::default());
            maps.m_done.bind(f.clone(), List::default());

            let lf_im = self.my_of_images.find_from_key(&f)?.clone();
            for f_im in lf_im.iter() {
                for e in Explorer::new(f_im, ShapeEnum::Edge, ShapeEnum::Shape)? {
                    // save connection to untrimmed face
                    append_bound(&mut maps.dmsf, &e, &f);

                    // save connection to untrimmed edge
                    let e_inf = self
                        .e_trim_e_inf_ref()?
                        .seek(&e)
                        .ok_or_else(not_found)?
                        .clone();
                    append_bound(&mut maps.me_inf_e_trim, &e_inf, &e);

                    for v in Explorer::new(&e, ShapeEnum::Vertex, ShapeEnum::Shape)? {
                        // save connection to face
                        append_bound(&mut maps.dmsf, &v, &f);

                        if look_vert_to_avoid {
                            // save connection to edges
                            append_bound(&mut maps.dmve_full, &v, &e);
                        }
                    }
                }
            }

            if look_vert_to_avoid {
                // get edges of invalid faces (from invalid splits only)
                let Some(lf_inv) = self.my_invalid_faces.seek(&f) else {
                    continue;
                };
                if self.my_art_invalid_faces.is_bound(&f) {
                    continue;
                }

                for f_inv in lf_inv.iter() {
                    for e in Explorer::new(f_inv, ShapeEnum::Edge, ShapeEnum::Shape)? {
                        append_indexed(&mut maps.dmef_inv, &e, &f)?;
                    }
                }
            }
        }
        Ok(())
    }

    /// `FindVerticesToAvoid(theDMEFInv, theDMVEFull, theMVRInv)`: the free vertices of the
    /// invalid edges shared by several invalid faces whose edges are all invalid (or two of
    /// them inverted).
    fn find_vertices_to_avoid(
        &self,
        dmef_inv: &IndexedDataMapOfShapeListOfShape,
        dmve_full: &ShapeListMap,
        mvr_inv: &mut MapOfShape,
    ) -> Result<(), Failure> {
        let mut m_fence = MapOfShape::default();
        for i in 1..=dmef_inv.extent() {
            if dmef_inv.find_from_index(i)?.len() == 1 {
                continue;
            }

            let e = dmef_inv.find_key(i)?;
            if !self.my_invalid_edges.contains(e) || self.my_valid_edges.contains(e) {
                continue;
            }

            if !m_fence.add(e.clone()) {
                continue;
            }

            let mut mve_edges = IndexedDataMapOfShapeListOfShape::default();
            // Do not check the splitting vertices, but check only the ending ones
            if let Some(le_or) = self.my_oe_origins.seek(e) {
                for e_or in le_or.iter() {
                    for e_im in self.my_oe_images.seek(e_or).ok_or_else(not_found)?.iter() {
                        m_fence.add(e_im.clone());
                        topexp::map_shapes_and_ancestors(
                            e_im,
                            ShapeEnum::Vertex,
                            ShapeEnum::Edge,
                            &mut mve_edges,
                        )?;
                    }
                }
            } else {
                topexp::map_shapes_and_ancestors(
                    e,
                    ShapeEnum::Vertex,
                    ShapeEnum::Edge,
                    &mut mve_edges,
                )?;
            }

            for j in 1..=mve_edges.extent() {
                if mve_edges.find_from_index(j)?.len() != 1 {
                    continue;
                }

                let v = mve_edges.find_key(j)?;
                if !m_fence.add(v.clone()) {
                    continue;
                }
                let Some(le) = dmve_full.seek(v) else {
                    // isolated vertex
                    mvr_inv.add(v.clone());
                    continue;
                };

                // If all edges sharing the vertex are either invalid or
                // the vertex is connected to at least two inverted edges
                // mark the vertex to be avoided in the new splits
                let mut nb_e_inverted = 0;
                let mut b_all_edges_inv = true;
                for ev in le.iter() {
                    if self.my_inverted_edges.contains(ev) {
                        nb_e_inverted += 1;
                    }
                    if b_all_edges_inv {
                        b_all_edges_inv = self.my_invalid_edges.contains(ev);
                    }
                }

                if nb_e_inverted > 1 || b_all_edges_inv {
                    mvr_inv.add(v.clone());
                }
            }
        }
        Ok(())
    }

    /// `FindFacesForIntersection(...)`: the faces around the invalidity (connected to its
    /// edges and vertices), plus the isolated faces interfering with the invalid face.
    #[allow(clippy::too_many_arguments)]
    fn find_faces_for_intersection(
        &self,
        f_inv: &Shape,
        me: &IndexedMapOfShape,
        dmsf: &ShapeListMap,
        mv_inv_all: &MapOfShape,
        art_case: bool,
        mf_avoid: &mut IndexedMapOfShape,
        mf_int: &mut IndexedMapOfShape,
        mf_int_ext: &mut IndexedMapOfShape,
        lf_im_int: &mut List<Shape>,
    ) -> Result<(), Failure> {
        let mut m_shapes = IndexedMapOfShape::default();

        for i in 1..=me.extent() {
            let s = me.find_key(i)?;
            let Some(lf) = dmsf.seek(s) else {
                continue;
            };

            // in artificial case we intersect the faces which are close to invalidity
            let b_avoid =
                art_case && s.shape_type()? == ShapeEnum::Vertex && !mv_inv_all.contains(s);

            for f in lf.iter() {
                if mf_int.contains(f) {
                    continue;
                }

                if b_avoid && self.my_art_invalid_faces.is_bound(f) {
                    mf_avoid.add(f.clone());
                }

                mf_int.add(f.clone());

                let b_use = !f.is_same(f_inv);

                for f_im in self.my_of_images.find_from_key(f)?.iter() {
                    lf_im_int.append(f_im.clone());
                    if b_use {
                        topexp::map_shapes_of_type(f_im, ShapeEnum::Edge, &mut m_shapes)?;
                    }
                }
            }
        }

        let ss_interfs_map = if art_case {
            &self.my_ss_interfs_art
        } else {
            &self.my_ss_interfs
        };
        let Some(lf_inv) = ss_interfs_map.seek(f_inv) else {
            return Ok(());
        };

        let mut mf = MapOfShape::default();
        for f in lf_inv.iter() {
            mf.add(f.clone());
        }

        // the faces should be unique in each place
        let mut cf = Compound::default();
        BRepBuilder::default().make_compound(&mut cf);

        let mut mf_to_add = IndexedMapOfShape::default();
        let mut dmf_or = ShapeShapeMap::default();

        for i in 1..=me.extent() {
            let s = me.find_key(i)?;
            let Some(lf) = ss_interfs_map.seek(s) else {
                continue;
            };

            for f in lf.iter() {
                if mf_int.contains(f) || mf_to_add.contains(f) || !mf.contains(f) {
                    continue;
                }

                // check if the face has some connection to already added for intersection faces
                let lf_im = self.my_of_images.find_from_key(f)?;
                if !art_case {
                    let mut connected = false;
                    'splits: for f_im in lf_im.iter() {
                        for e in Explorer::new(f_im, ShapeEnum::Edge, ShapeEnum::Shape)? {
                            if m_shapes.contains(&e) {
                                connected = true;
                                break 'splits;
                            }
                        }
                    }
                    if !connected {
                        continue;
                    }
                }

                mf_to_add.add(f.clone());
                for f_im in lf_im.iter() {
                    dmf_or.bind(f_im.clone(), f.clone());
                    BRepBuilder::default().add(&mut cf, f_im)?;
                }
            }
        }

        if mf_to_add.is_empty() {
            return Ok(());
        }

        let mut lcb = Vec::new();
        bop_tools::make_connexity_compounds(&cf, ShapeEnum::Edge, ShapeEnum::Face, &mut lcb)?;

        if lcb.len() == 1 && mf_to_add.extent() > 1 {
            return Ok(());
        }

        for cb in &lcb {
            mf_to_add.clear(true);
            for f_im in Explorer::new(cb, ShapeEnum::Face, ShapeEnum::Shape)? {
                mf_to_add.add(dmf_or.seek(&f_im).ok_or_else(not_found)?.clone());
            }

            if mf_to_add.extent() == 1 {
                let f = mf_to_add.find_key(1)?.clone();

                mf_int.add(f.clone());
                mf_int_ext.add(f.clone());

                for f_im in self.my_of_images.find_from_key(&f)?.iter() {
                    lf_im_int.append(f_im.clone());
                }
            }
        }
        Ok(())
    }

    /// `ProcessCommonEdges(...)`: the untrimmed edges of the valid common edges of two faces
    /// (connected to the invalidity) used as their intersection edges.
    #[allow(clippy::too_many_arguments)]
    fn process_common_edges(
        &mut self,
        lec_in: &List<Shape>,
        me: &IndexedMapOfShape,
        me_inf_e_trim: &ShapeListMap,
        all_invs: &MapOfShape,
        force_use: bool,
        mecv: &mut IndexedMapOfShape,
        me_check_ext: &mut MapOfShape,
        dmee_trim: &mut ShapeListMap,
        lfei: &mut List<Shape>,
        lfej: &mut List<Shape>,
        me_to_int: &mut IndexedMapOfShape,
    ) -> Result<(), Failure> {
        let mut lec: List<Shape> = List::default();
        // process common edges
        for ec in lec_in.iter() {
            // check first if common edges are valid
            if self.my_invalid_edges.contains(ec) && !self.my_valid_edges.contains(ec) {
                continue;
            }

            // common edge should have connection to current invalidity
            if me.contains(ec) {
                lec.append(ec.clone());
                continue;
            }

            for ve in ShapeIterator::new(ec, true, true) {
                if me.contains(&ve) {
                    lec.append(ec.clone());
                    break;
                }
            }
        }

        let b_use_only_inf = lec.is_empty();
        if b_use_only_inf {
            if force_use {
                lec = lec_in.clone();
            } else {
                for ec in lec_in.iter() {
                    // check if all images of the origin of this edge
                    // are not connected to any invalidity
                    let e_int = self.e_trim_e_inf_ref()?.seek(ec).ok_or_else(not_found)?;
                    let lve = me_inf_e_trim.seek(e_int).ok_or_else(not_found)?;
                    for ecx in lve.iter() {
                        if all_invs.contains(ecx) || self.my_invalid_edges.contains(ecx) {
                            return Ok(());
                        }

                        for v in ShapeIterator::new(ecx, true, true) {
                            if all_invs.contains(&v) {
                                return Ok(());
                            }
                        }
                    }
                    // use only one element
                    if lec.is_empty() {
                        lec.append(ec.clone());
                    }
                }
            }
        }

        for ec in lec.iter() {
            let e_int = self
                .e_trim_e_inf_ref()?
                .seek(ec)
                .ok_or_else(not_found)?
                .clone();
            if !b_use_only_inf {
                // find the edges of the same original edge
                // and take their vertices as well
                let lve = me_inf_e_trim.seek(&e_int).ok_or_else(not_found)?;
                for ecx in lve.iter() {
                    let le_or = self.my_oe_origins.seek(ecx);
                    if le_or.is_none_or(|l| l.len() == 1) {
                        topexp::map_shapes_of_type(ecx, ShapeEnum::Vertex, mecv)?;
                    }
                }

                // bind unlimited edge to its trimmed part in face to update maps of
                // images and origins in the future
                append_bound(dmee_trim, &e_int, ec);
            } else if !force_use {
                me_check_ext.add(e_int.clone());
            }

            append_to_list(lfei, &e_int);
            append_to_list(lfej, &e_int);
            me_to_int.add(e_int);
        }
        Ok(())
    }

    /// `UpdateIntersectedFaces(...)`: the intersection edges common to the two faces (from
    /// an earlier intersection) reused, with the new origins.
    #[allow(clippy::too_many_arguments)]
    fn update_intersected_faces(
        &mut self,
        f_inv: &Shape,
        fi: &Shape,
        fj: &Shape,
        lf_inv: &List<Shape>,
        lf_imi: &List<Shape>,
        lf_imj: &List<Shape>,
        lfei: &List<Shape>,
        lfej: &List<Shape>,
        me_to_int: &mut IndexedMapOfShape,
    ) -> Result<(), Failure> {
        // Find common edges in these two lists
        let mut mei = MapOfShape::default();
        for e in lfei.iter() {
            mei.add(e.clone());
        }

        // find origins
        let mut me_to_find_origins = IndexedMapOfShape::default();
        let mut le_to_find_origins = List::default();
        if !fi.is_same(f_inv) {
            find_common_parts(lf_imi, lf_inv, &mut le_to_find_origins, ShapeEnum::Edge)?;
        }
        if !fj.is_same(f_inv) {
            find_common_parts(lf_imj, lf_inv, &mut le_to_find_origins, ShapeEnum::Edge)?;
        }

        let mut le_or_init = List::default();
        for ec in le_to_find_origins.iter() {
            me_to_find_origins.add(ec.clone());
        }

        find_origins(
            lf_imi,
            lf_imj,
            &me_to_find_origins,
            self.edges_origins(),
            &mut le_or_init,
        )?;

        for e in lfej.iter() {
            if mei.contains(e) {
                me_to_int.add(e.clone());
                if !le_or_init.is_empty() {
                    let origins = self.edges_origins();
                    if let Some(le_or) = origins.change_seek(e) {
                        for e_or in le_or_init.iter() {
                            append_to_list(le_or, e_or);
                        }
                    } else {
                        origins.bind(e.clone(), le_or_init.clone());
                    }
                }
            }
        }
        Ok(())
    }

    /// `IntersectFaces(theFInv, theFi, theFj, ...)`: the intersection edges of the pair of
    /// faces added to both, with the common vertices of their splits for trimming and the
    /// origins of the edges.
    #[allow(clippy::too_many_arguments)]
    fn intersect_faces_pair(
        &mut self,
        f_inv: &Shape,
        fi: &Shape,
        fj: &Shape,
        lf_inv: &List<Shape>,
        lf_imi: &List<Shape>,
        lf_imj: &List<Shape>,
        lfei: &mut List<Shape>,
        lfej: &mut List<Shape>,
        mecv: &mut IndexedMapOfShape,
        me_to_int: &mut IndexedMapOfShape,
    ) -> Result<(), Failure> {
        // intersect faces
        let mut l_int1 = List::default();
        let mut l_int2 = List::default();
        BRepOffsetTool::inter3d(
            &topods::face(fi)?,
            &topods::face(fj)?,
            &mut l_int1,
            &mut l_int2,
            State::Out,
            &Edge::default(),
            &Face::default(),
            &Face::default(),
        )?;

        if l_int1.is_empty() {
            return Ok(());
        }

        // find common vertices for trimming edges
        let mut lcv = List::default();
        find_common_parts(lf_imi, lf_imj, &mut lcv, ShapeEnum::Vertex)?;
        if lcv.len() > 1 {
            for cv in lcv.iter() {
                mecv.add(cv.clone());
            }
        }

        // find origins
        let mut me_to_find_origins = IndexedMapOfShape::default();
        let mut le_to_find_origins = List::default();
        if !fi.is_same(f_inv) {
            find_common_parts(lf_imi, lf_inv, &mut le_to_find_origins, ShapeEnum::Edge)?;
        }
        if !fj.is_same(f_inv) {
            find_common_parts(lf_imj, lf_inv, &mut le_to_find_origins, ShapeEnum::Edge)?;
        }
        let mut le_or_init = List::default();
        for ec in le_to_find_origins.iter() {
            me_to_find_origins.add(ec.clone());
        }

        find_origins(
            lf_imi,
            lf_imj,
            &me_to_find_origins,
            self.edges_origins(),
            &mut le_or_init,
        )?;

        for e_int in l_int1.iter() {
            lfei.append(e_int.clone());
            lfej.append(e_int.clone());

            if !le_or_init.is_empty() {
                self.edges_origins().bind(e_int.clone(), le_or_init.clone());
            }

            me_to_int.add(e_int.clone());
        }
        Ok(())
    }
}
