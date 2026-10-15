// Rust translation of BRepOffset_BuildOffsetFaces::IntersectAndTrimEdges, GetInvalidEdges,
// UpdateValidEdges, TrimNewIntersectionEdges and IntersectEdges of
// ModelingAlgorithms/TKOffset/BRepOffset/BRepOffset_MakeOffset_1.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The new intersection edges trimmed by the edges and vertices around the invalidity,
//! the invalid splits filtered, the images, origins and edges to avoid updated.
use super::{
    BuildOffsetFaces, ShapeListMap, append_to_list, modified_of, update_images, update_origins,
};
use rustycad_tkbo::bop_algo::{BOPAlgoBOP, BOPAlgoBuilder, Operation};
use rustycad_tkbo::bop_ds::DataStructure;
use rustycad_tkbo::bop_tools;
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::topexp::{self, Explorer};
use rustycad_tkbrep::topods::{self, Compound, Shape, ShapeIterator, Vertex};
use rustycad_tkbrep::toptools::{IndexedDataMapOfShapeListOfShape, IndexedMapOfShape, MapOfShape};
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::ShapeEnum;

/// Standard_NoSuchObject of a missing data map key.
pub(super) fn not_found() -> Failure {
    Failure::with_message("Standard_NoSuchObject", "NCollection_DataMap::Find")
}

/// The data structure of the intersection of a builder (Standard_NullObject without one,
/// a null dereference natively).
pub(super) fn ds_of(builder: &BOPAlgoBuilder) -> Result<&DataStructure, Failure> {
    builder
        .pave_filler()
        .and_then(|pf| pf.data_structure())
        .ok_or_else(|| Failure::with_message("Standard_NullObject", "BOPAlgo_Builder::PDS"))
}

/// `BOPTools_AlgoTools::ComputeVV(theV1, theV2)` with the default fuzzy value: 0 when the
/// vertices are within the sum of their tolerances (plus the confusion), 1 otherwise.
fn compute_vv(v1: &Vertex, v2: &Vertex) -> Result<i32, Failure> {
    let tol_sum =
        BRepTool::vertex_tolerance(v1)? + BRepTool::vertex_tolerance(v2)? + precision::confusion();
    let d2 = BRepTool::pnt(v1)?.square_distance(&BRepTool::pnt(v2)?);
    Ok(if d2 > tol_sum * tol_sum { 1 } else { 0 })
}

/// A builder of the arguments, performed; `None` with errors.
fn performed_builder(args: &List<Shape>) -> Option<BOPAlgoBuilder> {
    let mut gf = BOPAlgoBuilder::new();
    gf.set_arguments(args);
    gf.perform();
    if gf.options().has_errors() {
        return None;
    }
    Some(gf)
}

/// The first modified image of a shape, or the shape itself.
fn first_image(gf: &BOPAlgoBuilder, s: &Shape) -> Result<Shape, Failure> {
    Ok(gf
        .modified(s)?
        .into_iter()
        .next()
        .unwrap_or_else(|| s.clone()))
}

impl BuildOffsetFaces<'_> {
    /// `IntersectAndTrimEdges(...)`: the new intersection edges intersected with the
    /// vertices around the invalidity; the splits through invalid vertices removed, the
    /// common edges trimmed by the other splits; the images and bounds collected.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn intersect_and_trim_edges(
        &self,
        mf_int: &IndexedMapOfShape,
        me_int: &IndexedMapOfShape,
        dmee_trim: &ShapeListMap,
        ms_inv: &IndexedMapOfShape,
        mve: &IndexedMapOfShape,
        verts_to_avoid: &MapOfShape,
        new_verts_to_avoid: &MapOfShape,
        me_check_ext: &MapOfShape,
        use_ss_interfs_art: bool,
        mv_bounds: &mut MapOfShape,
        e_images: &mut ShapeListMap,
    ) -> Result<(), Failure> {
        if me_int.extent() == 0 {
            return Ok(());
        }

        let mut l_args: List<Shape> = List::default();
        let mut m_fence = MapOfShape::default();

        // get vertices from the splits of intersected faces.
        // vertices are taken from the edges close to invalidity
        let mut dmve = IndexedDataMapOfShapeListOfShape::default();
        for i in 1..=mf_int.extent() {
            let f = mf_int.find_key(i)?;
            for e in self.my_faces_to_rebuild.find_from_key(f)?.iter() {
                topexp::map_shapes_and_ancestors(e, ShapeEnum::Vertex, ShapeEnum::Edge, &mut dmve)?;

                for v1 in Explorer::new(e, ShapeEnum::Vertex, ShapeEnum::Shape)? {
                    if !verts_to_avoid.contains(&v1) && mve.contains(&v1) && m_fence.add(v1.clone())
                    {
                        l_args.append(v1);
                    }
                }
            }
        }

        for i in 1..=ms_inv.extent() {
            let s = ms_inv.find_key(i)?;
            // edge case
            if use_ss_interfs_art && let Some(lv) = self.my_ss_interfs_art.seek(s) {
                // Add vertices from intersection info to trim section edges of artificial faces
                for v in lv.iter() {
                    if v.shape_type()? == ShapeEnum::Vertex {
                        l_args.append(v.clone());
                    }
                }
            }

            // vertex case
            if let Some(lve) = dmve.seek(s) {
                for e in lve.iter() {
                    for v1 in Explorer::new(e, ShapeEnum::Vertex, ShapeEnum::Shape)? {
                        if !verts_to_avoid.contains(&v1) && m_fence.add(v1.clone()) {
                            l_args.append(v1);
                        }
                    }
                }
            }
        }

        // bounding vertices of untrimmed edges
        let mut lv_bounds: List<Shape> = List::default();
        // new intersection edges
        let mut le_new: List<Shape> = List::default();
        // get edges to intersect
        let mut le_int: List<Shape> = List::default();
        // Common intersection edges. Should be intersected separately
        let mut lce: List<Shape> = List::default();

        for i in 1..=me_int.extent() {
            let e = me_int.find_key(i)?;
            if me_check_ext.contains(e) {
                // avoid trimming of the intersection edges by additional common edges
                lce.append(e.clone());
                continue;
            }

            if !dmee_trim.is_bound(e) {
                le_new.append(e.clone());
            }

            le_int.append(e.clone());
            l_args.append(e.clone());

            for v in Explorer::new(e, ShapeEnum::Vertex, ShapeEnum::Shape)? {
                lv_bounds.append(v);
            }
        }

        // Intersect Edges
        let Some(gf) = performed_builder(&l_args) else {
            return Ok(());
        };

        // update vertices to avoid with SD vertices
        for v in lv_bounds.iter() {
            mv_bounds.add(first_image(&gf, v)?);
        }

        // find invalid splits of edges
        let mut me_inv = MapOfShape::default();
        get_invalid_edges(new_verts_to_avoid, mv_bounds, &gf, &mut me_inv)?;

        let bb = BRepBuilder::default();
        // get valid splits to intersect with the commons
        let mut ce_im = Compound::default();
        bb.make_compound(&mut ce_im);

        // remove the splits containing vertices from invalid edges
        for e in le_int.iter() {
            let mut le_im = modified_of(&gf, e)?;
            if le_im.is_empty() {
                continue;
            }

            let mut add_err = None;
            le_im.retain(|e_im| {
                if me_inv.contains(e_im) {
                    false
                } else {
                    if let Err(err) = bb.add(&mut ce_im, e_im) {
                        add_err.get_or_insert(err);
                    }
                    true
                }
            });
            if let Some(err) = add_err {
                return Err(err);
            }

            if !le_im.is_empty() {
                if !e_images.is_bound(e) {
                    e_images.bind(e.clone(), List::default());
                }
                let images = e_images.change_seek(e).ok_or_else(not_found)?;
                for x in le_im.iter() {
                    images.append(x.clone());
                }
            }
        }

        if lce.is_empty() {
            return Ok(());
        }

        // trim common edges by other intersection edges
        let mut gfce = BOPAlgoBuilder::new();
        gfce.set_arguments(&lce);
        gfce.add_argument(&ce_im.into_shape());
        gfce.perform();

        if gfce.options().has_errors() {
            return Ok(());
        }

        let ds = ds_of(&gfce)?;
        for e in lce.iter() {
            let le_im = modified_of(&gfce, e)?;
            if le_im.is_empty() {
                continue;
            }

            // check if it's not coincide with some intersection edge
            let mut coincides = false;
            for pb in ds.pave_blocks(ds.index(e))? {
                if ds.is_common_block(pb) {
                    // find with what it is a common
                    let cb = ds.common_block(pb).ok_or_else(not_found)?;
                    let lpbc: Vec<_> = cb.lock().expect("common-block lock").pave_blocks().to_vec();
                    for pbc in &lpbc {
                        let n_or = pbc.lock().expect("pave-block lock").original_edge();
                        let ec = ds.shape(n_or)?;
                        if !me_check_ext.contains(ec) {
                            coincides = true;
                            break;
                        }
                    }
                    if coincides {
                        break;
                    }
                }
            }
            if coincides {
                // avoid creation of unnecessary splits from commons which
                // coincide with intersection edges
                continue;
            }

            // save the images
            if !e_images.is_bound(e) {
                e_images.bind(e.clone(), List::default());
            }
            let images = e_images.change_seek(e).ok_or_else(not_found)?;
            for x in le_im.iter() {
                images.append(x.clone());
            }

            // save bounding vertices
            for v in ShapeIterator::new(e, true, true) {
                mv_bounds.add(first_image(&gfce, &v)?);
            }
        }
        Ok(())
    }

    /// `UpdateValidEdges(...)`: the new intersection edges trimmed, intersected by blocks and
    /// filtered by the bounds of the splits (two stages), then intersected with the bounds;
    /// the images, origins, AsDes and edges to avoid updated.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn update_valid_edges(
        &mut self,
        fle: &IndexedDataMapOfShapeListOfShape,
        oen_edges: &IndexedDataMapOfShapeListOfShape,
        mv_bounds: &MapOfShape,
        me_inv_on_art: &MapOfShape,
        me_check_ext: &mut MapOfShape,
        verts_to_avoid: &mut MapOfShape,
        e_images: &mut ShapeListMap,
        ee_trim: &mut ShapeListMap,
    ) -> Result<(), Failure> {
        // update images and origins of edges, plus update AsDes
        //
        // new edges
        let mut le: List<Shape> = List::default();
        // back connection from edges to faces
        let mut melf = ShapeListMap::default();

        let mut me_tmp = MapOfShape::default();
        for i in 1..=fle.extent() {
            let f = topods::face(fle.find_key(i)?)?;

            for e in fle.find_from_index(i)?.iter() {
                if (me_check_ext.contains(e) || me_tmp.contains(e)) && !e_images.is_bound(e) {
                    me_check_ext.remove(e);
                    me_tmp.add(e.clone());
                    continue;
                }
                if !melf.is_bound(e) {
                    melf.bind(e.clone(), List::default());
                    le.append(e.clone());
                }
                melf.change_seek(e)
                    .ok_or_else(not_found)?
                    .append(f.clone().into_shape());
            }
        }

        if le.is_empty() {
            return Ok(());
        }

        // bounding edges, that are going to be replaced
        let mut meb = MapOfShape::default();

        // new intersection edges
        let mut me_new = MapOfShape::default();
        // map of old vertices
        let mut mv_old = MapOfShape::default();
        // back connection to untrimmed edges
        let mut dme_or = ShapeListMap::default();

        // trim the new intersection edges
        self.trim_new_intersection_edges(
            &le,
            ee_trim,
            mv_bounds,
            me_check_ext,
            e_images,
            &mut meb,
            &mut mv_old,
            &mut me_new,
            &mut dme_or,
            &mut melf,
        )?;

        if e_images.is_empty() {
            // No new splits is preserved
            // update intersection edges and exit
            return self.update_new_intersection_edges(&le, &melf, e_images, ee_trim);
        }

        let bb = BRepBuilder::default();

        // Make connexity blocks of the invalid edges
        // and treat each block separately

        // Compound of all invalid edges to make the blocks
        let mut ce_all = Compound::default();
        bb.make_compound(&mut ce_all);

        for i in 1..=oen_edges.extent() {
            bb.add(&mut ce_all, oen_edges.find_key(i)?)?;
        }

        // Separate the edges into blocks
        let mut l_blocks = Vec::new();
        bop_tools::make_connexity_compounds(
            &ce_all,
            ShapeEnum::Vertex,
            ShapeEnum::Edge,
            &mut l_blocks,
        )?;

        // Perform intersection of the new splits for each block

        // Intersected splits
        let mut m_blocks_sp = IndexedDataMapOfShapeListOfShape::default();

        for block in &l_blocks {
            // Get the list of new edges for the block
            let mut block_le_new: List<Shape> = List::default();
            {
                // Fence map
                let mut me_fence = MapOfShape::default();
                for e in Explorer::new(block, ShapeEnum::Edge, ShapeEnum::Shape)? {
                    for x in oen_edges.find_from_key(&e)?.iter() {
                        if me_fence.add(x.clone()) {
                            block_le_new.append(x.clone());
                        }
                    }
                }
            }

            if block_le_new.is_empty() {
                continue;
            }

            // Get the splits of new edges to intersect
            let mut l_splits: List<Shape> = List::default();

            for e in block_le_new.iter() {
                let Some(le_im) = e_images.seek(e) else {
                    continue;
                };
                for x in le_im.iter() {
                    l_splits.append(x.clone());
                }
            }

            if l_splits.is_empty() {
                continue;
            }

            let ce = if l_splits.len() > 1 {
                // Intersect the new splits among themselves to avoid self-intersections
                let mut ce = Shape::default();
                self.intersect_edges(
                    &l_splits,
                    &block_le_new,
                    mv_bounds,
                    verts_to_avoid,
                    &mut me_new,
                    me_check_ext,
                    e_images,
                    &mut dme_or,
                    &mut melf,
                    &mut ce,
                )?;
                ce
            } else {
                l_splits.first().ok_or_else(not_found)?.clone()
            };

            m_blocks_sp.add(ce, block_le_new);
        }

        // Perform filtering of the edges in two steps:
        // - Check each block separately using localized bounds
        //   taken only from the splits of faces of the current block;
        // - Intersect all splits together and filter the splits by all bounds.

        // FIRST STAGE - separate treatment of the blocks

        // Valid splits to be preserved on the first stage
        let mut me_val = MapOfShape::default();

        // Blocks of valid edges on the first stage
        let mut l_val_blocks: List<Shape> = List::default();

        for i in 1..=m_blocks_sp.extent() {
            let ce = m_blocks_sp.find_key(i)?.clone();
            let block_le_new = m_blocks_sp.find_from_index(i)?;

            // Get all participating faces to get the bounds
            let mut l_faces: List<Shape> = List::default();
            for e in block_le_new.iter() {
                let Some(lf) = melf.seek(e) else {
                    continue;
                };
                for f in lf.iter() {
                    append_to_list(&mut l_faces, f);
                }
            }

            // Localized bounds of the splits of the offset faces
            // to filter the new splits of the current block
            let filter_bounds = self.get_bounds(&l_faces, &meb)?;

            // Filter the splits by bounds
            let mut me_inv_loc = MapOfShape::default();
            self.get_invalid_edges_by_bounds(
                &ce,
                &filter_bounds,
                &mv_old,
                &me_new,
                &dme_or,
                &melf,
                e_images,
                me_check_ext,
                me_inv_on_art,
                verts_to_avoid,
                &mut me_inv_loc,
            )?;

            // Keep only valid edges of the block
            let mut ce_val = Compound::default();
            bb.make_compound(&mut ce_val);

            let mut b_kept = false;

            for e_sp in Explorer::new(&ce, ShapeEnum::Edge, ShapeEnum::Shape)? {
                if !me_inv_loc.contains(&e_sp) && me_val.add(e_sp.clone()) {
                    bb.add(&mut ce_val, &e_sp)?;
                    b_kept = true;
                }
            }

            if b_kept {
                l_val_blocks.append(ce_val.into_shape());
            }
        }

        // Filter the images of edges after the first filtering stage
        let mut splits1 = filter_splits(&le, &me_val, false, e_images)?;

        if l_val_blocks.is_empty() {
            // update intersection edges
            return self.update_new_intersection_edges(&le, &melf, e_images, ee_trim);
        }

        // SECOND STAGE - Filter the remaining splits together

        // Add for intersection already removed new edges using them
        // as markers for other invalid edges
        for i in 1..=m_blocks_sp.extent() {
            let ce = m_blocks_sp.find_key(i)?;
            for e_im in Explorer::new(ce, ShapeEnum::Edge, ShapeEnum::Shape)? {
                if me_new.contains(&e_im) && !me_val.contains(&e_im) {
                    l_val_blocks.append(e_im);
                }
            }
        }

        if l_val_blocks.len() > 1 {
            // intersect the new splits among themselves to avoid self-intersections
            self.intersect_edges(
                &l_val_blocks,
                &le,
                mv_bounds,
                verts_to_avoid,
                &mut me_new,
                me_check_ext,
                e_images,
                &mut dme_or,
                &mut melf,
                &mut splits1,
            )?;
        } else {
            splits1 = l_val_blocks.first().ok_or_else(not_found)?.clone();
        }

        // Get all faces to get the bounds from their splits
        let mut l_faces: List<Shape> = List::default();
        for i in 1..=self.my_of_images.extent() {
            l_faces.append(self.my_of_images.find_key(i)?.clone());
        }

        // Bounds of the splits of the offset faces to filter the new splits
        let filter_bounds = self.get_bounds(&l_faces, &meb)?;

        // Filter the splits by intersection with bounds
        let mut me_inv = MapOfShape::default();
        self.get_invalid_edges_by_bounds(
            &splits1,
            &filter_bounds,
            &mv_old,
            &me_new,
            &dme_or,
            &melf,
            e_images,
            me_check_ext,
            me_inv_on_art,
            verts_to_avoid,
            &mut me_inv,
        )?;

        // Filter the images of edges after the second filtering stage
        // and combine all valid edges into a single compound
        let splits = filter_splits(&le, &me_inv, true, e_images)?;

        // get bounds to update
        // we need to update the edges of all the affected faces
        let mut lf: List<Shape> = List::default();
        // prepare the vertices from new splits of edges
        let mut mv_sp = IndexedMapOfShape::default();
        topexp::map_shapes_of_type(&splits, ShapeEnum::Vertex, &mut mv_sp)?;

        for i in 1..=self.my_of_images.extent() {
            let f = self.my_of_images.find_key(i)?;
            if fle.contains(f) {
                lf.append(f.clone());
                continue;
            }

            // check the splits of faces to have vertices from splits
            let mut found = false;
            'splits: for f_im in self.my_of_images.find_from_index(i)?.iter() {
                for v in Explorer::new(f_im, ShapeEnum::Vertex, ShapeEnum::Shape)? {
                    if mv_sp.contains(&v) {
                        found = true;
                        break 'splits;
                    }
                }
            }

            if found {
                lf.append(f.clone());
            }
        }

        // get bounds from splits of faces of aLF
        let mut la_valid: List<Shape> = List::default();
        let mut la_bounds: List<Shape> = List::default();
        let bounds = self.get_bounds_to_update(&lf, &meb, &mut la_bounds, &mut la_valid)?;

        // Intersect valid splits with bounds and update both
        let mut gf = BOPAlgoBuilder::new();
        gf.add_argument(&bounds);
        gf.add_argument(&splits);
        gf.perform();

        // update splits
        update_images(&le, e_images, &gf, &mut self.my_modified_edges)?;

        // update new intersection edges
        self.update_new_intersection_edges(&le, &melf, e_images, ee_trim)?;

        // update bounds
        update_images(
            &la_valid,
            &mut self.my_oe_images,
            &gf,
            &mut self.my_modified_edges,
        )?;
        update_origins(&la_bounds, &mut self.my_oe_origins, &gf)?;
        update_origins(&la_bounds, self.edges_origins(), &gf)?;
        self.update_intersected_edges(&la_bounds, &gf)?;

        // update the EdgesToAvoid with the splits
        let mut new_edges = IndexedMapOfShape::default();
        if let Some(splits_im) = gf.images().seek(&splits) {
            for x in splits_im.iter() {
                topexp::map_shapes_of_type(x, ShapeEnum::Edge, &mut new_edges)?;
            }
        }

        let mut inside_edges = Compound::default();
        bb.make_compound(&mut inside_edges);
        for i_e in 1..=self.my_inside_edges.extent() {
            bb.add(&mut inside_edges, self.my_inside_edges.find_key(i_e)?)?;
        }

        // Rebuild the map of edges to avoid, using the intersection results
        let mut me_avoid = IndexedMapOfShape::default();
        let mut ce_avoid = Compound::default();
        bb.make_compound(&mut ce_avoid);
        // GF's data structure
        let ds = ds_of(&gf)?;

        for i in 1..=self.my_edges_to_avoid.extent() {
            let e = self.my_edges_to_avoid.find_key(i)?;
            let le_im = gf.modified(e)?;

            // Only untouched and fully coinciding edges should be kept in the avoid map
            let mut b_keep = le_im.is_empty();
            if le_im.len() == 1 && e.is_same(&le_im[0]) {
                let lpb = ds.pave_blocks(ds.index(e))?;
                if lpb.len() == 1
                    && let Some(cb) = ds.common_block(&lpb[0])
                {
                    let lpbcb: Vec<_> =
                        cb.lock().expect("common-block lock").pave_blocks().to_vec();
                    let mut split = false;
                    for pb in &lpbcb {
                        let n_or = pb.lock().expect("pave-block lock").original_edge();
                        if ds.pave_blocks(n_or)?.len() > 1 {
                            split = true;
                            break;
                        }
                    }
                    b_keep = !split;
                }
            }

            if b_keep {
                // keep the original edge
                me_avoid.add(e.clone());
                continue;
            }

            for e_im in &le_im {
                if !new_edges.contains(e_im) {
                    bb.add(&mut ce_avoid, e_im)?;
                }
            }
        }

        let mut is_cut = false;
        if ce_avoid.nb_children() > 0 {
            // Perform intersection with the small subset of the edges to make
            // it possible to use the inside edges for building new splits.
            let mut bop = BOPAlgoBOP::new();
            bop.add_argument(&ce_avoid.clone().into_shape());
            bop.add_tool(&inside_edges.into_shape());
            bop.set_operation(Operation::Cut);
            bop.perform();
            is_cut = !bop.options().has_errors();

            if is_cut {
                for x in ShapeIterator::new(&ce_avoid, true, true) {
                    if !bop.is_deleted(&x)? {
                        me_avoid.add(x);
                    }
                }
            }
        }

        if !is_cut {
            topexp::map_shapes_of_type(&ce_avoid, ShapeEnum::Edge, &mut me_avoid)?;
        }

        self.my_edges_to_avoid = me_avoid;
        Ok(())
    }

    /// `TrimNewIntersectionEdges(...)`: the images of the new intersection edges fused with
    /// their bounding vertices, the splits through bounding vertices removed; the trimmed
    /// edges replaced and the new splits recorded.
    #[allow(clippy::too_many_arguments)]
    fn trim_new_intersection_edges(
        &mut self,
        le: &List<Shape>,
        ee_trim: &ShapeListMap,
        mv_bounds: &MapOfShape,
        me_check_ext: &mut MapOfShape,
        e_images: &mut ShapeListMap,
        meb: &mut MapOfShape,
        mv_old: &mut MapOfShape,
        me_new: &mut MapOfShape,
        dme_or: &mut ShapeListMap,
        melf: &mut ShapeListMap,
    ) -> Result<(), Failure> {
        for e in le.iter() {
            let b_check_ext = me_check_ext.remove(e);

            let b_old = ee_trim.is_bound(e);
            if let Some(let_) = ee_trim.seek(e) {
                for et in let_.iter() {
                    meb.add(et.clone());
                    for v in Explorer::new(et, ShapeEnum::Vertex, ShapeEnum::Shape)? {
                        mv_old.add(v);
                    }
                }
            }

            let Some(le_im) = e_images.seek(e).cloned() else {
                continue;
            };
            if le_im.is_empty() {
                e_images.unbind(e);
                continue;
            }

            let mut ce_im = Shape::default();
            let mut mev_bounds = MapOfShape::default();

            if le_im.len() > 1 {
                let mut mv = IndexedMapOfShape::default();
                // fuse these parts
                let mut gfe = BOPAlgoBuilder::new();
                for e_im in le_im.iter() {
                    gfe.add_argument(e_im);
                    topexp::map_shapes_of_type(e_im, ShapeEnum::Vertex, &mut mv)?;
                }

                // add two bounding vertices of this edge to the operation
                let (v1, v2) = topexp::vertices(&topods::edge(e)?, false);

                gfe.add_argument(&v1.clone().into_shape());
                gfe.add_argument(&v2.clone().into_shape());
                mv.add(v1.clone().into_shape());
                mv.add(v2.clone().into_shape());

                gfe.perform();
                if !gfe.options().has_errors() {
                    // get images of bounding vertices to remove splits containing them
                    // in case some of the bounding edges has been interfered
                    // during operation it is necessary to update their images as well
                    for i_v in 1..=mv.extent() {
                        let v = mv.find_key(i_v)?;
                        if mv_bounds.contains(v) || v.is_same(&v1) || v.is_same(&v2) {
                            mev_bounds.add(first_image(&gfe, v)?);
                        }
                    }

                    ce_im = gfe.shape().clone();
                }
            } else {
                ce_im = le_im.first().ok_or_else(not_found)?.clone();
            }

            let mut le_im_new: List<Shape> = List::default();

            if !ce_im.is_null() {
                for e_im in Explorer::new(&ce_im, ShapeEnum::Edge, ShapeEnum::Shape)? {
                    // check the split not to contain bounding vertices
                    let mut bounded = false;
                    for v in ShapeIterator::new(&e_im, true, true) {
                        if mev_bounds.contains(&v) || mv_bounds.contains(&v) {
                            bounded = true;
                            break;
                        }
                    }

                    if !bounded {
                        le_im_new.append(e_im.clone());

                        let mut l = List::default();
                        l.append(e.clone());
                        dme_or.bind(e_im, l);
                    }
                }
            }

            if le_im_new.is_empty() {
                e_images.unbind(e);
            } else {
                let lfe = melf.seek(e).ok_or_else(not_found)?.clone();
                for e_im in le_im_new.iter() {
                    if !melf.is_bound(e_im) {
                        melf.bind(e_im.clone(), List::default());
                    }
                    let lfe_im = melf.change_seek(e_im).ok_or_else(not_found)?;
                    for f in lfe.iter() {
                        append_to_list(lfe_im, f);
                    }

                    if b_check_ext {
                        me_check_ext.add(e_im.clone());
                    } else if !b_old {
                        me_new.add(e_im.clone());
                    }
                }
                *e_images.change_seek(e).ok_or_else(not_found)? = le_im_new;
            }
        }
        Ok(())
    }

    /// `IntersectEdges(...)`: the splits intersected among themselves (or copied on errors),
    /// the images, new edges, extended checks, faces and origins updated, the splits through
    /// vertices to avoid removed.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn intersect_edges(
        &mut self,
        la: &List<Shape>,
        le: &List<Shape>,
        mv_bounds: &MapOfShape,
        verts_to_avoid: &MapOfShape,
        me_new: &mut MapOfShape,
        me_check_ext: &mut MapOfShape,
        e_images: &mut ShapeListMap,
        dme_or: &mut ShapeListMap,
        melf: &mut ShapeListMap,
        splits: &mut Shape,
    ) -> Result<(), Failure> {
        let Some(gfa) = performed_builder(la) else {
            // just copy input to the result
            let mut sp = Compound::default();
            let bb = BRepBuilder::default();
            bb.make_compound(&mut sp);
            for e in la.iter() {
                bb.add(&mut sp, e)?;
            }
            *splits = sp.into_shape();
            return Ok(());
        };

        update_images(le, e_images, &gfa, &mut self.my_modified_edges)?;

        // compound of valid splits
        *splits = gfa.shape().clone();

        // prepare list of edges to update
        let mut le_input: List<Shape> = List::default();
        for a in la.iter() {
            for e in Explorer::new(a, ShapeEnum::Edge, ShapeEnum::Shape)? {
                le_input.append(e);
            }
        }

        // update new edges
        for e in le_input.iter() {
            if !me_new.contains(e) {
                continue;
            }

            let le_im = gfa.modified(e)?;
            if le_im.is_empty() {
                continue;
            }

            me_new.remove(e);
            for x in &le_im {
                me_new.add(x.clone());
            }
        }

        // update edges after intersection for extended checking
        for e in le_input.iter() {
            let le_im = gfa.modified(e)?;
            if le_im.is_empty() {
                continue;
            }

            if me_check_ext.contains(e) {
                for x in &le_im {
                    me_check_ext.add(x.clone());
                }
                me_check_ext.remove(e);
            }

            let lfe = melf.seek(e).ok_or_else(not_found)?.clone();
            for e_im in &le_im {
                if !melf.is_bound(e_im) {
                    melf.bind(e_im.clone(), List::default());
                }
                let lfe_im = melf.change_seek(e_im).ok_or_else(not_found)?;
                for f in lfe.iter() {
                    append_to_list(lfe_im, f);
                }
            }
        }

        let mut me_inv = MapOfShape::default();
        get_invalid_edges(verts_to_avoid, mv_bounds, &gfa, &mut me_inv)?;
        if me_inv.extent() > 0 {
            // update shape
            let mut sp = Compound::default();
            let bb = BRepBuilder::default();
            bb.make_compound(&mut sp);
            for e in Explorer::new(splits, ShapeEnum::Edge, ShapeEnum::Shape)? {
                if !me_inv.contains(&e) {
                    bb.add(&mut sp, &e)?;
                }
            }
            *splits = sp.into_shape();
        }

        // update origins
        update_origins(&le_input, dme_or, &gfa)
    }
}

/// `GetInvalidEdges(theVertsToAvoid, theMVBounds, theGF, theMEInv)`: the edges of the result
/// through the new (not bounding) vertices coinciding with vertices to avoid.
fn get_invalid_edges(
    verts_to_avoid: &MapOfShape,
    mv_bounds: &MapOfShape,
    gf: &BOPAlgoBuilder,
    me_inv: &mut MapOfShape,
) -> Result<(), Failure> {
    if verts_to_avoid.is_empty() {
        return Ok(());
    }

    // get vertices created with intersection edges
    let res = gf.shape();
    let mut dmve = IndexedDataMapOfShapeListOfShape::default();
    topexp::map_shapes_and_ancestors(res, ShapeEnum::Vertex, ShapeEnum::Edge, &mut dmve)?;

    let ds = ds_of(gf)?;

    // find invalid splits of edges
    // check if the vertex is invalid:
    // a. it may be the vertex SD with the vertices to avoid
    // b. or it may be the vertex which is created by the intersection
    //    of only existing edges, i.e. no new intersection edges goes
    //    through this vertex
    for i in 1..=dmve.extent() {
        let v = topods::vertex(dmve.find_key(i)?)?;
        if mv_bounds.contains(&v) {
            continue;
        }

        let n_v = ds.index(&v);
        if n_v >= 0 && !ds.is_new_shape(n_v) {
            continue;
        }

        let mut coincides = false;
        for v_inv in verts_to_avoid.iter() {
            if compute_vv(&v, &topods::vertex(v_inv)?)? == 0 {
                coincides = true;
                break;
            }
        }

        if coincides {
            for e in dmve.find_from_index(i)?.iter() {
                me_inv.add(e.clone());
            }
        }
    }
    Ok(())
}

/// `FilterSplits(theLE, theMEFilter, theIsInv, theEImages, theSplits)`: the images of the
/// edges filtered by the map (removed when in it for invalid ones, when not in it
/// otherwise), the kept ones returned in a compound.
pub(super) fn filter_splits(
    le: &List<Shape>,
    me_filter: &MapOfShape,
    is_inv: bool,
    e_images: &mut ShapeListMap,
) -> Result<Shape, Failure> {
    let bb = BRepBuilder::default();
    let mut splits = Compound::default();
    bb.make_compound(&mut splits);
    let mut m_fence = MapOfShape::default();

    for e in le.iter() {
        let Some(le_im) = e_images.change_seek(e) else {
            continue;
        };

        let mut add_err = None;
        le_im.retain(|e_im| {
            if me_filter.contains(e_im) == is_inv {
                return false;
            }
            if m_fence.add(e_im.clone())
                && let Err(err) = bb.add(&mut splits, e_im)
            {
                add_err.get_or_insert(err);
            }
            true
        });
        if let Some(err) = add_err {
            return Err(err);
        }

        if le_im.is_empty() {
            e_images.unbind(e);
        }
    }
    Ok(splits.into_shape())
}
