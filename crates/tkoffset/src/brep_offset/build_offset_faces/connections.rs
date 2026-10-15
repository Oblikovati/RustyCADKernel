// Rust translation of BRepOffset_BuildOffsetFaces::ShapesConnections, RemoveHangingParts,
// RemoveValidSplits and RemoveInvalidSplits of
// ModelingAlgorithms/TKOffset/BRepOffset/BRepOffset_MakeOffset_1.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The connections between the offset faces found by the intersection of the splits, and
//! the removal of the hanging and removed splits from the images.
use super::{
    BuildOffsetFaces, ShapeListMap, ShapeShapeMap, add_to_map_list, append_to_list, take_modified,
};
use rustycad_tkbo::bop_algo::{BOPAlgoBuilder, BOPAlgoMakerVolume};
use rustycad_tkbo::bop_ds::DataStructure;
use rustycad_tkbo::bop_tools;
use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::topexp::{self, Explorer};
use rustycad_tkbrep::topods::{Compound, Shape, ShapeIterator};
use rustycad_tkbrep::toptools::{IndexedDataMapOfShapeListOfShape, IndexedMapOfShape, MapOfShape};
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::ShapeEnum;

/// The data structure of the intersection of a builder (Standard_NullObject without one,
/// a null dereference natively).
fn ds_of(builder: &BOPAlgoBuilder) -> Result<&DataStructure, Failure> {
    builder
        .pave_filler()
        .and_then(|pf| pf.data_structure())
        .ok_or_else(|| Failure::with_message("Standard_NullObject", "BOPAlgo_Builder::PDS"))
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

impl BuildOffsetFaces<'_> {
    /// `ShapesConnections(theDMFOr, theBuilder)`: the offset faces whose invalid splits are
    /// intersected by the splits of other faces recorded as interfering (with the original
    /// invalid edges), and the intersections of the artificially invalid faces.
    pub(super) fn shapes_connections(
        &mut self,
        dmf_or: &ShapeShapeMap,
        builder: &BOPAlgoBuilder,
    ) -> Result<(), Failure> {
        // Make connexity blocks from invalid edges to use the whole block
        // to which the edge is connected instead of the single edge.
        let mut ce_inv = Compound::default();
        BRepBuilder::default().make_compound(&mut ce_inv);
        for i in 1..=self.my_invalid_edges.extent() {
            BRepBuilder::default().add(&mut ce_inv, self.my_invalid_edges.find_key(i)?)?;
        }

        let mut lcb = Vec::new();
        bop_tools::make_connexity_compounds(&ce_inv, ShapeEnum::Vertex, ShapeEnum::Edge, &mut lcb)?;

        // Binding from the edge to the block
        let mut ecb_map = ShapeShapeMap::default();
        for cb in &lcb {
            for e in ShapeIterator::new(cb, true, true) {
                ecb_map.bind(e, cb.clone());
            }
        }

        // update invalid edges with images and keep connection to original edge
        let mut dme_or = ShapeListMap::default();
        for i in 1..=self.my_invalid_edges.extent() {
            let e_inv = self.my_invalid_edges.find_key(i)?;
            let le_im = builder.modified(e_inv)?;
            if le_im.is_empty() {
                let mut l = List::default();
                l.append(e_inv.clone());
                dme_or.bind(e_inv.clone(), l);
                continue;
            }

            for e_im in &le_im {
                add_to_map_list(e_im, e_inv, &mut dme_or);
            }
        }

        // get shapes connections for using in the rebuilding process
        let ds = ds_of(builder)?;
        // analyze all Face/Face intersections
        for ff in ds.interf_ff_records().iter() {
            let vnc = ff.curves();
            if vnc.iter().len() == 0 {
                continue;
            }

            let f_im1 = ds.shape(ff.index1())?.clone();
            let f_im2 = ds.shape(ff.index2())?.clone();

            let (Some(f1), Some(f2)) = (dmf_or.seek(&f_im1).cloned(), dmf_or.seek(&f_im2).cloned())
            else {
                continue;
            };

            if f1.is_same(&f2) {
                continue;
            }

            let b_inv1 = self.my_invalid_faces.contains(&f1);
            let b_inv2 = self.my_invalid_faces.contains(&f2);

            if !b_inv1 && !b_inv2 {
                continue;
            }

            // check if it is real Face/Face intersection
            let mut me_int = MapOfShape::default();
            for nc in vnc.iter() {
                for pb in nc.pave_blocks() {
                    let mut n_e_int = 0;
                    let has_edge = pb
                        .lock()
                        .expect("pave-block lock")
                        .has_edge_into(&mut n_e_int);
                    if has_edge {
                        me_int.add(ds.shape(n_e_int)?.clone());
                    }
                }
            }

            if me_int.is_empty() {
                continue;
            }

            // check if invalid edges of the face are in the same splits with intersection edges
            for i in 0..2 {
                if (i == 0 && !b_inv1) || (i == 1 && !b_inv2) {
                    continue;
                }

                let (f, f_op, f_im) = if i == 0 {
                    (&f1, &f2, &f_im1)
                } else {
                    (&f2, &f1, &f_im2)
                };

                let mut b_found = false;

                let mut lf_im: List<Shape> = builder.modified(f_im)?.into_iter().collect();
                if lf_im.is_empty() {
                    lf_im.append(f_im.clone());
                }

                for f_im_im in lf_im.iter() {
                    let (mut b_inv, mut b_int) = (false, false);
                    for e in Explorer::new(f_im_im, ShapeEnum::Edge, ShapeEnum::Shape)? {
                        if !b_inv {
                            b_inv = dme_or.is_bound(&e);
                        }
                        if !b_int {
                            b_int = me_int.contains(&e);
                        }
                        if b_inv && b_int {
                            break;
                        }
                    }

                    if !b_int || !b_inv {
                        continue;
                    }

                    b_found = true;

                    // append opposite face to all invalid edges in the split
                    for e in Explorer::new(f_im_im, ShapeEnum::Edge, ShapeEnum::Shape)? {
                        let Some(le_or) = dme_or.seek(&e) else {
                            continue;
                        };
                        for e_or in le_or.iter() {
                            append_bound(&mut self.my_ss_interfs, e_or, f_op);
                        }
                    }
                }
                if b_found {
                    // save connection between offset faces
                    append_bound(&mut self.my_ss_interfs, f, f_op);
                }
            }

            // Treatment for the artificial case - check if one of the faces is artificially invalid
            for i_f in 0..2 {
                let (f_art, f_opposite) = if i_f == 0 { (&f1, &f2) } else { (&f2, &f1) };

                if !self.my_art_invalid_faces.is_bound(f_art) {
                    continue;
                }

                if self.my_invalid_faces.contains(f_opposite)
                    && !self.my_art_invalid_faces.is_bound(f_opposite)
                {
                    continue;
                }

                // Collect own invalid edges of the face and the invalid edges connected to those
                // own invalid edges to be avoided in the check for intersection.
                let mut me_avoid = IndexedMapOfShape::default();
                if let Some(fe_inv) = self.my_art_invalid_faces.seek(f_opposite) {
                    for i_e in 1..=fe_inv.extent() {
                        if let Some(cb) = ecb_map.seek(fe_inv.find_key(i_e)?) {
                            topexp::map_shapes_of_type(cb, ShapeEnum::Edge, &mut me_avoid)?;
                        }
                    }
                } else if let Some(lf_im) = self.my_of_images.seek(f_opposite) {
                    for f in lf_im.iter() {
                        for e in Explorer::new(f, ShapeEnum::Edge, ShapeEnum::Shape)? {
                            if let Some(cb) = ecb_map.seek(&e) {
                                topexp::map_shapes_of_type(cb, ShapeEnum::Edge, &mut me_avoid)?;
                            }
                        }
                    }
                }

                let (f_art_im, f_opposite_im) = if i_f == 0 {
                    (&f_im1, &f_im2)
                } else {
                    (&f_im2, &f_im1)
                };

                // Check if there are any intersections between edges of artificially
                // invalid face and opposite face
                let n_f_op = ds.index(f_opposite_im);

                for e in Explorer::new(f_art_im, ShapeEnum::Edge, ShapeEnum::Shape)? {
                    if !self.my_invalid_edges.contains(&e)
                        || self.my_inverted_edges.contains(&e)
                        || me_avoid.contains(&e)
                    {
                        continue;
                    }
                    let n_e = ds.index(&e);
                    if n_e < 0 {
                        continue;
                    }

                    if !ds.has_interference_between(n_e, n_f_op) {
                        continue;
                    }

                    let mut lv: List<Shape> = List::default();
                    for ef in ds.interf_ef_records().iter() {
                        if ef.contains(n_e)
                            && ef.contains(n_f_op)
                            && ef.common_part().shape_type() == ShapeEnum::Vertex
                        {
                            lv.append(ds.shape(ef.index_new())?.clone());
                        }
                    }

                    if lv.is_empty() {
                        continue;
                    }

                    // Make sure that there is an opposite intersection exists, i.e. some of the edges
                    // of the opposite face intersect the artificially invalid face.
                    let n_f_art = ds.index(f_art_im);
                    let mut opposite = false;
                    for e_op in Explorer::new(f_opposite_im, ShapeEnum::Edge, ShapeEnum::Shape)? {
                        let n_e_op = ds.index(&e_op);
                        if ds.has_interference_between(n_e_op, n_f_art) {
                            opposite = true;
                            break;
                        }
                    }
                    if !opposite {
                        continue;
                    }

                    // Intersection is present - add connection between offset faces.
                    add_to_map_list(f_art, f_opposite, &mut self.my_ss_interfs_art);

                    // Add connection between edge and opposite face
                    add_to_map_list(&e, f_opposite, &mut self.my_ss_interfs_art);

                    // Along with the opposite face, save the intersection vertices to
                    // be used for trimming the intersection edge in the rebuilding process
                    for v in lv.iter() {
                        // Add connection to intersection vertex
                        add_to_map_list(&e, v, &mut self.my_ss_interfs_art);
                    }
                }
            }
        }
        Ok(())
    }

    /// `RemoveHangingParts(theMV, theDMFImF, theMFInv, theMFToRem)`: the blocks of splits
    /// outside the solids containing inverted edges, or invalid splits not connected to the
    /// invalid parts of the solids (or isolated), marked for removal.
    pub(super) fn remove_hanging_parts(
        &self,
        mv: &BOPAlgoMakerVolume,
        dmf_im_f: &ShapeShapeMap,
        mf_inv: &IndexedMapOfShape,
        mf_to_rem: &mut MapOfShape,
    ) -> Result<(), Failure> {
        // Map the faces of the result solids to filter them from avoided faces
        let mut mfs = IndexedMapOfShape::default();
        topexp::map_shapes_of_type(mv.shape(), ShapeEnum::Face, &mut mfs)?;

        // Build compound of all faces not included into solids
        let mut cf_hangs_compound = Compound::default();
        BRepBuilder::default().make_compound(&mut cf_hangs_compound);
        let mut cf_hangs = cf_hangs_compound.into_shape();

        // Tool for getting the splits of faces
        let mv_ims = mv.images();

        for arg in mv.arguments().iter() {
            for f in Explorer::new(arg, ShapeEnum::Face, ShapeEnum::Shape)? {
                take_modified(&f, mv_ims, &mut cf_hangs, Some(&mut mfs));
            }
        }

        // Make connexity blocks of all hanging parts and check that they are isolated
        let mut lcb_hangs = Vec::new();
        bop_tools::make_connexity_compounds(
            &cf_hangs,
            ShapeEnum::Edge,
            ShapeEnum::Face,
            &mut lcb_hangs,
        )?;
        if lcb_hangs.is_empty() {
            return Ok(());
        }

        // To be removed, the block should contain invalid splits of offset faces and should
        // meet one of the following conditions:
        // 1. The block should not be connected to any invalid parts (Faces or Edges)
        //    contained in solids;
        // 2. The block should be isolated from other faces, i.e. it should consist of
        //    the splits of the single offset face.

        // Map the edges and vertices of the result solids to check connectivity
        // of the hanging blocks to invalid parts contained in solids
        let mut dmef = IndexedDataMapOfShapeListOfShape::default();
        let mut dmve = IndexedDataMapOfShapeListOfShape::default();
        topexp::map_shapes_and_ancestors(mv.shape(), ShapeEnum::Edge, ShapeEnum::Face, &mut dmef)?;
        topexp::map_shapes_and_ancestors(
            mv.shape(),
            ShapeEnum::Vertex,
            ShapeEnum::Edge,
            &mut dmve,
        )?;

        // Update invalid edges with intersection results
        let mut me_inv = MapOfShape::default();
        for i in 1..=self.my_invalid_edges.extent() {
            take_modified(
                self.my_invalid_edges.find_key(i)?,
                mv_ims,
                &mut me_inv,
                None,
            );
        }

        // Update inverted edges with intersection results
        let mut me_inverted = MapOfShape::default();
        for i in 1..=self.my_inverted_edges.extent() {
            take_modified(
                self.my_inverted_edges.find_key(i)?,
                mv_ims,
                &mut me_inverted,
                None,
            );
        }

        // Tool for getting the origins of the splits
        let mv_ors = mv.origins();

        // Find hanging blocks to remove
        let mut blocks_to_remove: Vec<&Shape> = Vec::new();

        for cbh in &lcb_hangs {
            // Remove the block containing the inverted edges
            let mut b_has_inverted = false;
            for e in Explorer::new(cbh, ShapeEnum::Edge, ShapeEnum::Shape)? {
                b_has_inverted = !dmef.contains(&e) && me_inverted.contains(&e);
                if b_has_inverted {
                    break;
                }
            }

            if b_has_inverted {
                blocks_to_remove.push(cbh);
                continue;
            }

            // Check the block to contain invalid split
            let mut b_has_invalid_face = false;
            // Check connectivity to invalid parts
            let mut b_is_connected = false;
            let mut block_me = IndexedMapOfShape::default();
            topexp::map_shapes_of_type(cbh, ShapeEnum::Edge, &mut block_me)?;
            // Map to collect all original faces
            let mut m_offset_f = MapOfShape::default();

            for f in Explorer::new(cbh, ShapeEnum::Face, ShapeEnum::Shape)? {
                // Check block to contain invalid face
                if !b_has_invalid_face {
                    b_has_invalid_face = mf_inv.contains(&f);
                }

                // Check block for connectivity to invalid parts
                if !b_is_connected {
                    // check edges
                    for e in Explorer::new(&f, ShapeEnum::Edge, ShapeEnum::Shape)? {
                        if let Some(lf) = dmef.seek(&e) {
                            for x in lf.iter() {
                                b_is_connected = mf_inv.contains(x);
                                if b_is_connected {
                                    break;
                                }
                            }
                        }
                        if b_is_connected {
                            break;
                        }
                    }
                    // check vertices
                    if !b_is_connected {
                        for v in Explorer::new(&f, ShapeEnum::Vertex, ShapeEnum::Shape)? {
                            if let Some(le) = dmve.seek(&v) {
                                for x in le.iter() {
                                    b_is_connected = !block_me.contains(x) && me_inv.contains(x);
                                    if b_is_connected {
                                        break;
                                    }
                                }
                            }
                            if b_is_connected {
                                break;
                            }
                        }
                    }
                }

                // Check block to be isolated
                if let Some(lf_or) = mv_ors.seek(&f) {
                    for f_or in lf_or.iter() {
                        if let Some(f_offset) = dmf_im_f.seek(f_or) {
                            m_offset_f.add(f_offset.clone());
                        }
                    }
                } else if let Some(f_offset) = dmf_im_f.seek(&f) {
                    m_offset_f.add(f_offset.clone());
                }
            }

            let b_remove = b_has_invalid_face && (!b_is_connected || m_offset_f.extent() == 1);

            if b_remove {
                blocks_to_remove.push(cbh);
            }
        }

        // remove the invalidated blocks
        for cbh in blocks_to_remove {
            for f in Explorer::new(cbh, ShapeEnum::Face, ShapeEnum::Shape)? {
                mf_to_rem.add(f);
            }
        }
        Ok(())
    }

    /// `RemoveValidSplits(theSpRem, theGF, theMERemoved)`: the splits (or those whose images
    /// are all) marked for removal removed from the images, their edges collected.
    pub(super) fn remove_valid_splits(
        &mut self,
        sp_rem: &MapOfShape,
        gf: &BOPAlgoBuilder,
        me_removed: &mut IndexedMapOfShape,
    ) -> Result<(), Failure> {
        for i in 1..=self.my_of_images.extent() {
            let ls_im = self.my_of_images.change_from_index(i)?;
            ls_im.try_retain(|s_im| -> Result<bool, Failure> {
                if sp_rem.contains(s_im) {
                    topexp::map_shapes_of_type(s_im, ShapeEnum::Edge, me_removed)?;
                    return Ok(false);
                }

                // check if all its images are have to be removed
                let ls_im_im = gf.modified(s_im)?;
                if !ls_im_im.is_empty() {
                    let mut b_all_rem = true;
                    for s_im_im in &ls_im_im {
                        if sp_rem.contains(s_im_im) {
                            topexp::map_shapes_of_type(s_im_im, ShapeEnum::Edge, me_removed)?;
                        } else {
                            b_all_rem = false;
                        }
                    }

                    if b_all_rem {
                        topexp::map_shapes_of_type(s_im, ShapeEnum::Edge, me_removed)?;
                        return Ok(false);
                    }
                }
                Ok(true)
            })?;
        }
        Ok(())
    }

    /// `RemoveInvalidSplits(theSpRem, theGF, theMERemoved)`: the invalid splits marked for
    /// removal (or all of whose images are, or whose invalid edges are all removed) removed
    /// from the invalid faces.
    pub(super) fn remove_invalid_splits(
        &mut self,
        sp_rem: &MapOfShape,
        gf: &BOPAlgoBuilder,
        me_removed: &mut IndexedMapOfShape,
    ) -> Result<(), Failure> {
        for i in 1..=self.my_invalid_faces.extent() {
            let s = self.my_invalid_faces.find_key(i)?.clone();
            let b_art = self.my_art_invalid_faces.is_bound(&s);
            let invalid_edges = &self.my_invalid_edges;

            let ls_im = self.my_invalid_faces.change_from_index(i)?;
            ls_im.try_retain(|s_im| -> Result<bool, Failure> {
                if sp_rem.contains(s_im) {
                    topexp::map_shapes_of_type(s_im, ShapeEnum::Edge, me_removed)?;
                    return Ok(false);
                }

                // check if all its images are have to be removed
                let ls_im_im = gf.modified(s_im)?;
                if ls_im_im.is_empty() {
                    return Ok(true);
                }

                let mut b_all_rem = true;
                let mut me_removed_loc = IndexedMapOfShape::default();
                for s_im_im in &ls_im_im {
                    if sp_rem.contains(s_im_im) {
                        topexp::map_shapes_of_type(s_im_im, ShapeEnum::Edge, &mut me_removed_loc)?;
                    } else {
                        b_all_rem = false;
                    }
                }

                if b_all_rem {
                    return Ok(false);
                }

                if b_art {
                    return Ok(true);
                }

                // remove the face from invalid if all invalid edges of this face
                // have been marked for removal
                for e_inv in Explorer::new(s_im, ShapeEnum::Edge, ShapeEnum::Shape)? {
                    if invalid_edges.contains(&e_inv) && !me_removed_loc.contains(&e_inv) {
                        return Ok(true);
                    }
                }
                topexp::map_shapes_of_type(s_im, ShapeEnum::Edge, me_removed)?;
                Ok(false)
            })?;
        }
        Ok(())
    }
}
