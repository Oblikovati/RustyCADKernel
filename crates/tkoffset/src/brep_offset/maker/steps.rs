// Rust translation of BRepOffset_MakeOffset::Intersection3D, Intersection2D, MakeLoops,
// MakeFaces and the static UpdateInitOffset of
// ModelingAlgorithms/TKOffset/BRepOffset/BRepOffset_MakeOffset.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The intersection steps of the offset algorithm: the 3d and 2d intersections of the
//! offset faces, their loops and the rebuilt faces.
use super::BRepOffsetMakeOffset;
use super::statics::{self, not_bound};
use crate::brep_offset::{
    BRepOffsetError, BRepOffsetInter2d, BRepOffsetInter3d, BRepOffsetTool, CoincidentVertices,
};
use rustycad_tkbool::brep_algo::BRepAlgoImage;
use rustycad_tkbrep::topods::{self, Shape};
use rustycad_tkbrep::toptools::IndexedMapOfShape;
use rustycad_tkernel::message::{ProgressRange, ProgressScope};
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::geom_abs::JoinType;

/// `UpdateInitOffset(myInitOffset, myImageOffset, myOffsetShape, theShapeType)`: the
/// images of the roots replaced by the last images of their last images, filtered on
/// the shapes of the type in the result.
pub(super) fn update_init_offset(
    my_init_offset: &mut BRepAlgoImage,
    my_image_offset: &BRepAlgoImage,
    my_offset_shape: &Shape,
    the_shape_type: ShapeEnum,
) -> Result<(), Failure> {
    let mut niof = BRepAlgoImage::new();
    let roots = my_init_offset.roots().to_vec();
    for r in &roots {
        niof.set_root(r);
    }
    for si in &roots {
        let mut li = Vec::new();
        for o1 in my_init_offset.last_image(si) {
            li.extend(my_image_offset.last_image(&o1));
        }
        niof.bind_many(si, &li)?;
    }
    niof.filter(my_offset_shape, the_shape_type)?;
    *my_init_offset = niof;
    Ok(())
}

impl BRepOffsetMakeOffset {
    /// `Intersection3D(Inter, Range)`: the offset faces intersected with the caps (arc
    /// joins), then completely (oriented for intersection joins) or between neighbours;
    /// nothing in the complete intersection of planar solids.
    pub(super) fn intersection3d(
        &mut self,
        inter: &mut BRepOffsetInter3d,
        the_range: &ProgressRange,
    ) -> Result<(), Failure> {
        let a_ps = ProgressScope::unnamed(
            the_range,
            if !self.my_faces.is_empty() && self.my_join == JoinType::Arc {
                2.
            } else {
                1.
            },
            false,
        );
        // In the complete intersection mode, implemented for planar solids only, the
        // faces are intersected in BuildShellsCompleteInter.
        if self.my_inter
            && self.my_join == JoinType::Intersection
            && self.my_is_planar
            && !self.my_thickening
            && self.my_faces.is_empty()
            && statics::is_solid(&self.my_shape)?
        {
            return Ok(());
        }
        // The list of the parallel faces created.
        let mut offset_faces = List::default();
        statics::make_list(&mut offset_faces, &self.my_init_offset_face, &self.my_faces);
        if !self.my_faces.is_empty() {
            // Temporary: InSide should account for the concavity or convexity of the
            // edges between the caps and the part.
            let in_side = self.my_offset < 0.;
            if self.my_join == JoinType::Arc {
                inter.context_int_by_arc(
                    &self.my_faces,
                    in_side,
                    &self.my_analyse,
                    &self.my_init_offset_face,
                    &mut self.my_init_offset_edge,
                    &a_ps.next(1.),
                )?;
            }
        }
        if self.my_inter {
            // Complete.
            inter.complet_int(&offset_faces, &self.my_init_offset_face, &a_ps.next(1.))?;
            if !a_ps.more() {
                self.my_error = BRepOffsetError::UserBreak;
                return Ok(());
            }
            if self.my_join == JoinType::Intersection {
                let new_edges = inter.new_edges().clone();
                BRepOffsetTool::correct_orientation(
                    &self.my_face_comp,
                    &new_edges,
                    self.as_des(),
                    &self.my_init_offset_face,
                    self.my_offset,
                )?;
            }
        } else {
            // Only between neighbour faces.
            inter.connex_int_by_arc(
                &offset_faces,
                &self.my_face_comp,
                &self.my_analyse,
                &self.my_init_offset_face,
                &a_ps.next(1.),
            )?;
            if !a_ps.more() {
                self.my_error = BRepOffsetError::UserBreak;
                return Ok(());
            }
        }
        Ok(())
    }

    /// `Intersection2D(Modif, NewEdges, Range)`: the 2d intersections of the edges of
    /// the touched faces, the coinciding vertices fused.
    pub(super) fn intersection2d(
        &mut self,
        modif: &IndexedMapOfShape,
        new_edges: &IndexedMapOfShape,
        the_range: &ProgressRange,
    ) -> Result<(), Failure> {
        let mut a_dmvv = CoincidentVertices::default();
        let a_ps =
            ProgressScope::with_literal(the_range, "Intersection 2D", modif.extent() as f64, false);
        for i in 1..=modif.extent() {
            if !a_ps.more() {
                self.my_error = BRepOffsetError::UserBreak;
                return Ok(());
            }
            let f = topods::face(modif.find_key(i)?)?;
            BRepOffsetInter2d::compute(
                self.as_des(),
                &f,
                new_edges,
                self.my_tol,
                &self.my_edge_int_edges,
                &mut a_dmvv,
                &a_ps.next(1.),
            )?;
        }
        let as_des_h = self.my_as_des.clone();
        BRepOffsetInter2d::fuse_vertices(
            &a_dmvv,
            as_des_h.get().expect("AsDes"),
            &mut self.my_image_vv,
        )?;
        Ok(())
    }

    /// `MakeLoops(Modif, Range)`: the touched offset faces cut into loops, then the caps.
    pub(super) fn make_loops(
        &mut self,
        modif: &mut IndexedMapOfShape,
        the_range: &ProgressRange,
    ) -> Result<(), Failure> {
        let mut lf = List::default();
        // The unwinding of the parallel faces modified.
        for i in 1..=modif.extent() {
            let s = modif.find_key(i)?;
            if !self.my_faces.contains(s) {
                lf.append(s.clone());
            }
        }
        let a_ps =
            ProgressScope::unnamed(the_range, (lf.len() + self.my_faces.extent()) as f64, false);
        let as_des_h = self.my_as_des.clone();
        let as_des = as_des_h.get().expect("AsDes");
        if self.my_join == JoinType::Intersection && self.my_inter && self.my_is_planar {
            let mut image = std::mem::take(&mut self.my_image_offset);
            let r = self.build_splits_of_trimmed_faces(
                &lf,
                as_des,
                &mut image,
                &a_ps.next(lf.len() as f64),
            );
            self.my_image_offset = image;
            r?;
        } else {
            self.my_make_loops.build(
                &lf,
                as_des,
                &mut self.my_image_offset,
                &mut self.my_image_vv,
                &a_ps.next(lf.len() as f64),
            )?;
        }
        if !a_ps.more() {
            self.my_error = BRepOffsetError::UserBreak;
            return Ok(());
        }
        // The unwinding of the caps.
        let mut lc = List::default();
        for i in 1..=self.my_faces.extent() {
            lc.append(self.my_faces.find_key(i)?.clone());
        }
        let in_side = self.my_offset <= 0.;
        self.my_make_loops.build_on_context(
            &lc,
            &self.my_analyse,
            as_des,
            &mut self.my_image_offset,
            in_side,
            &a_ps.next(lc.len() as f64),
        )?;
        Ok(())
    }

    /// `MakeFaces(Modif, Range)`: the rebuilding of the topologically unchanged faces
    /// sharing rebuilt edges.
    pub(super) fn make_faces(
        &mut self,
        _modif: &mut IndexedMapOfShape,
        the_range: &ProgressRange,
    ) -> Result<(), Failure> {
        let mut lof = List::default();
        // The loop on all parallel faces.
        for r in self.my_init_offset_face.roots().to_vec() {
            let f = self
                .my_init_offset_face
                .image(&r)
                .first()
                .cloned()
                .ok_or_else(not_bound)?;
            let _ = topods::face(&f)?;
            if !self.my_image_offset.has_image(&f) {
                lof.append(f);
            }
        }
        let a_ps = ProgressScope::unnamed(the_range, 1., false);
        let as_des_h = self.my_as_des.clone();
        let as_des = as_des_h.get().expect("AsDes");
        if self.my_join == JoinType::Intersection && self.my_inter && self.my_is_planar {
            let mut image = std::mem::take(&mut self.my_image_offset);
            let r = self.build_splits_of_trimmed_faces(&lof, as_des, &mut image, &a_ps.next(1.));
            self.my_image_offset = image;
            r?;
        } else {
            self.my_make_loops.build_faces(
                &lof,
                as_des,
                &mut self.my_image_offset,
                &a_ps.next(1.),
            )?;
        }
        if !a_ps.more() {
            self.my_error = BRepOffsetError::UserBreak;
        }
        Ok(())
    }
}
