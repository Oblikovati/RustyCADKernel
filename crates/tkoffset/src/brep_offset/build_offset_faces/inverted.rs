// Rust translation of BRepOffset_BuildOffsetFaces::CheckInverted, CheckInvertedBlock (with
// the static GetVerticesOnEdges), RemoveInvalidSplitsByInvertedEdges and
// RemoveInvalidSplitsFromValid of
// ModelingAlgorithms/TKOffset/BRepOffset/BRepOffset_MakeOffset_1.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The inverted edges (whose vertex order changed with respect to the original face) and
//! the removal of the invalid splits made of them.
use super::{
    BuildOffsetFaces, ShapeListMap, ShapeMapMap, ShapeShapeMap, append_to_list, find_shape,
};
use rustycad_tkbo::bop_tools;
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::topexp::{self, Explorer};
use rustycad_tkbrep::topods::{self, Compound, Edge, Face, Shape, ShapeIterator, Vertex, Wire};
use rustycad_tkbrep::toptools::{IndexedDataMapOfShapeListOfShape, IndexedMapOfShape, MapOfShape};
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::gp::Vector3;
use std::f64::consts::PI;

/// `GetVerticesOnEdges(theCB, theEdges, theVerticesOnEdges, theAllVertices)`: the vertices of
/// the block, and those on the given edges.
fn get_vertices_on_edges(
    cb: &Shape,
    edges: &IndexedMapOfShape,
    vertices_on_edges: &mut MapOfShape,
    all_vertices: &mut MapOfShape,
) -> Result<(), Failure> {
    for e in Explorer::new(cb, ShapeEnum::Edge, ShapeEnum::Shape)? {
        let is_on_given_edges = edges.contains(&e);
        for v in ShapeIterator::new(&e, true, true) {
            all_vertices.add(v.clone());
            if is_on_given_edges {
                vertices_on_edges.add(v);
            }
        }
    }
    Ok(())
}

/// Standard_NoSuchObject of a missing data map key.
fn not_found() -> Failure {
    Failure::with_message("Standard_NoSuchObject", "NCollection_DataMap::Find")
}

impl BuildOffsetFaces<'_> {
    /// `CheckInverted(theEIm, theFOr, theDMVE, theMEdges)`: whether the direction between the
    /// vertices of the offset edge (or of the chain of images of its origins) is opposite to
    /// the one between the corresponding vertices of the original face; the images are then
    /// recorded as inverted.
    pub(super) fn check_inverted(
        &mut self,
        e_im: &Edge,
        f_or: &Face,
        dmve: &IndexedDataMapOfShapeListOfShape,
        m_edges: &IndexedMapOfShape,
    ) -> Result<bool, Failure> {
        // It is necessary to compare the direction from first vertex
        // to the last vertex on the original edge with the
        // same direction on the new edge. If the directions
        // will be different - the edge has been inverted.
        //
        // find vertices of the offset shape
        let (mut vi1, mut vi2) = topexp::vertices(e_im, false);

        // find images
        let mut le_images: List<Shape> = List::default();
        if let Some(l_offset_or) = self.my_oe_origins.seek(e_im).cloned() {
            let mut images = Wire::default();
            BRepBuilder::default().make_wire(&mut images);

            let mut m_im_fence = MapOfShape::default();
            for e_offset_or in l_offset_or.iter() {
                let l_images = self.my_oe_images.seek(e_offset_or).ok_or_else(not_found)?;
                for im in l_images.iter() {
                    if m_edges.contains(im) && m_im_fence.add(im.clone()) {
                        BRepBuilder::default().add(&mut images, im)?;
                        le_images.append(im.clone());
                    }
                }
            }

            // find alone vertices
            let mut dm_im_ve = IndexedDataMapOfShapeListOfShape::default();
            topexp::map_shapes_and_ancestors(
                &images,
                ShapeEnum::Vertex,
                ShapeEnum::Edge,
                &mut dm_im_ve,
            )?;

            let mut lv_alone: List<Shape> = List::default();
            for i in 1..=dm_im_ve.extent() {
                if dm_im_ve.find_from_index(i)?.len() == 1 {
                    lv_alone.append(dm_im_ve.find_key(i)?.clone());
                }
            }

            if lv_alone.len() > 1 {
                let vw1 = topods::vertex(lv_alone.first().ok_or_else(not_found)?)?;
                let vw2 = topods::vertex(lv_alone.last().ok_or_else(not_found)?)?;

                // check distances
                let pi1 = BRepTool::pnt(&vi1)?;
                let pw1 = BRepTool::pnt(&vw1)?;
                let pw2 = BRepTool::pnt(&vw2)?;

                let dist1 = pi1.square_distance(&pw1);
                let dist2 = pi1.square_distance(&pw2);

                if dist1 < dist2 {
                    vi1 = vw1;
                    vi2 = vw2;
                } else {
                    vi1 = vw2;
                    vi2 = vw1;
                }
            }
        } else {
            le_images.append(e_im.clone().into_shape());
        }

        // Find edges connected to these vertices
        let lie1 = dmve.find_from_key(&vi1)?.clone();
        let lie2 = dmve.find_from_key(&vi2)?.clone();

        // Find vertices on the original face corresponding to vertices on the offset edge
        //
        // find original edges for both lists
        let mut loe1: List<Shape> = List::default();
        let mut loe2: List<Shape> = List::default();
        for (lie, loe) in [(&lie1, &mut loe1), (&lie2, &mut loe2)] {
            let mut m_fence = MapOfShape::default();
            for ei in lie.iter() {
                let Some(le_origins) = self.edges_origins().seek(ei).cloned() else {
                    continue;
                };
                for eo in le_origins.iter() {
                    if eo.shape_type()? == ShapeEnum::Edge
                        && m_fence.add(eo.clone())
                        && find_shape(eo, f_or, None)?.is_some()
                    {
                        append_to_list(loe, eo);
                    }
                }
            }
        }

        if loe1.len() < 2 || loe2.len() < 2 {
            return Ok(false);
        }

        // find vertices common for the max number of edges in the lists
        let mut vo = [Vertex::default(), Vertex::default()];
        for (k, loe) in [&loe1, &loe2].into_iter().enumerate() {
            let mut dmve_loc = IndexedDataMapOfShapeListOfShape::default();
            for e in loe.iter() {
                topexp::map_shapes_and_ancestors(
                    e,
                    ShapeEnum::Vertex,
                    ShapeEnum::Edge,
                    &mut dmve_loc,
                )?;
            }

            let mut nb_e_max = 0;
            for j in 1..=dmve_loc.extent() {
                let nb_e = dmve_loc.find_from_index(j)?.len();
                if nb_e > 1 && nb_e > nb_e_max {
                    vo[k] = topods::vertex(dmve_loc.find_key(j)?)?;
                    nb_e_max = nb_e;
                }
            }
            if vo[k].is_null() {
                return Ok(false);
            }
        }
        let [vo1, vo2] = vo;

        if vo1.is_same(&vo2) {
            return Ok(false);
        }

        // check positions of the offset and original vertices
        let pi1 = BRepTool::pnt(&vi1)?;
        let pi2 = BRepTool::pnt(&vi2)?;
        let po1 = BRepTool::pnt(&vo1)?;
        let po2 = BRepTool::pnt(&vo2)?;

        let v_i = Vector3::between(pi1, pi2);
        let v_o = Vector3::between(po1, po2);

        let angle = v_i.angle(v_o)?;
        let b_inverted = (angle - PI).abs() < 1.0e-4;
        if b_inverted {
            for e_invr in le_images.iter() {
                self.my_inverted_edges.add(e_invr.clone());
            }
        }
        Ok(b_inverted)
    }

    /// `CheckInvertedBlock(theCB, theLCBF, theDMCBVInverted, theDMCBVAll)`: whether the
    /// block of splits can be removed: more than one face, two connected inverted edges of
    /// different origins, no inverted vertex shared with another block.
    pub(super) fn check_inverted_block(
        &self,
        cb: &Shape,
        lcbf: &[Shape],
        dmcbv_inverted: &mut ShapeMapMap,
        dmcbv_all: &mut ShapeMapMap,
    ) -> Result<bool, Failure> {
        // For possible removal of the block:
        // 1. There should be more than just one face in the block
        if cb.nb_children() < 2 {
            return Ok(false);
        }

        // 2. The block should at least contain two connected inverted edges with
        //    different origins (not just two images/splits of the same edge)
        let mut me_cb_inv = MapOfShape::default();
        let mut ce_cb_inv = Compound::default();
        BRepBuilder::default().make_compound(&mut ce_cb_inv);
        for e in Explorer::new(cb, ShapeEnum::Edge, ShapeEnum::Shape)? {
            if self.my_inverted_edges.contains(&e) && me_cb_inv.add(e.clone()) {
                BRepBuilder::default().add(&mut ce_cb_inv, &e)?;
            }
        }

        if me_cb_inv.extent() < 2 {
            return Ok(false);
        }

        // check that the edges are connected and different
        let mut lcbe = Vec::new();
        bop_tools::make_connexity_compounds(
            &ce_cb_inv,
            ShapeEnum::Vertex,
            ShapeEnum::Edge,
            &mut lcbe,
        )?;

        let mut found = false;
        for cbe in &lcbe {
            // count the unique edges in the block
            let mut nb_unique = 0;
            let mut me_origins = MapOfShape::default();
            for e in ShapeIterator::new(cbe, true, true) {
                let Some(le_or) = self.my_oe_origins.seek(&e) else {
                    me_origins.add(e);
                    nb_unique += 1;
                    continue;
                };
                for e_or in le_or.iter() {
                    if me_origins.add(e_or.clone()) {
                        nb_unique += 1;
                    }
                }
            }

            if nb_unique >= 2 {
                found = true;
                break;
            }
        }

        if !found {
            return Ok(false);
        }

        // 3. the block should not contain inverted edges which vertices
        //    are contained in other blocks
        //
        // collect vertices from inverted edges and compare them with
        // vertices from other blocks
        if !dmcbv_inverted.is_bound(cb) {
            let mut mv_inverted = MapOfShape::default();
            let mut mv_all = MapOfShape::default();
            get_vertices_on_edges(cb, &self.my_inverted_edges, &mut mv_inverted, &mut mv_all)?;
            dmcbv_inverted.bind(cb.clone(), mv_inverted);
            dmcbv_all.bind(cb.clone(), mv_all);
        }

        for cb1 in lcbf {
            if cb1.is_same(cb) {
                continue;
            }

            // collect vertices from inverted edges
            if !dmcbv_inverted.is_bound(cb1) {
                let mut mv_inverted1 = MapOfShape::default();
                let mut mv_all1 = MapOfShape::default();
                get_vertices_on_edges(
                    cb1,
                    &self.my_inverted_edges,
                    &mut mv_inverted1,
                    &mut mv_all1,
                )?;
                dmcbv_inverted.bind(cb1.clone(), mv_inverted1);
                dmcbv_all.bind(cb1.clone(), mv_all1);
            }

            let mv_inverted = dmcbv_inverted.seek(cb).ok_or_else(not_found)?;
            let mv_all1 = dmcbv_all.seek(cb1).ok_or_else(not_found)?;
            if mv_inverted.has_intersection(mv_all1) {
                return Ok(false);
            }
        }

        Ok(true)
    }

    /// `RemoveInvalidSplitsByInvertedEdges(theMERemoved)`: the invalid splits with at least
    /// two disconnected removable inverted edges removed from the invalid faces and the
    /// images; the edges of the removed splits collected.
    pub(super) fn remove_invalid_splits_by_inverted_edges(
        &mut self,
        me_removed: &mut IndexedMapOfShape,
    ) -> Result<(), Failure> {
        if self.my_inverted_edges.is_empty() {
            return Ok(());
        }

        // check the faces on regularity, i.e. the splits of the same face
        // should not be connected only by vertex. Such irregular splits
        // will have to be rebuilt and cannot be removed.
        let bb = BRepBuilder::default();
        let mut me_avoid = IndexedMapOfShape::default();
        let mut dmvf = ShapeListMap::default();
        for i in 1..=self.my_of_images.extent() {
            let lf_im = self.my_of_images.find_from_index(i)?.clone();

            let mut cf_im = Compound::default();
            bb.make_compound(&mut cf_im);

            let mut dmef = ShapeListMap::default();
            for f in lf_im.iter() {
                bb.add(&mut cf_im, f)?;

                // make a map to use only outer edges
                for e in Explorer::new(f, ShapeEnum::Edge, ShapeEnum::Shape)? {
                    if dmef.is_bound(&e) {
                        // internal edges should not be used
                        me_avoid.add(e.clone());
                    } else {
                        dmef.bind(e.clone(), List::default());
                    }
                    if let Some(lf) = dmef.change_seek(&e) {
                        append_to_list(lf, f);
                    }
                }

                // fill connection map of the vertices of inverted edges to faces
                for v in Explorer::new(f, ShapeEnum::Vertex, ShapeEnum::Shape)? {
                    if !dmvf.is_bound(&v) {
                        dmvf.bind(v.clone(), List::default());
                    }
                    if let Some(lf) = dmvf.change_seek(&v) {
                        append_to_list(lf, f);
                    }
                }
            }

            // for the splits to be regular they should form only one block
            let mut lcbf = Vec::new();
            bop_tools::make_connexity_compounds(
                &cf_im,
                ShapeEnum::Edge,
                ShapeEnum::Face,
                &mut lcbf,
            )?;
            if lcbf.len() == 1 {
                continue;
            }

            // check if the inverted edges create the irregularity
            let mut dmcbv_inverted = ShapeMapMap::default();
            let mut dmcbv_all = ShapeMapMap::default();

            for cb in &lcbf {
                // check if it is possible to remove the block
                if !self.check_inverted_block(cb, &lcbf, &mut dmcbv_inverted, &mut dmcbv_all)? {
                    // non of the edges in this block should be removed
                    topexp::map_shapes_of_type(cb, ShapeEnum::Edge, &mut me_avoid)?;
                    continue;
                }
            }
        }

        // all edges not included in aMEAvoid can be removed
        let mut me_rem = MapOfShape::default();
        for i_inverted in 1..=self.my_inverted_edges.extent() {
            let e = self.my_inverted_edges.find_key(i_inverted)?;
            if !me_avoid.contains(e) {
                for v in ShapeIterator::new(e, true, true) {
                    if dmvf.seek(&v).is_some_and(|lf| lf.len() > 3) {
                        me_rem.add(e.clone());
                        break;
                    }
                }
            }
        }

        if me_rem.is_empty() {
            return Ok(());
        }

        // all invalid faces containing these edges can be removed
        let mut inv_faces = IndexedDataMapOfShapeListOfShape::default();
        let mut mf_rem = MapOfShape::default();
        let mut mf_to_update = IndexedMapOfShape::default();
        for i in 1..=self.my_invalid_faces.extent() {
            let f = self.my_invalid_faces.find_key(i)?.clone();
            let lf_im = self.my_invalid_faces.change_from_index(i)?;

            lf_im.try_retain(|f_im| -> Result<bool, Failure> {
                // to be removed the face should have at least two not connected
                // inverted edges
                let mut ce_inv = Compound::default();
                bb.make_compound(&mut ce_inv);
                for e in Explorer::new(f_im, ShapeEnum::Edge, ShapeEnum::Shape)? {
                    if me_rem.contains(&e) {
                        bb.add(&mut ce_inv, &e)?;
                    }
                }

                // check connectivity
                let mut lcbe = Vec::new();
                bop_tools::make_connexity_compounds(
                    &ce_inv,
                    ShapeEnum::Vertex,
                    ShapeEnum::Edge,
                    &mut lcbe,
                )?;

                if lcbe.len() >= 2 {
                    mf_to_update.add(f.clone());
                    mf_rem.add(f_im.clone());
                    Ok(false)
                } else {
                    Ok(true)
                }
            })?;

            if !lf_im.is_empty() {
                inv_faces.add(f, lf_im.clone());
            }
        }

        if mf_rem.is_empty() {
            return Ok(());
        }

        self.my_invalid_faces = inv_faces;
        // remove from splits
        for i in 1..=mf_to_update.extent() {
            let f = mf_to_update.find_key(i)?;
            let lf_im = self.my_of_images.change_from_key(f)?;
            lf_im.try_retain(|f_im| -> Result<bool, Failure> {
                if mf_rem.contains(f_im) {
                    topexp::map_shapes_of_type(f_im, ShapeEnum::Edge, me_removed)?;
                    Ok(false)
                } else {
                    Ok(true)
                }
            })?;
        }
        Ok(())
    }

    /// `RemoveInvalidSplitsFromValid(theDMFMVIE)`: the blocks of invalid splits whose free
    /// edges are all valid (or made of the inverted edges) removed from the images.
    pub(super) fn remove_invalid_splits_from_valid(
        &mut self,
        dmfmvie: &ShapeMapMap,
    ) -> Result<(), Failure> {
        // Decide whether to remove the found invalid faces or not.
        // The procedure is the following:
        // 1. Make connexity blocks from invalid faces;
        // 2. Find free edges in this blocks;
        // 3. If all free edges are valid for the faces - remove block.
        let mut m_fence = MapOfShape::default();
        let mut mf_to_rem = MapOfShape::default();
        let bb = BRepBuilder::default();
        let mut cf_inv = Compound::default();
        bb.make_compound(&mut cf_inv);

        // make compound of invalid faces
        let mut dmifof = ShapeShapeMap::default();
        for i in 1..=self.my_invalid_faces.extent() {
            let f = self.my_invalid_faces.find_key(i)?;
            // artificially invalid faces should not be removed
            if self.my_art_invalid_faces.is_bound(f) {
                continue;
            }
            for f_im in self.my_invalid_faces.find_from_index(i)?.iter() {
                if m_fence.add(f_im.clone()) {
                    bb.add(&mut cf_inv, f_im)?;
                    dmifof.bind(f_im.clone(), f.clone());
                }
            }
        }

        // make connexity blocks
        let mut lcb_inv = Vec::new();
        bop_tools::make_connexity_compounds(
            &cf_inv,
            ShapeEnum::Edge,
            ShapeEnum::Face,
            &mut lcb_inv,
        )?;

        // analyze each block
        for cb in &lcb_inv {
            // if connexity block contains only one face - it should be removed;
            let faces: Vec<Shape> = Explorer::new(cb, ShapeEnum::Face, ShapeEnum::Shape)?.collect();
            let mut all_removable = true;
            if faces.len() > 1 {
                // check if there are valid images left
                for f_im in &faces {
                    let f = dmifof.seek(f_im).ok_or_else(not_found)?;
                    let lf_im = self.my_of_images.find_from_key(f)?;
                    let lf_inv = self.my_invalid_faces.find_from_key(f)?;
                    if lf_im.len() == lf_inv.len() {
                        all_removable = false;
                        break;
                    }
                }
            }

            if all_removable {
                for f in &faces {
                    mf_to_rem.add(f.clone());
                }
                continue;
            }

            // remove faces connected by inverted edges
            let mut dmef = IndexedDataMapOfShapeListOfShape::default();
            topexp::map_shapes_and_ancestors(cb, ShapeEnum::Edge, ShapeEnum::Face, &mut dmef)?;

            let mut dmff = ShapeListMap::default();
            for f_cb in &faces {
                let f = dmifof.seek(f_cb).ok_or_else(not_found)?.clone();
                if !dmff.is_bound(&f) {
                    dmff.bind(f.clone(), List::default());
                }
                if let Some(list) = dmff.change_seek(&f) {
                    list.append(f_cb.clone());
                }
            }

            // (each entry is decided independently: the hashed order does not matter)
            for (f, lf_cb) in dmff.items() {
                let p_valid_inverted = dmfmvie.seek(f);

                // either remove all of these faces or none.
                let mut remove = false;
                for f_cb in lf_cb.iter() {
                    let mut kept_by_edge = false;
                    for e_cb in Explorer::new(f_cb, ShapeEnum::Edge, ShapeEnum::Shape)? {
                        if p_valid_inverted.is_some_and(|m| m.contains(&e_cb)) {
                            kept_by_edge = true;
                            break;
                        }
                        if dmef.find_from_key(&e_cb)?.len() > 1
                            && !self.my_inverted_edges.contains(&e_cb)
                        {
                            kept_by_edge = true;
                            break;
                        }
                    }
                    if !kept_by_edge {
                        // if one removed - remove all
                        remove = true;
                        break;
                    }
                }
                if remove {
                    for f_cb in lf_cb.iter() {
                        mf_to_rem.add(f_cb.clone());
                    }
                }
            }
        }

        if mf_to_rem.extent() > 0 {
            // remove invalid faces from images
            for i in 1..=self.my_invalid_faces.extent() {
                let f = self.my_invalid_faces.find_key(i)?.clone();
                self.my_of_images
                    .change_from_key(&f)?
                    .retain(|f_im| !mf_to_rem.contains(f_im));
            }
        }
        Ok(())
    }
}
