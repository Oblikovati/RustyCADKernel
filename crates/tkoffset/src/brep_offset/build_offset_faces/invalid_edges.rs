// Rust translation of the two BRepOffset_BuildOffsetFaces::FindInvalidEdges (with the
// static addAsNeutral) and MakeInvertedEdgesInvalid of
// ModelingAlgorithms/TKOffset/BRepOffset/BRepOffset_MakeOffset_1.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The invalid edges of the splits: edges whose orientation or vertex order changed with
//! respect to their origins, the neutral edges of same-domain splits, and the inverted
//! edges locked inside loops of invalid edges.
use super::{
    BuildOffsetFaces, ShapeIndexedMapMap, ShapeListMap, ShapeMapMap, ShapeShapeMap, append_to_list,
    find_shape, get_average_tangent, nb_points,
};
use rustycad_tkbo::bop_tools::{self, ShapeSet, algo_tools3d};
use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::topexp::{self, Explorer};
use rustycad_tkbrep::topods::{self, Compound, Face, Shape, ShapeIterator};
use rustycad_tkbrep::toptools::{IndexedDataMapOfShapeListOfShape, IndexedMapOfShape, MapOfShape};
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::{self, Orientation, ShapeEnum};
use rustycad_tkmath::gp::Direction3;

/// The list bound to a key in an indexed data map, appended to (created when absent).
fn indexed_list_append(
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

/// `addAsNeutral(theE, theFInv, theFVal, theLocInvEdges, theLocValidEdges)`: the edge
/// invalid in one split and valid in the other.
fn add_as_neutral(
    e: &Shape,
    f_inv: &Shape,
    f_val: &Shape,
    loc_inv_edges: &mut ShapeIndexedMapMap,
    loc_valid_edges: &mut ShapeMapMap,
) {
    if !loc_inv_edges.is_bound(f_inv) {
        loc_inv_edges.bind(f_inv.clone(), IndexedMapOfShape::default());
    }
    if let Some(m) = loc_inv_edges.change_seek(f_inv) {
        m.add(e.clone());
    }
    if !loc_valid_edges.is_bound(f_val) {
        loc_valid_edges.bind(f_val.clone(), MapOfShape::default());
    }
    if let Some(m) = loc_valid_edges.change_seek(f_val) {
        m.add(e.clone());
    }
}

/// The normal of the face along the edge (`BOPTools_AlgoTools3D::GetNormalToFaceOnEdge`).
fn normal_on_edge(e: &Shape, f: &Shape) -> Result<Direction3, Failure> {
    let mut d = Direction3::default();
    algo_tools3d::get_normal_to_face_on_edge(&topods::edge(e)?, &topods::face(f)?, &mut d, None)?;
    Ok(d)
}

impl BuildOffsetFaces<'_> {
    /// `FindInvalidEdges(theF, theLFImages, ...)`: the edges of the splits of the face
    /// classified valid or invalid by comparing their average tangents with their origins
    /// (an edge born from a vertex is invalid when the original edges at the vertex no
    /// longer share it), the inverted ones checked; the neutral and inverted-valid edges
    /// of the face recorded.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn find_invalid_edges(
        &mut self,
        f: &Face,
        lf_images: &List<Shape>,
        dmfmve: &mut ShapeMapMap,
        dmfmne: &mut ShapeMapMap,
        dmfmie: &mut ShapeIndexedMapMap,
        dmfmvie: &mut ShapeMapMap,
        dme_or_le_im: &mut ShapeListMap,
        edges_invalid_by_vertex: &mut MapOfShape,
        edges_valid_by_vertex: &mut MapOfShape,
    ) -> Result<(), Failure> {
        // Edge is considered as invalid in the following cases:
        // 1. Its orientation on the face has changed comparing to the originals edge and face;
        // 2. The vertices of the edge have changed places comparing to the originals edge and face.
        //
        // The edges created from vertices, i.e. as intersection between two faces connected only
        // by VERTEX, will also be checked on validity. For these edges the correct orientation will
        // be defined by the edges on the original face adjacent to the connection vertex

        // original face
        let f_or = topods::face(self.faces_origins_ref()?.seek(f).ok_or_else(|| {
            Failure::with_message("Standard_NoSuchObject", "NCollection_DataMap::Find")
        })?)?;
        let analyzer = self.my_analyzer;
        // invalid edges
        let mut me_inv = IndexedMapOfShape::default();
        // valid edges
        let mut me_val = MapOfShape::default();
        // internal edges
        let mut me_int = MapOfShape::default();
        // maps for checking the inverted edges
        let mut dmve = IndexedDataMapOfShapeListOfShape::default();
        let mut dmef = IndexedDataMapOfShapeListOfShape::default();
        let mut m_edges = IndexedMapOfShape::default();
        // back map from the original shapes to their offset images
        let mut an_images = ShapeListMap::default();

        for f_im in lf_images.iter() {
            for e in Explorer::new(f_im, ShapeEnum::Edge, ShapeEnum::Shape)? {
                // keep all edges
                m_edges.add(e.clone());
                // keep connection from edges to faces
                indexed_list_append(&mut dmef, &e, f_im)?;
                // keep connection from vertices to edges
                for v in ShapeIterator::new(&e, true, true) {
                    indexed_list_append(&mut dmve, &v, &e)?;
                }
                // back map from original edges to their offset images
                let Some(l_or) = self.edges_origins().seek(&e).cloned() else {
                    continue;
                };
                for s_or in l_or.iter() {
                    let Some(s_in_f) = find_shape(s_or, &f_or, analyzer)? else {
                        continue;
                    };
                    if !an_images.is_bound(&s_in_f) {
                        an_images.bind(s_in_f.clone(), List::default());
                    }
                    if let Some(images) = an_images.change_seek(&s_in_f) {
                        append_to_list(images, &e);
                    }
                }
            }
        }

        // the map will be used to find the edges on the original face
        // adjacent to the same vertex. It will be filled at first necessity;
        let mut dmve_f_or = IndexedDataMapOfShapeListOfShape::default();

        for f_im in lf_images.iter() {
            // valid edges for this split
            let mut mve = MapOfShape::default();
            // invalid edges for this split
            let mut mie = IndexedMapOfShape::default();

            for e_im in Explorer::new(f_im, ShapeEnum::Edge, ShapeEnum::Shape)? {
                let e_im = topods::edge(&e_im)?;
                if e_im.orientation() == Orientation::Internal {
                    me_int.add(e_im.into_shape());
                    continue;
                }

                let Some(le_or) = self.edges_origins().seek(&e_im).cloned() else {
                    continue;
                };
                if le_or.is_empty() {
                    continue;
                }

                let nb_v_or = le_or
                    .iter()
                    .filter(|s| s.shape_type().ok() == Some(ShapeEnum::Vertex))
                    .count();
                let nb_le_or = le_or.len();
                if nb_v_or > 1 && (nb_le_or - nb_v_or) > 1 {
                    continue;
                }

                let mut me = MapOfShape::default();
                let mut mv = MapOfShape::default();
                let mut mf = MapOfShape::default();
                let (mut b_invalid, mut b_checked) = (false, false);
                let nb_p = nb_points(&e_im)?;
                let mut nb_inv = 0;
                let b_use_vertex = nb_v_or != 0
                    && (nb_v_or == 1
                        && dmef.find_from_key(&e_im)?.len() == 1
                        && !self.my_oe_origins.is_bound(&e_im));

                for s_or in le_or.iter() {
                    let b_vertex = s_or.shape_type()? == ShapeEnum::Vertex;

                    let mut e_or_f = Shape::default();
                    if b_vertex {
                        // for some cases it is impossible to check the validity of the edge
                        if !b_use_vertex {
                            continue;
                        }
                        // find edges on the original face adjacent to this vertex
                        if dmve_f_or.is_empty() {
                            // fill the map
                            topexp::map_shapes_and_ancestors(
                                &f_or,
                                ShapeEnum::Vertex,
                                ShapeEnum::Edge,
                                &mut dmve_f_or,
                            )?;
                        }

                        if dmve_f_or.contains(s_or) {
                            let le_f_or: Vec<Shape> =
                                dmve_f_or.find_from_key(s_or)?.iter().cloned().collect();
                            let mut ce_or = Compound::default();
                            BRepBuilder::default().make_compound(&mut ce_or);
                            // Avoid classification of edges originated from vertices
                            // located between tangent edges
                            let mut b_all_tgt = true;
                            let v_ref =
                                get_average_tangent(le_f_or.first().ok_or_else(empty)?, nb_p)?;
                            for e_or in &le_f_or {
                                BRepBuilder::default().add(&mut ce_or, e_or)?;
                                let v_cur = get_average_tangent(e_or, nb_p)?;
                                if !v_ref.is_parallel(v_cur, precision::angular())? {
                                    b_all_tgt = false;
                                }
                            }
                            if !b_all_tgt {
                                e_or_f = ce_or.into_shape();
                            }
                        }
                    } else {
                        if let Some(found) = find_shape(s_or, &f_or, analyzer)? {
                            e_or_f = found;
                        }
                        if !dme_or_le_im.is_bound(s_or) {
                            dme_or_le_im.bind(s_or.clone(), List::default());
                        }
                        if let Some(le_im) = dme_or_le_im.change_seek(s_or) {
                            append_to_list(le_im, &e_im);
                        }
                    }

                    if e_or_f.is_null() {
                        // the edge has not been found
                        continue;
                    }

                    if b_vertex {
                        let mut mv_total = MapOfShape::default();
                        let mut nb_checked = 0;
                        // Just check if the original edges sharing the vertex do not share it any more.
                        for e_or in ShapeIterator::new(&e_or_f, true, true) {
                            let Some(l_im) = an_images.seek(&e_or) else {
                                continue;
                            };
                            nb_checked += 1;
                            let mut mv_loc = IndexedDataMapOfShapeListOfShape::default();
                            for x in l_im.iter() {
                                topexp::map_shapes_and_ancestors(
                                    x,
                                    ShapeEnum::Vertex,
                                    ShapeEnum::Edge,
                                    &mut mv_loc,
                                )?;
                            }
                            for i in 1..=mv_loc.extent() {
                                if mv_loc.find_from_index(i)?.len() > 1
                                    && !mv_total.add(mv_loc.find_key(i)?.clone())
                                {
                                    b_invalid = true;
                                    edges_invalid_by_vertex.add(e_im.clone().into_shape());
                                    break;
                                }
                            }
                            if b_invalid {
                                break;
                            }
                        }
                        if !b_invalid && nb_checked < 2 {
                            continue;
                        } else {
                            edges_valid_by_vertex.add(e_im.clone().into_shape());
                        }
                    } else {
                        // Check orientations of the image edge and original edge.
                        // In case the 3d curves are having the same direction the orientations
                        // must be the same. Otherwise the orientations should also be different.
                        //
                        // get average tangent vector for each curve taking into account
                        // the orientations of the edges, i.e. the edge is reversed
                        // the vector is reversed as well
                        let mut v_sum1 = get_average_tangent(&e_im, nb_p)?;
                        let mut v_sum2 = get_average_tangent(&e_or_f, nb_p)?;
                        v_sum1.normalize()?;
                        v_sum2.normalize()?;

                        let cos = v_sum1.dot(v_sum2);
                        if cos.abs() < 0.9999 {
                            continue;
                        }

                        me.add(e_or_f.clone());
                        for v in Explorer::new(&e_or_f, ShapeEnum::Vertex, ShapeEnum::Shape)? {
                            mv.add(v);
                        }
                        if let Some(analyzer) = analyzer {
                            for fa in analyzer.ancestors(&e_or_f)? {
                                mf.add(fa.clone());
                            }
                        }

                        if cos < precision::confusion() {
                            b_invalid = true;
                            nb_inv += 1;
                        }
                    }
                    b_checked = true;
                }

                if !b_checked {
                    continue;
                }

                let mut b_local_only = nb_v_or > 1 && (nb_le_or - nb_v_or) > 1;
                let (nb_e, nb_v) = (me.extent(), mv.extent());
                if nb_e > 1 && nb_v == 2 * nb_e {
                    let mut b_skip = true;

                    // It seems the edge originated from not connected edges and cannot be
                    // considered as correctly classified as it may fill some undesired parts.
                    // Still, allow the edge to be accounted for local analysis if it is:
                    // * originated from more than two faces
                    // * unanimously considered valid or invalid
                    // * not a boundary edge in the splits
                    if mf.extent() > 2 && (nb_inv == 0 || nb_inv == nb_e) && lf_images.len() > 2 {
                        let mut v_exhausted = true;
                        'vertices: for v in ShapeIterator::new(&e_im, true, true) {
                            for e in dmve.find_from_key(&v)?.iter() {
                                if dmef.find_from_key(e)?.len() < 2 {
                                    v_exhausted = false;
                                    break 'vertices;
                                }
                            }
                        }
                        b_skip = !v_exhausted;
                    }
                    if b_skip {
                        continue;
                    } else {
                        b_local_only = true;
                    }
                }

                if b_invalid {
                    if !b_local_only {
                        self.my_invalid_edges.add(e_im.clone().into_shape());
                    }
                    mie.add(e_im.clone().into_shape());
                    me_inv.add(e_im.into_shape());
                    continue;
                }

                // check if the edge has been inverted
                let b_inverted = if nb_e == 0 || b_local_only {
                    false
                } else {
                    self.check_inverted(&e_im, &f_or, &dmve, &m_edges)?
                };

                if !b_inverted || nb_v_or == 0 {
                    if !b_local_only {
                        self.my_valid_edges.add(e_im.clone().into_shape());
                    }
                    mve.add(e_im.clone().into_shape());
                    me_val.add(e_im.into_shape());
                }
            }

            // valid edges
            if mve.extent() > 0 {
                dmfmve.bind(f_im.clone(), mve);
            }
            // invalid edges
            if mie.extent() > 0 {
                dmfmie.bind(f_im.clone(), mie);
            }
        }

        // process invalid edges:
        // check for the inverted edges
        let mut mvie = MapOfShape::default();
        // fill neutral edges
        let mut mne = MapOfShape::default();

        for i in 1..=me_inv.extent() {
            let e_im = me_inv.find_key(i)?.clone();
            // neutral edges - on the splits of the same offset face
            // it is valid for one split and invalid for other
            if me_val.contains(&e_im) {
                mne.add(e_im);
                continue;
            }
            // the inverted images of the origins of invalid edges should also be invalid
            if !self.my_inverted_edges.contains(&e_im) {
                continue;
            }
            let Some(loe_or) = self.my_oe_origins.seek(&e_im).cloned() else {
                continue;
            };
            for oe_or in loe_or.iter() {
                let le_im1 = self.my_oe_images.seek(oe_or).cloned().ok_or_else(|| {
                    Failure::with_message("Standard_NoSuchObject", "NCollection_DataMap::Find")
                })?;
                for e_im1 in le_im1.iter() {
                    if m_edges.contains(e_im1)
                        && !me_inv.contains(e_im1)
                        && !me_int.contains(e_im1)
                        && self.my_inverted_edges.contains(e_im1)
                    {
                        self.my_invalid_edges.add(e_im1.clone());
                        mvie.add(e_im1.clone());
                    }
                }
            }
        }

        if mne.extent() > 0 {
            dmfmne.bind(f.clone().into_shape(), mne);
        }
        if mvie.extent() > 0 {
            dmfmvie.bind(f.clone().into_shape(), mvie);
        }
        Ok(())
    }

    /// `FindInvalidEdges(theLFOffset, theLocInvEdges, theLocValidEdges, theNeutralEdges)`:
    /// the edges classified in some splits but not in others of same-domain faces made
    /// neutral when their orientations disagree.
    pub(super) fn find_invalid_edges_sd(
        &mut self,
        lf_offset: &List<Shape>,
        loc_inv_edges: &mut ShapeIndexedMapMap,
        loc_valid_edges: &mut ShapeMapMap,
        neutral_edges: &mut ShapeMapMap,
    ) -> Result<(), Failure> {
        // 1. Find edges unclassified in faces
        // 2. Find SD faces in which the same edge is classified
        // 3. Check if the edge is neutral in face in which it wasn't classified
        let mut me_unclassified: rustycad_tkernel::ncollection::IndexedDataMap<
            Shape,
            MapOfShape,
            rustycad_tkbrep::toptools::ShapeMapHasher,
        > = Default::default();
        let mut f_split_f_offset = ShapeShapeMap::default();

        // Avoid artificial faces
        let mut new_faces = MapOfShape::default();
        if let Some(analyzer) = self.my_analyzer {
            let mut map_new_tmp = MapOfShape::default();
            for f in analyzer.new_faces() {
                map_new_tmp.add(f.clone());
            }
            for f_offset in lf_offset.iter() {
                let f_origin = self.faces_origins_ref()?.seek(f_offset).ok_or_else(|| {
                    Failure::with_message("Standard_NoSuchObject", "NCollection_DataMap::Find")
                })?;
                if map_new_tmp.contains(f_origin) {
                    new_faces.add(f_offset.clone());
                }
            }
        }

        let mut an_ef_map = IndexedDataMapOfShapeListOfShape::default();
        for f in lf_offset.iter() {
            if new_faces.contains(f) {
                continue;
            }
            let lf_images = self.my_of_images.find_from_key(f)?.clone();
            for f_im in lf_images.iter() {
                topexp::map_shapes_and_ancestors(
                    f_im,
                    ShapeEnum::Edge,
                    ShapeEnum::Face,
                    &mut an_ef_map,
                )?;

                for e in Explorer::new(f_im, ShapeEnum::Edge, ShapeEnum::Shape)? {
                    if self.my_invalid_edges.contains(&e) != self.my_valid_edges.contains(&e) {
                        // edge is classified in some face
                        let in_invalid = loc_inv_edges.seek(f_im).is_some_and(|m| m.contains(&e));
                        let in_valid = loc_valid_edges.seek(f_im).is_some_and(|m| m.contains(&e));
                        if !in_invalid && !in_valid {
                            // but not in the current one
                            if !me_unclassified.contains(&e) {
                                me_unclassified.add(e.clone(), MapOfShape::default());
                            }
                            me_unclassified.change_from_key(&e)?.add(f_im.clone());
                            f_split_f_offset.bind(f_im.clone(), f.clone());
                        }
                    }
                }
            }
        }

        if me_unclassified.is_empty() {
            return Ok(());
        }

        // Analyze unclassified edges
        for i_e in 1..=me_unclassified.extent() {
            let e = me_unclassified.find_key(i_e)?.clone();
            let mf_unclassified = me_unclassified.find_from_index(i_e)?.clone();
            let lf: Vec<Shape> = an_ef_map.find_from_key(&e)?.iter().cloned().collect();

            for f_classified in &lf {
                if mf_unclassified.contains(f_classified) {
                    continue;
                }

                let mut edge_set_class = ShapeSet::new();
                edge_set_class.add(f_classified, ShapeEnum::Edge)?;

                let e_classified = find_shape(&e, f_classified, None)?.unwrap_or_default();
                let ori_class = e_classified.orientation();

                let dn_class = normal_on_edge(&e_classified, f_classified)?;

                let is_invalid = loc_inv_edges
                    .seek(f_classified)
                    .is_some_and(|m| m.contains(&e));

                for f_unclassified in mf_unclassified.iter() {
                    let mut edge_set_unclass = ShapeSet::new();
                    edge_set_unclass.add(f_unclassified, ShapeEnum::Edge)?;

                    if edge_set_class.is_equal(&edge_set_unclass) {
                        let dn_unclass = normal_on_edge(&e, f_unclassified)?;

                        let is_same_ori = dn_class.is_equal(&dn_unclass, precision::angular());

                        // Among other splits of the same face find those where the edge is contained with
                        // different orientation
                        let f_offset =
                            f_split_f_offset
                                .seek(f_unclassified)
                                .cloned()
                                .ok_or_else(|| {
                                    Failure::with_message(
                                        "Standard_NoSuchObject",
                                        "NCollection_DataMap::Find",
                                    )
                                })?;
                        let lf_splits = self.my_of_images.find_from_key(&f_offset)?.clone();
                        for f_sp in lf_splits.iter() {
                            if !f_sp.is_same(f_unclassified) && mf_unclassified.contains(f_sp) {
                                let e_unclassified =
                                    find_shape(&e, f_sp, None)?.unwrap_or_default();

                                let mut ori_unclass = e_unclassified.orientation();
                                if !is_same_ori {
                                    ori_unclass = top_abs::reverse(ori_unclass);
                                }

                                if ori_class != ori_unclass {
                                    // make the edge neutral for the face
                                    if !neutral_edges.is_bound(&f_offset) {
                                        neutral_edges.bind(f_offset.clone(), MapOfShape::default());
                                    }
                                    if let Some(m) = neutral_edges.change_seek(&f_offset) {
                                        m.add(e.clone());
                                    }

                                    if is_invalid && is_same_ori {
                                        // make edge invalid in aFUnclassified and valid in aFSp
                                        add_as_neutral(
                                            &e,
                                            f_classified,
                                            f_sp,
                                            loc_inv_edges,
                                            loc_valid_edges,
                                        );
                                    } else {
                                        // make edge invalid in aFSp and valid in aFUnclassified
                                        add_as_neutral(
                                            &e,
                                            f_sp,
                                            f_classified,
                                            loc_inv_edges,
                                            loc_valid_edges,
                                        );
                                    }
                                }
                            }
                        }
                        // (the native loop never breaks: its iterator is exhausted here)
                    }
                }
            }
        }
        Ok(())
    }

    /// `MakeInvertedEdgesInvalid(theLFOffset)`: the inverted edges of the splits whose two
    /// vertices lie on the same closed loop of invalid edges made invalid.
    pub(super) fn make_inverted_edges_invalid(
        &mut self,
        lf_offset: &List<Shape>,
    ) -> Result<(), Failure> {
        if self.my_invalid_edges.is_empty() || self.my_inverted_edges.is_empty() {
            return Ok(());
        }

        // Map all invalid edges
        let mut cbe_inv = Compound::default();
        BRepBuilder::default().make_compound(&mut cbe_inv);
        for i in 1..=self.my_invalid_edges.extent() {
            BRepBuilder::default().add(&mut cbe_inv, self.my_invalid_edges.find_key(i)?)?;
        }

        // Make loops of invalid edges
        let mut lcb = Vec::new();
        bop_tools::make_connexity_compounds(
            &cbe_inv,
            ShapeEnum::Vertex,
            ShapeEnum::Edge,
            &mut lcb,
        )?;

        // Analyze each loop on closeness and use only closed ones
        let mut dmvcb = ShapeShapeMap::default();

        for cb in lcb.iter() {
            let mut dmve = IndexedDataMapOfShapeListOfShape::default();
            topexp::map_shapes_and_ancestors(cb, ShapeEnum::Vertex, ShapeEnum::Edge, &mut dmve)?;
            let mut is_closed = true;
            for i_v in 1..=dmve.extent() {
                if dmve.find_from_index(i_v)?.len() != 2 {
                    is_closed = false;
                    break;
                }
            }
            if !is_closed {
                continue;
            }

            // Bind loop to each vertex of the loop
            for i_v in 1..=dmve.extent() {
                dmvcb.bind(dmve.find_key(i_v)?.clone(), cb.clone());
            }
        }

        // Check if any inverted edges of offset faces are locked inside the loops of invalid edges.
        // Make such edges invalid as well.
        for f in lf_offset.iter() {
            let lf_im = self.my_of_images.find_from_key(f)?.clone();
            for f_im in lf_im.iter() {
                for e in Explorer::new(f_im, ShapeEnum::Edge, ShapeEnum::Shape)? {
                    let e = topods::edge(&e)?;
                    if !self.my_invalid_edges.contains(&e) && self.my_inverted_edges.contains(&e) {
                        let cb1 = dmvcb.seek(&topexp::first_vertex(&e, false));
                        let cb2 = dmvcb.seek(&topexp::last_vertex(&e, false));
                        if let (Some(cb1), Some(cb2)) = (cb1, cb2)
                            && cb1.is_same(cb2)
                        {
                            self.my_invalid_edges.add(e.into_shape());
                        }
                    }
                }
            }
        }
        Ok(())
    }
}

/// Standard_NoSuchObject of the first item of an empty list.
fn empty() -> Failure {
    Failure::with_message("Standard_NoSuchObject", "NCollection_List::First")
}
