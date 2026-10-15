// Rust translation of the Perform methods of one limit of
// ModelingAlgorithms/TKFeat/BRepFeat/BRepFeat_MakePrism.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The prisms of a length, up to a shape, up to the end, through all, and of a length
//! towards a shape.
use super::BRepFeatMakePrism;
use super::statics::{height_max, maj_map, sens_of_prism, test_curve};
use crate::brep_feat::common::to_fuse;
use crate::brep_feat::form::global_perform;
use crate::brep_feat::{PerfSelection, is_inside, tool};
use crate::errors::construction_error;
use crate::loc_ope::{CSIntersector, Prism};
use rustycad_tkbo::brep_algo_api::BRepAlgoAPICut;
use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{self, Compound, Face, Shape};
use rustycad_tkernel::ncollection::Sequence;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::GeomCurve;
use rustycad_tkg3d::top_abs::{self, ShapeEnum};
use rustycad_tkmath::gp::Vector3;

impl BRepFeatMakePrism {
    /// The direction scaled by `k` (the native `k * myDir`, `k` evaluated first).
    pub(super) fn along(&self, k: f64) -> Vector3 {
        Vector3::from(self.dir) * k
    }

    /// The history of the prism mapped (`MajMap`); its shape returned.
    pub(super) fn map_prism(&mut self, the_prism: &Prism) -> Result<Shape, Failure> {
        let f = &mut self.form;
        maj_map(
            &self.pbase,
            the_prism,
            &mut f.map,
            &mut f.f_shape,
            &mut f.l_shape,
        )?;
        Ok(the_prism.shape()?.clone())
    }

    /// The prism made the generated shape of the feature, its history mapped.
    pub(super) fn take_prism(&mut self, the_prism: &Prism) -> Result<Shape, Failure> {
        let vrai_prism = self.map_prism(the_prism)?;
        self.form.g_shape = vrai_prism.clone();
        self.form.generated_shape_valid();
        Ok(vrai_prism)
    }

    /// The curves of the prism kept, then the topological reconstruction.
    pub(super) fn global_with(&mut self, the_prism: &Prism) -> Result<(), Failure> {
        the_prism.curves(&mut self.curves)?;
        self.b_curve = the_prism.baryc_curve()?;
        global_perform(self)
    }

    /// `Perform(Length)`: the prism of length `length` fused with or cut from the basis;
    /// `prism.perform_length(10.)?`.
    pub fn perform_length(&mut self, length: f64) -> Result<(), Failure> {
        let f = &mut self.form;
        f.s_from.nullify();
        f.shape_from_valid();
        f.s_until.nullify();
        f.shape_until_valid();
        f.glued_f.clear(false);
        f.perf_selection = PerfSelection::NoSelection;
        f.perf_selection_valid();
        // construction of prism of height Length
        let the_prism = Prism::with_vector(&self.pbase, self.along(length))?;
        let vrai_prism = self.take_prism(&the_prism)?;
        self.detect_top_face(&vrai_prism)?;
        // management of faces of gluing given by the user
        self.form.glued_faces_valid();
        if !self.form.glued_f.is_empty() {
            // case gluing
            self.form.just_gluer = true;
            self.global_with(&the_prism)?; // topological reconstruction
        }
        // if there is no gluing -> call of ope topo
        if !self.form.just_gluer {
            let g_shape = self.form.g_shape.clone();
            self.form.finish_by_boolean(&g_shape)?;
        }
        Ok(())
    }

    /// try to detect the faces of gluing in case if the top of the prism is tangent to
    /// the initial shape (natively without effect: the faces found are not kept).
    fn detect_top_face(&self, vrai_prism: &Shape) -> Result<(), Failure> {
        let f = &self.form;
        if f.skface.is_null() && self.slface.is_empty() {
            return Ok(());
        }
        let mut fface = Face::default();
        if f.l_shape.shape_type()? == ShapeEnum::Wire {
            'faces: for ex1 in Explorer::new(vrai_prism, ShapeEnum::Face, ShapeEnum::Shape)? {
                for ex2 in Explorer::new(&ex1, ShapeEnum::Wire, ShapeEnum::Shape)? {
                    if ex2.is_same(&f.l_shape) {
                        fface = topods::face(&ex1)?;
                        break 'faces;
                    }
                }
            }
        }
        for exp in Explorer::new(&f.sbase, ShapeEnum::Face, ShapeEnum::Shape)? {
            let ff = topods::face(&exp)?;
            if to_fuse(&ff, &fface)? && !fface.is_same(&self.pbase) && is_inside(&ff, &fface)? {
                break;
            }
        }
        Ok(())
    }

    /// `Perform(Until)`: the prism up to the shape `until`; `prism.perform_until(&face)?`.
    pub fn perform_until(&mut self, until: &Shape) -> Result<(), Failure> {
        if until.is_null() || !Explorer::new(until, ShapeEnum::Face, ShapeEnum::Shape)?.more() {
            return Err(construction_error(
                "BRepFeat_MakePrism::Perform",
                "no face to stop at",
            ));
        }
        let f = &mut self.form;
        f.glued_f.clear(false);
        f.perf_selection = PerfSelection::SelectionU;
        f.perf_selection_valid();
        f.s_from.nullify();
        f.shape_from_valid();
        f.s_until = until.clone();
        let trf = f.transform_shape_fu(1)?;
        f.shape_until_valid();
        let c = test_curve(&self.pbase, self.dir)?;
        let sens = sens_of_prism(&c, &self.form.s_until)?;
        let height = height_max(
            &self.form.sbase,
            &self.form.skface,
            &self.form.s_from,
            &self.form.s_until,
        )?;
        // construction of long prism
        let the_prism = Prism::with_vector(&self.pbase, self.along(((2 * sens) as f64) * height))?;
        self.until_or_support(&the_prism, trf, &c, sens, true)
    }

    /// The end of the prisms up to a shape: the topological reconstruction for a face
    /// Until, else the prism cut by the solid of the support (its first solid when
    /// `first_solid`), then fused with or cut from the basis.
    fn until_or_support(
        &mut self,
        the_prism: &Prism,
        trf: bool,
        c: &Handle<dyn GeomCurve>,
        sens: i32,
        first_solid: bool,
    ) -> Result<(), Failure> {
        if !trf {
            self.take_prism(the_prism)?;
            self.form.glued_faces_valid();
            return self.global_with(the_prism);
        }
        // until support -> passage to topological operations
        let vrai_prism = self.map_prism(the_prism)?;
        let mut scur = Sequence::default();
        scur.append(c.clone());
        // direction of the prism depending on Until
        let mut asi = CSIntersector::with_shape(&self.form.s_until);
        asi.perform_curves(&scur)?;
        if !asi.is_done() || asi.nb_points(1)? < 1 {
            return Ok(());
        }
        let k = if self.form.fuse { 1 } else { asi.nb_points(1)? };
        let mut or = asi.point(1, k)?.orientation();
        if sens == -1 {
            or = top_abs::reverse(or);
        }
        let f_until = asi.point(1, 1)?.face().clone();
        let b = BRepBuilder::default();
        let mut comp = Compound::default();
        b.make_compound(&mut comp);
        let mut comp = comp.into_shape();
        let s = tool(&self.form.s_until, &f_until, or)?;
        if !s.is_null() {
            b.add(&mut comp, &s)?;
        }
        let tr_p = BRepAlgoAPICut::from_shapes(&vrai_prism, &comp)?;
        self.form
            .update_descendants_bop(&tr_p, tr_p.shape(), false)?;
        let cutsh = if first_solid {
            Explorer::new(tr_p.shape(), ShapeEnum::Solid, ShapeEnum::Shape)?
                .current()
                .clone()
        } else {
            tr_p.shape().clone()
        };
        self.form.finish_by_boolean(&cutsh)
    }

    /// `PerformUntilEnd()`: the prism through the basis in the direction (a cut, or the
    /// reconstruction of a boss); `prism.perform_until_end()?`.
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
        let the_prism = Prism::with_vector(&self.pbase, self.along(2. * height))?;
        self.through(&the_prism)
    }

    /// The end of the prisms through the basis: the cut of the basis without fusion, else
    /// the topological reconstruction.
    fn through(&mut self, the_prism: &Prism) -> Result<(), Failure> {
        let vrai_prism = self.take_prism(the_prism)?;
        self.form.glued_faces_valid();
        if !self.form.fuse {
            return self.form.cut_when_done(&vrai_prism);
        }
        self.global_with(the_prism)
    }

    /// `PerformThruAll()`: the prism through all of the basis on both sides of the
    /// profile; `prism.perform_thru_all()?`.
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
        let v = self.along(3. * height);
        let vtra = self.along((-3. * height) / 2.);
        let the_prism = Prism::with_translation(&self.pbase, v, vtra)?;
        self.through(&the_prism)
    }

    /// `PerformUntilHeight(Until, Length)`: the prism of length `length` towards the
    /// shape `until` (natively the null and zero cases perform and then go on);
    /// `prism.perform_until_height(&face, 5.)?`.
    pub fn perform_until_height(&mut self, until: &Shape, length: f64) -> Result<(), Failure> {
        if until.is_null() {
            self.perform_length(length)?;
        }
        if length == 0. {
            self.perform_until(until)?;
        }
        if !Explorer::new(until, ShapeEnum::Face, ShapeEnum::Shape)?.more() {
            return Err(construction_error(
                "BRepFeat_MakePrism::PerformUntilHeight",
                "no face to stop at",
            ));
        }
        let f = &mut self.form;
        f.glued_f.clear(false);
        f.perf_selection = PerfSelection::NoSelection;
        f.perf_selection_valid();
        f.s_from.nullify();
        f.shape_from_valid();
        f.s_until = until.clone();
        let trf = f.transform_shape_fu(1)?;
        f.shape_until_valid();
        let c = test_curve(&self.pbase, self.dir)?;
        let sens = sens_of_prism(&c, &self.form.s_until)?;
        let the_prism = Prism::with_vector(&self.pbase, self.along((sens as f64) * length))?;
        self.until_or_support(&the_prism, trf, &c, sens, false)
    }
}
