// Rust translation of the one-limit Perform methods of
// ModelingAlgorithms/TKFeat/BRepFeat/BRepFeat_MakeDPrism.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The drafted prisms of a height, up to a shape, up to the end, through all, and of a
//! height towards a shape.
use super::BRepFeatMakeDPrism;
use super::statics::{height_max, sens_of_prism, test_curve};
use crate::brep_feat::{PerfSelection, tool};
use crate::errors::{construction_error, null_object};
use crate::loc_ope::{CSIntersector, DPrism};
use rustycad_tkbo::brep_algo_api::BRepAlgoAPICut;
use rustycad_tkbool::brep_algo;
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{self, Compound, Shape};
use rustycad_tkernel::ncollection::Sequence;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::GeomCurve;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tktopalgo::brep_lib;

/// The test curve reversed when the prism goes backward (`C->Reversed()`).
pub(super) fn oriented_curve(
    c: &Handle<dyn GeomCurve>,
    sens: i32,
) -> Result<Handle<dyn GeomCurve>, Failure> {
    if sens != -1 {
        return Ok(c.clone());
    }
    c.get()
        .ok_or_else(|| {
            null_object(
                "BRepFeat_MakeDPrism",
                "no test curve of a profile not planar",
            )
        })?
        .reversed()
}

/// Standard_ConstructionError unless `s` has a face.
pub(super) fn require_face(s: &Shape) -> Result<(), Failure> {
    if !Explorer::new(s, ShapeEnum::Face, ShapeEnum::Shape)?.more() {
        return Err(construction_error(
            "BRepFeat_MakeDPrism::Perform",
            "the limiting shape has no face",
        ));
    }
    Ok(())
}

impl BRepFeatMakeDPrism {
    /// `Perform(Height)`: the drafted prism of height `height` (its sides measured along
    /// the draft) fused with or cut from the basis; `dprism.perform_height(10.)?`.
    pub fn perform_height(&mut self, height: f64) -> Result<(), Failure> {
        let f = &mut self.form;
        f.s_from.nullify();
        f.shape_from_valid();
        f.s_until.nullify();
        f.shape_until_valid();
        f.glued_f.clear(false);
        f.perf_selection = PerfSelection::NoSelection;
        f.perf_selection_valid();
        let theheight = height / self.angle.cos();
        let the_dprism = DPrism::with_height(&self.pbase, theheight, self.angle)?;
        self.take_dprism(&the_dprism)?;
        if self.invalid_first_shape(&the_dprism)? {
            return Ok(());
        }
        // management of gluing faces
        self.form.glued_faces_valid();
        if !self.form.glued_f.is_empty() {
            // case gluing
            self.form.just_gluer = true;
            self.global_with(&the_dprism)?;
        }
        // if there is no gluing -> call topological operations
        if !self.form.just_gluer {
            let g_shape = self.form.g_shape.clone();
            self.form.finish_with(&g_shape, false)?;
        }
        Ok(())
    }

    /// The shape Until set (extended when it is an infinite face): whether it was.
    pub(super) fn set_until(
        &mut self,
        until: &Shape,
        selection: PerfSelection,
    ) -> Result<bool, Failure> {
        let f = &mut self.form;
        f.glued_f.clear(false);
        f.perf_selection = selection;
        f.perf_selection_valid();
        f.s_from.nullify();
        f.shape_from_valid();
        f.s_until = until.clone();
        let trf = f.transform_shape_fu(1)?;
        f.shape_until_valid();
        Ok(trf)
    }

    /// `Perform(Until)`: the drafted prism up to the shape `until`, then the invalid new
    /// edges made same parameter; `dprism.perform_until(&face)?`.
    pub fn perform_until(&mut self, until: &Shape) -> Result<(), Failure> {
        if until.is_null() {
            return Err(construction_error(
                "BRepFeat_MakeDPrism::Perform",
                "null limiting shape",
            ));
        }
        require_face(until)?;
        let trf = self.set_until(until, PerfSelection::SelectionU)?;
        let c = test_curve(&self.pbase)?;
        let sens = sens_of_prism(&c, &self.form.s_until)?;
        let f = &self.form;
        let height = (sens as f64) * height_max(&f.sbase, &f.skface, &f.s_from, &f.s_until)?;
        let the_dprism = DPrism::with_height(&self.pbase, height, self.angle)?;
        if !self.limited_or_support(&the_dprism, trf, &c, sens, true)? {
            return Ok(());
        }
        self.fix_new_edges()
    }

    /// The end of the drafted prisms up to a shape: the topological reconstruction for a
    /// face Until (false on an invalid base), else the drafted prism cut by the solid of
    /// the support then fused with or cut from the basis (`up_to` for `Perform(Until)`,
    /// which keeps the history of the cut and its first solid).
    fn limited_or_support(
        &mut self,
        the_dprism: &DPrism,
        trf: bool,
        c: &Handle<dyn GeomCurve>,
        sens: i32,
        up_to: bool,
    ) -> Result<bool, Failure> {
        if !trf {
            self.take_dprism(the_dprism)?;
            if self.invalid_first_shape(the_dprism)? {
                return Ok(false);
            }
            self.form.glued_faces_valid();
            self.global_with(the_dprism)?;
            return Ok(true);
        }
        let vrai = self.map_dprism(the_dprism)?;
        let mut scur = Sequence::default();
        scur.append(oriented_curve(c, sens)?);
        let mut asi = CSIntersector::with_shape(&self.form.s_until);
        asi.perform_curves(&scur)?;
        if asi.is_done() && asi.nb_points(1)? >= 1 {
            let k = if self.form.fuse { 1 } else { asi.nb_points(1)? };
            let or = asi.point(1, k)?.orientation();
            let f_until = asi.point(1, 1)?.face().clone();
            let b = BRepBuilder::default();
            let mut comp = Compound::default();
            b.make_compound(&mut comp);
            let mut comp = comp.into_shape();
            let s = tool(&self.form.s_until, &f_until, or)?;
            if !s.is_null() {
                b.add(&mut comp, &s)?;
            }
            let tr_p = BRepAlgoAPICut::from_shapes(&vrai, &comp)?;
            let cutsh = if up_to {
                self.form
                    .update_descendants_bop(&tr_p, tr_p.shape(), false)?;
                Explorer::new(tr_p.shape(), ShapeEnum::Solid, ShapeEnum::Shape)?
                    .current()
                    .clone()
            } else {
                tr_p.shape().clone()
            };
            self.form.finish_with(&cutsh, false)?;
        }
        Ok(true)
    }

    /// The new edges not valid made same range and same parameter again.
    fn fix_new_edges(&mut self) -> Result<(), Failure> {
        let b = BRepBuilder::default();
        for e in self.form.new_edges.iter() {
            let ledg = topods::edge(e)?;
            if !brep_algo::is_valid(&ledg)? {
                b.same_range(&ledg, false)?;
                b.same_parameter(&ledg, false)?;
                brep_lib::same_parameter_edge(&ledg, BRepTool::edge_tolerance(&ledg)?)?;
            }
        }
        Ok(())
    }

    /// `PerformUntilEnd()`: the drafted prism through the basis along the normal, by the
    /// topological reconstruction; `dprism.perform_until_end()?`.
    pub fn perform_until_end(&mut self) -> Result<(), Failure> {
        let f = &mut self.form;
        f.perf_selection = PerfSelection::SelectionSh;
        f.perf_selection_valid();
        f.glued_f.clear(false);
        f.s_until.nullify();
        f.shape_until_valid();
        f.s_from.nullify();
        f.shape_from_valid();
        let height = height_max(&f.sbase, &f.skface, &f.s_from, &f.s_until)?;
        let the_dprism = DPrism::with_height(&self.pbase, height, self.angle)?;
        self.take_dprism(&the_dprism)?;
        self.form.glued_faces_valid();
        self.global_with(&the_dprism)
    }

    /// `PerformThruAll()`: the drafted prism through all of the basis on both sides of
    /// the face; `dprism.perform_thru_all()?`.
    pub fn perform_thru_all(&mut self) -> Result<(), Failure> {
        let f = &mut self.form;
        f.s_until.nullify();
        f.shape_until_valid();
        f.s_from.nullify();
        f.shape_from_valid();
        f.perf_selection = if f.fuse {
            PerfSelection::SelectionSh
        } else {
            PerfSelection::NoSelection
        };
        f.perf_selection_valid();
        f.glued_f.clear(false);
        f.glued_faces_valid();
        let height = height_max(&f.sbase, &f.skface, &f.s_from, &f.s_until)?;
        let the_dprism = DPrism::with_heights(&self.pbase, height, height, self.angle)?;
        let vrai = self.take_dprism(&the_dprism)?;
        if !self.form.fuse {
            return self.form.cut_when_done(&vrai);
        }
        self.global_with(&the_dprism)
    }

    /// `PerformUntilHeight(Until, Height)`: the drafted prism of height `height` towards
    /// the shape `until` (natively the null and zero cases perform and then go on);
    /// `dprism.perform_until_height(&face, 5.)?`.
    pub fn perform_until_height(&mut self, until: &Shape, height: f64) -> Result<(), Failure> {
        if until.is_null() {
            self.perform_height(height)?;
        }
        if height == 0. {
            self.perform_until(until)?;
        }
        require_face(until)?;
        let trf = self.set_until(until, PerfSelection::NoSelection)?;
        let c = test_curve(&self.pbase)?;
        let sens = sens_of_prism(&c, &self.form.s_until)?;
        let the_dprism = DPrism::with_height(&self.pbase, (sens as f64) * height, self.angle)?;
        self.limited_or_support(&the_dprism, trf, &c, sens, false)?;
        Ok(())
    }
}
