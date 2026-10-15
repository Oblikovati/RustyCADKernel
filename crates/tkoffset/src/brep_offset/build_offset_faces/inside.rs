// Rust translation of the statics buildPairs and buildIntersectionPairs and of
// BRepOffset_BuildOffsetFaces::RemoveInsideFaces of
// ModelingAlgorithms/TKOffset/BRepOffset/BRepOffset_MakeOffset_1.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The splits inside the volumes built from all splits (MakerVolume) removed, and the
//! intersection pairs of the offset faces built from the volumes.
use super::{BuildOffsetFaces, ShapeMapMap, ShapeShapeMap, find_shape, take_modified};
use crate::brep_offset::BRepOffsetTool;
use rustycad_tkbo::bop_algo::{BOPAlgoBuilder, BOPAlgoMakerVolume};
use rustycad_tkbo::bop_tools;
use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::topexp::{self, Explorer};
use rustycad_tkbrep::topods::{self, Compound, Shape};
use rustycad_tkbrep::toptools::ShapeMapHasher;
use rustycad_tkbrep::toptools::{IndexedDataMapOfShapeListOfShape, IndexedMapOfShape, MapOfShape};
use rustycad_tkernel::ncollection::{DataMap, List};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};

/// The intersection pairs of the offset faces, per invalid offset face.
pub(super) type IntersectionPairs = DataMap<Shape, ShapeMapMap, ShapeMapHasher>;

/// Standard_NoSuchObject of a missing data map key.
fn not_found() -> Failure {
    Failure::with_message("Standard_NoSuchObject", "NCollection_DataMap::Find")
}

/// `buildPairs(theSMap, theIntPairs)`: each shape of the map paired with all the others.
fn build_pairs(s_map: &IndexedMapOfShape, int_pairs: &mut ShapeMapMap) -> Result<(), Failure> {
    let nb_s = s_map.extent();
    if nb_s < 2 {
        return Ok(());
    }
    for it1 in 1..=nb_s {
        let s = s_map.find_key(it1)?;
        if !int_pairs.is_bound(s) {
            int_pairs.bind(s.clone(), MapOfShape::default());
        }
    }

    for it1 in 1..=nb_s {
        let s1 = s_map.find_key(it1)?.clone();
        for it2 in (it1 + 1)..=nb_s {
            let s2 = s_map.find_key(it2)?.clone();
            int_pairs
                .change_seek(&s1)
                .ok_or_else(not_found)?
                .add(s2.clone());
            int_pairs
                .change_seek(&s2)
                .ok_or_else(not_found)?
                .add(s1.clone());
        }
    }
    Ok(())
}

/// The offset faces of the origins of a split of the builder.
fn offset_origins_of(
    f_connected: &Shape,
    origins: &super::ShapeListMap,
    f_origins: &ShapeShapeMap,
    mf_inter: &mut IndexedMapOfShape,
) {
    let mut lf_or: List<Shape> = List::default();
    take_modified(f_connected, origins, &mut lf_or, None);
    for f_or in lf_or.iter() {
        if let Some(f) = f_origins.seek(f_or) {
            mf_inter.add(f.clone());
        }
    }
}

/// `buildIntersectionPairs(...)`: the pairs of offset faces connected through the splits
/// of the intersection result (the not removed ones by vertices, the removed ones by the
/// edges shared by the faces connected to them).
fn build_intersection_pairs(
    of_images: &IndexedDataMapOfShapeListOfShape,
    invalid_faces: &IndexedDataMapOfShapeListOfShape,
    builder: &BOPAlgoBuilder,
    mf_removed: &MapOfShape,
    f_origins: &ShapeShapeMap,
    int_pairs: &mut IntersectionPairs,
) -> Result<(), Failure> {
    let c_type = ShapeEnum::Vertex;
    // Build connection map from vertices to faces
    let mut dmvf = IndexedDataMapOfShapeListOfShape::default();
    topexp::map_shapes_and_ancestors(builder.shape(), c_type, ShapeEnum::Face, &mut dmvf)?;

    let images = builder.images();
    let origins = builder.origins();

    // Find all faces connected to the not removed faces and build intersection pairs among them.
    // For removed faces intersect only those connected to each other.

    for i_f in 1..=invalid_faces.extent() {
        let f_inv = invalid_faces.find_key(i_f)?;

        let mut cf = Compound::default();
        let mut cf_rem = Compound::default();
        BRepBuilder::default().make_compound(&mut cf);
        BRepBuilder::default().make_compound(&mut cf_rem);

        for i_c in 0..2 {
            let lf = if i_c == 0 {
                invalid_faces.find_from_index(i_f)?
            } else {
                of_images.find_from_key(f_inv)?
            };

            for f in lf.iter() {
                let mut lf_im: List<Shape> = List::default();
                take_modified(f, images, &mut lf_im, None);

                for f_im in lf_im.iter() {
                    if mf_removed.contains(f_im) {
                        BRepBuilder::default().add(&mut cf_rem, f_im)?;
                    } else {
                        BRepBuilder::default().add(&mut cf, f_im)?;
                    }
                }
            }
        }

        let mut lcb = Vec::new();
        bop_tools::make_connexity_compounds(&cf, ShapeEnum::Edge, ShapeEnum::Face, &mut lcb)?;

        if lcb.is_empty() {
            continue;
        }

        // (Bound: a new map replaces any previous one)
        int_pairs.bind(f_inv.clone(), ShapeMapMap::default());
        let f_inter_map = int_pairs.change_seek(f_inv).ok_or_else(not_found)?;

        // build pairs for not removed faces
        for cb in &lcb {
            let mut mf_inter = IndexedMapOfShape::default();
            for cs in Explorer::new(cb, c_type, ShapeEnum::Shape)? {
                let Some(lfv) = dmvf.seek(&cs) else {
                    continue;
                };
                for f_connected in lfv.iter() {
                    offset_origins_of(f_connected, origins, f_origins, &mut mf_inter);
                }
            }

            // build intersection pairs
            build_pairs(&mf_inter, f_inter_map)?;
        }

        lcb.clear();
        bop_tools::make_connexity_compounds(&cf_rem, ShapeEnum::Edge, ShapeEnum::Face, &mut lcb)?;

        if lcb.is_empty() {
            continue;
        }

        for cb in &lcb {
            let mut dmef = IndexedDataMapOfShapeListOfShape::default();
            for cs in Explorer::new(cb, c_type, ShapeEnum::Shape)? {
                let Some(lfv) = dmvf.seek(&cs) else {
                    continue;
                };
                for f_connected in lfv.iter() {
                    topexp::map_shapes_and_ancestors(
                        f_connected,
                        ShapeEnum::Edge,
                        ShapeEnum::Face,
                        &mut dmef,
                    )?;
                }
            }

            for i_e in 1..=dmef.extent() {
                let lf_connected = dmef.find_from_index(i_e)?;
                if lf_connected.len() < 2 {
                    continue;
                }

                let mut mf_inter = IndexedMapOfShape::default();
                for f_connected in lf_connected.iter() {
                    offset_origins_of(f_connected, origins, f_origins, &mut mf_inter);
                }

                build_pairs(&mf_inter, f_inter_map)?;
            }
        }
    }
    Ok(())
}

impl BuildOffsetFaces<'_> {
    /// `RemoveInsideFaces(...)`: the splits (with the invalid ones) intersected into
    /// volumes; the splits shared by two solids, internal, in holes or in solids made only
    /// of invalid splits removed; the removed edges not on the solid boundaries made inside
    /// edges; the intersection pairs built from the result.
    pub(super) fn remove_inside_faces(
        &mut self,
        inverted_faces: &List<Shape>,
        mf_to_check_int: &IndexedMapOfShape,
        mf_inv_in_hole: &IndexedMapOfShape,
        f_holes: &Shape,
        me_removed: &mut IndexedMapOfShape,
        me_inside: &mut IndexedMapOfShape,
    ) -> Result<(), Failure> {
        let mut ls: List<Shape> = List::default();
        let mut m_fence = MapOfShape::default();
        let mut mf_inv = IndexedMapOfShape::default();
        let mut dmf_im_f = ShapeShapeMap::default();

        for i in 1..=self.my_of_images.extent() {
            let f = self.my_of_images.find_key(i)?.clone();
            // to avoid intersection of the splits of the same
            // offset faces among themselves make compound of the
            // splits and use it as one argument
            let mut cf_imi = Compound::default();
            BRepBuilder::default().make_compound(&mut cf_imi);

            for j in 0..2 {
                let p_lf_sp = if j == 0 {
                    self.my_invalid_faces.seek(&f)
                } else {
                    Some(self.my_of_images.find_from_index(i)?)
                };
                let Some(lf_sp) = p_lf_sp else {
                    continue;
                };

                for f_im in lf_sp.iter() {
                    if m_fence.add(f_im.clone()) {
                        BRepBuilder::default().add(&mut cf_imi, f_im)?;
                        dmf_im_f.bind(f_im.clone(), f.clone());
                        if j == 0 {
                            mf_inv.add(f_im.clone());
                        }
                    }
                }
            }

            ls.append(cf_imi.into_shape());
        }

        // to make the solids more complete add for intersection also the faces
        // consisting only of invalid edges and not included into splits
        for i in 1..=mf_to_check_int.extent() {
            let f_sp = mf_to_check_int.find_key(i)?;
            if m_fence.add(f_sp.clone()) {
                ls.append(f_sp.clone());
            }
        }

        let mut mv = BOPAlgoMakerVolume::new();
        mv.set_arguments(&ls);
        mv.set_intersect(true);
        mv.perform();
        if mv.options().has_errors() {
            return Ok(());
        }

        // get shapes connection for using in the rebuilding process
        // for the cases in which some of the intersection left undetected
        self.shapes_connections(&dmf_im_f, &mv)?;

        // find faces to remove
        let sols = mv.shape().clone();

        let mut dmfs = IndexedDataMapOfShapeListOfShape::default();
        topexp::map_shapes_and_ancestors(&sols, ShapeEnum::Face, ShapeEnum::Solid, &mut dmfs)?;

        if dmfs.extent() == 0 {
            return Ok(());
        }

        // To use the created solids for classifications, firstly, it is necessary
        // to check them on validity - the created solids should be complete,
        // i.e. all faces should be included.
        let mut mf_to_rem = MapOfShape::default();
        // Check completeness
        if mv.has_deleted() {
            let mut me_holes = IndexedMapOfShape::default();
            topexp::map_shapes_of_type(f_holes, ShapeEnum::Edge, &mut me_holes)?;

            // Map edges of the solids to check the connectivity
            // of the removed invalid splits
            let mut me_sols = IndexedMapOfShape::default();
            topexp::map_shapes_of_type(&sols, ShapeEnum::Edge, &mut me_sols)?;

            // perform additional check on faces
            for i in 1..=self.my_of_images.extent() {
                let lf_im = self.my_of_images.find_from_index(i)?;
                if lf_im.is_empty() {
                    continue;
                }

                let f = self.my_of_images.find_key(i)?;
                let b_invalid = self.my_invalid_faces.contains(f);
                // For invalid faces it is allowed to be at least connected
                // to the solids, otherwise the solids are considered as broken
                let mut b_connected = false;

                let mut b_face_kept = false;
                for f_im in lf_im.iter() {
                    if !mv.is_deleted(f_im)? {
                        b_face_kept = true;
                        continue;
                    }

                    for e in Explorer::new(f_im, ShapeEnum::Edge, ShapeEnum::Shape)? {
                        if me_holes.contains(&e) {
                            b_face_kept = true;
                            mf_to_rem.add(f_im.clone());
                            break;
                        }
                        if !b_face_kept && b_invalid && !b_connected {
                            b_connected = me_sols.contains(&e);
                        }
                    }
                }

                if !b_face_kept && !b_connected {
                    return Ok(());
                }
            }
        }

        let mut me_boundary = IndexedMapOfShape::default();
        for i in 1..=dmfs.extent() {
            let f_im = dmfs.find_key(i)?;
            let l_sol = dmfs.find_from_index(i)?;
            if l_sol.len() > 1 {
                mf_to_rem.add(f_im.clone());
            } else if f_im.orientation() != Orientation::Internal {
                topexp::map_shapes_of_type(f_im, ShapeEnum::Edge, &mut me_boundary)?;
            }
        }

        // Tool for getting the splits of faces
        let mv_ims = mv.images();

        // update invalid faces with images
        let nb = mf_inv.extent();
        for i in 1..=nb {
            let f_inv = mf_inv.find_key(i)?.clone();
            take_modified(&f_inv, mv_ims, &mut mf_inv, None);
        }

        // Take into account the faces invalid by inverted edges
        for f in inverted_faces.iter() {
            take_modified(f, mv_ims, &mut mf_inv, None);
        }

        // check if the invalid faces inside the holes are really invalid:
        // check its normal direction - if it has changed relatively the
        // original face the offset face is invalid and should be kept for rebuilding
        for i in 1..=mf_inv_in_hole.extent() {
            let f_inv = mf_inv_in_hole.find_key(i)?;
            let mut lf_inv_im: List<Shape> = mv.modified(f_inv)?.into_iter().collect();
            if lf_inv_im.is_empty() {
                lf_inv_im.append(f_inv.clone());
            }

            let Some(f_offset) = dmf_im_f.seek(f_inv) else {
                continue;
            };
            for f_inv_im in lf_inv_im.iter() {
                let Some(l_sols) = dmfs.seek(f_inv_im) else {
                    continue;
                };
                if l_sols.len() != 1 {
                    continue;
                }

                let f_sol = l_sols.first().ok_or_else(not_found)?;

                let Some(fx) = find_shape(f_inv_im, f_sol, None)? else {
                    continue;
                };

                if BRepOffsetTool::check_planes_normals(
                    &topods::face(&fx)?,
                    &topods::face(f_offset)?,
                    1.0e-8,
                )? {
                    // the normal direction has not changed, thus the face can be removed
                    mf_to_rem.add(f_inv_im.clone());
                }
            }
        }

        let mut solids = Compound::default();
        BRepBuilder::default().make_compound(&mut solids);
        let mut mf_keep = MapOfShape::default();

        for sol in Explorer::new(&sols, ShapeEnum::Solid, ShapeEnum::Shape)? {
            let mut b_all_inv = true;
            let mut b_all_removed = true;

            for fs in Explorer::new(&sol, ShapeEnum::Face, ShapeEnum::Shape)? {
                if fs.orientation() == Orientation::Internal {
                    mf_to_rem.add(fs);
                    continue;
                }

                if mf_to_rem.contains(&fs) {
                    continue;
                }

                b_all_removed = false;
                b_all_inv &= mf_inv.contains(&fs);
            }

            if b_all_inv && !b_all_removed {
                // remove invalid faces but keep those that have already been marked for removal
                for fs in Explorer::new(&sol, ShapeEnum::Face, ShapeEnum::Shape)? {
                    if mf_to_rem.contains(&fs) {
                        if !mf_keep.add(fs.clone()) {
                            mf_keep.remove(&fs);
                        }
                    } else {
                        mf_to_rem.add(fs);
                    }
                }
            } else {
                BRepBuilder::default().add(&mut solids, &sol)?;
                // (the native mySolids shares the TShape of the compound filled further)
                self.my_solids = solids.clone().into_shape();
            }
        }

        for f in mf_keep.iter() {
            mf_to_rem.remove(f);
        }

        // Remove the invalid hanging parts external to the solids
        self.remove_hanging_parts(&mv, &dmf_im_f, &mf_inv, &mut mf_to_rem)?;

        // Remove newly found internal and hanging faces
        self.remove_valid_splits(&mf_to_rem, &mv, me_removed)?;
        self.remove_invalid_splits(&mf_to_rem, &mv, me_removed)?;

        // Get inside faces from the removed ones comparing them with boundary edges
        me_inside.clear(true);
        for i in 1..=me_removed.extent() {
            let e = me_removed.find_key(i)?;
            if !me_boundary.contains(e) {
                me_inside.add(e.clone());
            }
        }

        // build all possible intersection pairs basing on the intersection results
        // taking into account removed faces.
        if mf_to_rem.extent() > 0 {
            build_intersection_pairs(
                &self.my_of_images,
                &self.my_invalid_faces,
                &mv,
                &mf_to_rem,
                &dmf_im_f,
                &mut self.my_intersection_pairs,
            )?;
        }
        Ok(())
    }
}
