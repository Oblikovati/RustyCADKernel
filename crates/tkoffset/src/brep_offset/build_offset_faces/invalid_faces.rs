// Rust translation of BRepOffset_BuildOffsetFaces::FindInvalidFaces and
// FindFacesInsideHoleWires of
// ModelingAlgorithms/TKOffset/BRepOffset/BRepOffset_MakeOffset_1.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The invalid splits of an offset face, and the splits filling the hole wires of the
//! original face.
use super::{BuildOffsetFaces, ShapeIndexedMapMap, ShapeListMap, ShapeMapMap};
use rustycad_tkbo::bop_algo::BuilderFace;
use rustycad_tkbo::bop_tools::algo_tools3d;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::brep_tools;
use rustycad_tkbrep::topexp::{self, Explorer};
use rustycad_tkbrep::topods::{self, Face, Shape, ShapeIterator};
use rustycad_tkbrep::toptools::{IndexedDataMapOfShapeListOfShape, IndexedMapOfShape, MapOfShape};
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkmath::gp::{Point2, Point3};

impl BuildOffsetFaces<'_> {
    /// `FindInvalidFaces(...)`: the splits of an offset face classified by the edges of
    /// their outer wires: invalid splits appended to `inv_faces`, the splits made only of
    /// invalid neutral edges (or inside holes) removed, the inverted ones recorded.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn find_invalid_faces(
        &mut self,
        lf_images: &mut List<Shape>,
        dmfmve: &ShapeMapMap,
        dmfmie: &ShapeIndexedMapMap,
        me_neutral: &MapOfShape,
        edges_invalid_by_vertex: &MapOfShape,
        edges_valid_by_vertex: &MapOfShape,
        mf_holes: &MapOfShape,
        mf_inv_in_hole: &mut IndexedMapOfShape,
        inv_faces: &mut List<Shape>,
        inverted_faces: &mut List<Shape>,
    ) -> Result<(), Failure> {
        // The face should be considered as invalid in the following cases:
        // 1. It has been reverted, i.e. at least two not connected edges
        //    have changed orientation (i.e. invalid). In this case all edges,
        //    should be invalid for that face, because edges have also been reverted;
        // 2. All checked edges of the face are invalid for this face;
        // The face should be removed from the splits in the following cases:
        // 1. All checked edges of the face are invalid for this one, but valid for
        //    some other face in this list of splits.
        // The face will be kept in the following cases:
        // 1. Some of the edges are valid for this face.
        let b_treat_inverted_as_invalid = lf_images.len() == 1;

        // neutral edges to remove
        let mut me_n_rem = IndexedMapOfShape::default();
        // faces for post treat
        let mut lf_pt: List<Shape> = List::default();

        let mut dmef = IndexedDataMapOfShapeListOfShape::default();
        for f_im in lf_images.iter() {
            topexp::map_shapes_and_ancestors(f_im, ShapeEnum::Edge, ShapeEnum::Face, &mut dmef)?;
        }

        let this = &*self;
        lf_images.try_retain(|f_im| -> Result<bool, Failure> {
            // valid edges for this split
            let p_mve = dmfmve.seek(f_im);
            // invalid edges for this split
            let p_mie = dmfmie.seek(f_im);

            let mut b_has_valid = false;
            let mut b_all_valid = true;
            let mut b_all_invalid = true;
            let mut b_has_really_invalid = false;
            let mut b_all_inv_neutral = true;
            let mut b_is_invalid_by_inverted = true;
            let mut b_has_inverted = false;
            let mut nb_checked = 0;

            let w_im = brep_tools::outer_wire(&topods::face(f_im)?)?;
            for e_im in Explorer::new(&w_im, ShapeEnum::Edge, ShapeEnum::Shape)? {
                let b_valid = this.my_valid_edges.contains(&e_im);
                let mut b_invalid = this.my_invalid_edges.contains(&e_im);
                let b_neutral = me_neutral.contains(&e_im);

                if !b_valid && !b_invalid && !b_neutral {
                    // edge has not been checked for some reason
                    continue;
                }

                // skip not-boundary edges originated from vertex
                if (edges_invalid_by_vertex.contains(&e_im)
                    || edges_valid_by_vertex.contains(&e_im))
                    && dmef.find_from_key(&e_im)?.len() != 1
                {
                    continue;
                }

                nb_checked += 1;

                let b_invalid_loc = p_mie.is_some_and(|m| m.contains(&e_im));
                b_has_really_invalid = b_invalid
                    && b_invalid_loc
                    && !b_valid
                    && !edges_invalid_by_vertex.contains(&e_im);
                if b_has_really_invalid {
                    break;
                }

                let b_valid_loc = p_mve.is_some_and(|m| m.contains(&e_im));
                let b_inverted = this.my_inverted_edges.contains(&e_im);
                if !b_invalid && !b_invalid_loc && b_treat_inverted_as_invalid {
                    b_invalid = b_inverted;
                }

                if b_valid_loc && b_neutral {
                    b_has_valid = true;
                }

                b_all_valid &= b_valid_loc;
                b_all_invalid &= b_invalid || b_invalid_loc;
                b_all_inv_neutral &= b_all_invalid && b_neutral;
                b_is_invalid_by_inverted &= b_invalid_loc || b_inverted;
                b_has_inverted |= b_inverted;
            }

            if nb_checked == 0 {
                return Ok(true);
            }

            if !b_has_really_invalid && (b_all_inv_neutral && !b_has_valid) && nb_checked > 1 {
                if b_has_inverted {
                    // The part seems to be filled due to overlapping of parts rather than
                    // due to multi-connection of faces. No need to remove the part.
                    return Ok(true);
                }
                // remove edges from neutral
                topexp::map_shapes_of_type(f_im, ShapeEnum::Edge, &mut me_n_rem)?;
                // remove face
                return Ok(false);
            }

            if b_has_really_invalid
                || (b_all_invalid
                    && !(b_has_valid || b_all_valid)
                    && (!b_all_inv_neutral || nb_checked != 1))
            {
                inv_faces.append(f_im.clone());
                if mf_holes.contains(f_im) {
                    mf_inv_in_hole.add(f_im.clone());
                }
                return Ok(true);
            }

            if mf_holes.contains(f_im) {
                // remove edges from neutral
                topexp::map_shapes_of_type(f_im, ShapeEnum::Edge, &mut me_n_rem)?;
                // remove face
                return Ok(false);
            }

            if b_is_invalid_by_inverted && !(b_has_valid || b_all_valid) {
                // The face contains only the inverted and locally invalid edges
                inverted_faces.append(f_im.clone());
            }

            if !b_all_inv_neutral {
                lf_pt.append(f_im.clone());
            } else {
                // remove edges from neutral
                topexp::map_shapes_of_type(f_im, ShapeEnum::Edge, &mut me_n_rem)?;
            }
            Ok(true)
        })?;

        if lf_pt.is_empty() || me_n_rem.is_empty() {
            return Ok(());
        }

        // check the splits once more
        for f_im in lf_pt.iter() {
            // valid edges for this split
            let p_mve = dmfmve.seek(f_im);

            let mut b_has_valid = false;
            let mut b_all_valid = true;
            let mut b_all_invalid = true;

            let w_im = brep_tools::outer_wire(&topods::face(f_im)?)?;
            for e_im in Explorer::new(&w_im, ShapeEnum::Edge, ShapeEnum::Shape)? {
                let mut b_invalid = self.my_invalid_edges.contains(&e_im);
                let b_neutral = me_neutral.contains(&e_im) && !me_n_rem.contains(&e_im);
                let b_valid_loc = p_mve.is_some_and(|m| m.contains(&e_im));

                if !b_invalid && b_treat_inverted_as_invalid {
                    b_invalid = self.my_inverted_edges.contains(&e_im);
                }

                if b_valid_loc && b_neutral {
                    b_has_valid = true;
                }

                b_all_valid = b_all_valid && b_valid_loc;
                b_all_invalid = b_all_invalid && b_invalid;
            }

            if b_all_invalid && !b_has_valid && !b_all_valid {
                inv_faces.append(f_im.clone());
            }
        }
        Ok(())
    }

    /// `FindFacesInsideHoleWires(...)`: the splits of the offset face located inside the
    /// faces built on the images of the hole wires of the original face, except those whose
    /// removal would leave uncovered holes.
    pub(super) fn find_faces_inside_hole_wires(
        &mut self,
        f_origin: &Face,
        f_offset: &Face,
        lf_images: &List<Shape>,
        dme_or_le_im: &ShapeListMap,
        ef_map: &IndexedDataMapOfShapeListOfShape,
        mf_holes: &mut MapOfShape,
    ) -> Result<(), Failure> {
        if lf_images.is_empty() {
            return Ok(());
        }

        // find all hole wires in the original face
        let mut lhole_wires: List<Shape> = List::default();
        let outer_wire = brep_tools::outer_wire(f_origin)?;
        for hole_wire in Explorer::new(f_origin, ShapeEnum::Wire, ShapeEnum::Shape)? {
            if !hole_wire.is_same(&outer_wire) && hole_wire.orientation() != Orientation::Internal {
                lhole_wires.append(hole_wire);
            }
        }

        if lhole_wires.is_empty() {
            // no holes in the face
            return Ok(());
        }

        if !self.my_f_new_holes.is_bound(f_origin) {
            self.my_f_new_holes
                .bind(f_origin.clone().into_shape(), List::default());
        }
        let need_fill = self
            .my_f_new_holes
            .seek(f_origin)
            .is_some_and(List::is_empty);
        if need_fill {
            // find the faces representing holes in the images of the faces:
            // 1. for each original hole wire try to build its image
            // 2. build the new planar face from the images
            //
            // map vertices and edges of the splits
            let mut me_splits = IndexedMapOfShape::default();
            for f in lf_images.iter() {
                topexp::map_shapes_of_type(f, ShapeEnum::Edge, &mut me_splits)?;
            }

            let mut new_holes: List<Shape> = List::default();
            for hole_wire in lhole_wires.iter() {
                // find images of all edges of the original wire
                let mut me_im_wire = IndexedMapOfShape::default();
                for e_or in ShapeIterator::new(hole_wire, true, true) {
                    let Some(le_im) = dme_or_le_im.seek(&e_or) else {
                        continue;
                    };
                    for e_im in le_im.iter() {
                        if me_splits.contains(e_im) {
                            me_im_wire.add(e_im.clone());
                        }
                    }
                }

                if me_im_wire.is_empty() {
                    continue;
                }

                // build new planar face using these edges
                let mut le = List::default();
                for i in 1..=me_im_wire.extent() {
                    let e = me_im_wire.find_key(i)?;
                    le.append(e.oriented(Orientation::Forward));
                    le.append(e.oriented(Orientation::Reversed));
                }

                let mut bf = BuilderFace::new();
                bf.set_face(&topods::face(&f_offset.oriented(Orientation::Forward))?);
                bf.set_shapes(&le);
                if bf.perform().is_err() {
                    continue;
                }

                let lf_new = bf.areas().clone();
                if lf_new.is_empty() {
                    continue;
                }

                // check if outer edges in the new faces are not inverted
                // because the inverted edges mean that the hole has been
                // filled during offset and there will be no faces to remove
                let mut dmef_new = IndexedDataMapOfShapeListOfShape::default();
                for f in lf_new.iter() {
                    topexp::map_shapes_and_ancestors(
                        f,
                        ShapeEnum::Edge,
                        ShapeEnum::Face,
                        &mut dmef_new,
                    )?;
                }

                let mut inverted_outer = false;
                for i in 1..=dmef_new.extent() {
                    if dmef_new.find_from_index(i)?.len() == 1
                        && self.my_inverted_edges.contains(dmef_new.find_key(i)?)
                    {
                        inverted_outer = true;
                        break;
                    }
                }
                if inverted_outer {
                    continue;
                }

                for f in lf_new.iter() {
                    new_holes.append(f.clone());
                }
            }
            if let Some(holes) = self.my_f_new_holes.change_seek(f_origin) {
                for f in new_holes.iter() {
                    holes.append(f.clone());
                }
            }
        }
        let lf_new_holes = self
            .my_f_new_holes
            .seek(f_origin)
            .cloned()
            .unwrap_or_default();

        // Build Edge-Face map for splits of current offset face
        let mut ef_splits_map = IndexedDataMapOfShapeListOfShape::default();
        // Build Edge-Face map for holes
        let mut ef_holes_map = IndexedDataMapOfShapeListOfShape::default();

        // among the splits of the offset face find those that are
        // located inside the hole faces
        for f_im in lf_images.iter() {
            let f_im_face = topods::face(f_im)?;
            topexp::map_shapes_and_ancestors(
                f_im,
                ShapeEnum::Edge,
                ShapeEnum::Face,
                &mut ef_splits_map,
            )?;
            // get the point inside the face and classify it relatively hole faces
            let mut p3d = Point3::default();
            let mut p2d = Point2::default();
            let i_err =
                algo_tools3d::point_in_face(&f_im_face, &mut p3d, &mut p2d, self.context()?)?;
            if i_err != 0 {
                continue;
            }

            let tol = BRepTool::face_tolerance(&f_im_face)?;

            for f_new in lf_new_holes.iter() {
                if self
                    .context()?
                    .is_valid_point_for_face(p3d, &topods::face(f_new)?, tol)?
                {
                    // the face is classified as IN
                    mf_holes.add(f_im.clone());
                    topexp::map_shapes_and_ancestors(
                        f_im,
                        ShapeEnum::Edge,
                        ShapeEnum::Face,
                        &mut ef_holes_map,
                    )?;
                    break;
                }
            }
        }

        // Out of all found holes find those which cannot be removed
        // by checking their connectivity to splits of other offset faces.
        // These are the faces, which will create uncovered holes if removed.
        for i in 1..=ef_holes_map.extent() {
            let edge = ef_holes_map.find_key(i)?;
            let lf_holes = ef_holes_map.find_from_index(i)?;
            // Check if the edge is outer for holes
            if lf_holes.len() != 1 {
                continue;
            }

            let f_hole = lf_holes.first().ok_or_else(empty)?.clone();
            if !mf_holes.contains(&f_hole) {
                // Already removed
                continue;
            }

            // Check if the edge is not outer for splits
            if ef_splits_map.find_from_key(edge)?.len() == 1 {
                continue;
            }

            // Check if edge is only connected to splits of the current offset face
            if ef_map.find_from_key(edge)?.len() == 2 {
                // Avoid removal of the hole from the splits
                mf_holes.remove(&f_hole);
            }
        }
        Ok(())
    }
}

/// Standard_NoSuchObject of the first item of an empty list.
fn empty() -> Failure {
    Failure::with_message("Standard_NoSuchObject", "NCollection_List::First")
}
