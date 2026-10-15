// Rust translation of BRepFeat_MakePrism::Perform(From, Until) and PerformFromEnd of
// ModelingAlgorithms/TKFeat/BRepFeat/BRepFeat_MakePrism.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The prisms limited on both sides: between two shapes, and from the end of the basis
//! up to a shape.
use super::BRepFeatMakePrism;
use super::statics::{height_max, sens_of_prism, test_curve};
use crate::brep_feat::{PerfSelection, StatusError, tool};
use crate::errors::construction_error;
use crate::loc_ope::{CSIntersector, Prism};
use rustycad_tkbo::brep_algo_api::BRepAlgoAPICut;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{self, Face, Shape};
use rustycad_tkernel::ncollection::{List, Sequence};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::{GeomCurve, GeomRectangularTrimmedSurface};
use rustycad_tkg3d::top_abs::{self, Orientation, ShapeEnum};
use rustycad_tktopalgo::brep_lib::BRepLibMakeFace;

/// Standard_ConstructionError unless `s` has a face.
fn require_face(s: &Shape) -> Result<(), Failure> {
    if !Explorer::new(s, ShapeEnum::Face, ShapeEnum::Shape)?.more() {
        return Err(construction_error(
            "BRepFeat_MakePrism::Perform",
            "the limiting shape has no face",
        ));
    }
    Ok(())
}

/// The first intersection of the test curve with a limiting shape: its orientation,
/// face and parameter, the orientation taken at the last point when `last`.
fn first_hit(
    s: &Shape,
    scur: &Sequence<Handle<dyn GeomCurve>>,
    last: bool,
) -> Result<Option<(Orientation, Face, f64)>, Failure> {
    let mut asi = CSIntersector::with_shape(s);
    asi.perform_curves(scur)?;
    if !asi.is_done() || asi.nb_points(1)? < 1 {
        return Ok(None);
    }
    let k = if last { asi.nb_points(1)? } else { 1 };
    let or = asi.point(1, k)?.orientation();
    let p = asi.point(1, 1)?;
    Ok(Some((or, p.face().clone(), p.parameter())))
}

impl BRepFeatMakePrism {
    /// The prism cut by the solids of the limiting shapes `tools`, then fused with or cut
    /// from the basis.
    fn cut_by_tools(&mut self, vrai_prism: &Shape, l_tools: &List<Shape>) -> Result<(), Failure> {
        let mut l_obj = List::default();
        l_obj.append(vrai_prism.clone());
        let mut tr_p = BRepAlgoAPICut::new();
        tr_p.set_arguments(&l_obj);
        tr_p.set_tools(l_tools);
        tr_p.build()?;
        self.form
            .update_descendants_bop(&tr_p, tr_p.shape(), false)?;
        let cut = tr_p.shape().clone();
        self.form.finish_by_boolean(&cut)
    }

    /// The solids of the shape Until and of the shape From limited by their faces, or the
    /// status error of the null one.
    fn limit_solids(
        &self,
        until: (&Face, Orientation),
        from: (&Face, Orientation),
    ) -> Result<Result<List<Shape>, StatusError>, Failure> {
        let mut l_tools = List::default();
        let s = tool(&self.form.s_until, until.0, until.1)?;
        if s.is_null() {
            return Ok(Err(StatusError::NullToolU));
        }
        l_tools.append(s.into_shape());
        let ss = tool(&self.form.s_from, from.0, from.1)?;
        if ss.is_null() {
            return Ok(Err(StatusError::NullToolF));
        }
        l_tools.append(ss.into_shape());
        Ok(Ok(l_tools))
    }

    /// The prism of `Perform(From, Until)`: long and moved back when both shapes are on
    /// the same side (`tran < 0`).
    fn between_prism(&self, height: f64, sens: i32, tran: i32) -> Result<Prism, Failure> {
        let mut the_prism = Prism::new();
        if tran < 0 {
            let vtra = self.along(((-3. * height) * sens as f64) / 2.);
            the_prism.perform_translated(
                &self.pbase,
                self.along(((3 * sens) as f64) * height),
                vtra,
            )?;
        } else {
            the_prism.perform(&self.pbase, self.along(((2 * sens) as f64) * height))?;
        }
        Ok(the_prism)
    }

    /// The shapes From and Until set: whether both are extended, None (IncTypes) when
    /// only one is.
    fn set_limits(&mut self, from: &Shape, until: &Shape) -> Result<Option<bool>, Failure> {
        let f = &mut self.form;
        f.glued_f.clear(false);
        f.perf_selection = PerfSelection::SelectionFU;
        f.perf_selection_valid();
        require_face(from)?;
        require_face(until)?;
        f.s_from = from.clone();
        let trff = f.transform_shape_fu(0)?;
        f.shape_from_valid();
        f.s_until = until.clone();
        let trfu = f.transform_shape_fu(1)?;
        f.shape_until_valid();
        if trfu != trff {
            f.fail(StatusError::IncTypes);
            return Ok(None);
        }
        Ok(Some(trff))
    }

    /// The case starting or ending on the sketch face: the prism up to the other shape,
    /// true when its gluing was kept.
    fn on_sketch_face(&mut self, from: &Shape, until: &Shape) -> Result<bool, Failure> {
        if self.form.skface.is_null() {
            return Ok(false);
        }
        let other = if from.is_same(&self.form.skface) {
            until
        } else if until.is_same(&self.form.skface) {
            from
        } else {
            return Ok(false);
        };
        self.form.just_gluer = true;
        self.perform_until(other)?;
        Ok(self.form.just_gluer)
    }

    /// `Perform(From, Until)`: the prism between the shapes `from` and `until`;
    /// `prism.perform_from_until(&from, &until)?`.
    pub fn perform_from_until(&mut self, from: &Shape, until: &Shape) -> Result<(), Failure> {
        if from.is_null() || until.is_null() {
            return Err(construction_error(
                "BRepFeat_MakePrism::Perform",
                "null limiting shape",
            ));
        }
        if self.on_sketch_face(from, until)? {
            return Ok(());
        }
        let Some(trff) = self.set_limits(from, until)? else {
            return Ok(());
        };
        // length depending on bounding boxes
        let f = &self.form;
        let height = height_max(&f.sbase, &f.skface, &f.s_from, &f.s_until)?;
        let c = test_curve(&self.pbase, self.dir)?;
        // direction of prism and transfer of prism
        let (sens, tran) = if from.is_same(until) {
            (1, -1)
        } else {
            let sens = sens_of_prism(&c, &f.s_until)?;
            (sens, sens * sens_of_prism(&c, &f.s_from)?)
        };
        let the_prism = self.between_prism(height, sens, tran)?;
        if !trff {
            self.take_prism(&the_prism)?;
            self.form.glued_faces_valid();
            return self.global_with(&the_prism);
        }
        // case until support -> topological operation
        let vrai_prism = self.map_prism(&the_prism)?;
        self.between_supports(&vrai_prism, &c, sens, tran)
    }

    /// The prism between two supports: cut by the solids they limit, then fused with or
    /// cut from the basis.
    fn between_supports(
        &mut self,
        vrai_prism: &Shape,
        c: &Handle<dyn GeomCurve>,
        sens: i32,
        tran: i32,
    ) -> Result<(), Failure> {
        let mut scur = Sequence::default();
        scur.append(c.clone());
        let Some((mut or_u, f_until, par_u)) =
            first_hit(&self.form.s_until, &scur, !self.form.fuse)?
        else {
            self.form.fail(StatusError::NoIntersectU);
            return Ok(());
        };
        if sens == -1 {
            or_u = top_abs::reverse(or_u);
        }
        let Some((mut or_f, f_from, par_f)) = first_hit(&self.form.s_from, &scur, false)? else {
            self.form.fail(StatusError::NoIntersectF);
            return Ok(());
        };
        if sens == 1 {
            or_f = top_abs::reverse(or_f);
        }
        if tran > 0 && par_u.abs() < par_f.abs() {
            std::mem::swap(&mut or_u, &mut or_f);
        }
        match self.limit_solids((&f_until, or_u), (&f_from, or_f))? {
            Ok(l_tools) => self.cut_by_tools(vrai_prism, &l_tools),
            Err(e) => {
                self.form.fail(e);
                Ok(())
            }
        }
    }

    /// `PerformFromEnd(Until)`: the prism from the far end of the basis up to the shape
    /// `until`; `prism.perform_from_end(&face)?`.
    pub fn perform_from_end(&mut self, until: &Shape) -> Result<(), Failure> {
        if until.is_null() {
            return Err(construction_error(
                "BRepFeat_MakePrism::PerformFromEnd",
                "null limiting shape",
            ));
        }
        if !self.form.skface.is_null() && until.is_same(&self.form.skface) {
            self.dir.reverse();
            return self.perform_until_end();
        }
        require_face(until)?;
        let f = &mut self.form;
        f.perf_selection = PerfSelection::SelectionShU;
        f.perf_selection_valid();
        f.s_from.nullify();
        f.shape_from_valid();
        f.s_until = until.clone();
        let trf = f.transform_shape_fu(1)?;
        f.shape_until_valid();
        let c = test_curve(&self.pbase, self.dir)?;
        let sens = sens_of_prism(&c, &self.form.s_until)?;
        let f = &self.form;
        let height = height_max(&f.sbase, &f.skface, &f.s_from, &f.s_until)?;
        let vtra = self.along(((-3. * height) * sens as f64) / 2.);
        let vect = self.along(((3 * sens) as f64) * height);
        let the_prism = Prism::with_translation(&self.pbase, vect, vtra)?;
        if !trf {
            // case face until
            self.take_prism(&the_prism)?;
            self.form.glued_f.clear(false);
            self.form.glued_faces_valid();
            return self.global_with(&the_prism);
        }
        // case support
        let vrai_prism = self.map_prism(&the_prism)?;
        self.end_support(&vrai_prism, &c, sens)
    }

    /// The shape From of the support case: the untrimmed surface of the basis face met
    /// first by the test curve, extended; its face and orientation (forward and null
    /// without intersection).
    fn end_from_face(
        &mut self,
        scur: &Sequence<Handle<dyn GeomCurve>>,
    ) -> Result<(Orientation, Face), Failure> {
        let Some((or_f, f_from, _)) = first_hit(&self.form.sbase, scur, false)? else {
            return Ok((Orientation::Forward, Face::default()));
        };
        // if(sens==1) OrF = TopAbs::Reverse(OrF); (disabled natively)
        let mut s = BRepTool::surface(&f_from)?;
        let trimmed: Handle<GeomRectangularTrimmedSurface> = s.cast();
        if let Some(t) = trimmed.get() {
            s = t.basis_surface();
        }
        let fac = BRepLibMakeFace::from_surface(&s, precision::confusion())?;
        self.form.s_from = fac.face()?.into_shape();
        self.form.transform_shape_fu(0)?;
        Ok((or_f, topods::face(&self.form.s_from)?))
    }

    /// The prism of `PerformFromEnd` on a support: cut by the solids of the support and
    /// of the basis face met first, then fused with or cut from the basis.
    fn end_support(
        &mut self,
        vrai_prism: &Shape,
        c: &Handle<dyn GeomCurve>,
        sens: i32,
    ) -> Result<(), Failure> {
        let mut scur = Sequence::default();
        scur.append(c.clone());
        let (mut or_u, mut f_until) = (Orientation::Forward, Face::default());
        if let Some((or, face, _)) = first_hit(&self.form.s_until, &scur, false)? {
            or_u = if sens == -1 { top_abs::reverse(or) } else { or };
            f_until = face;
        }
        let (or_f, f_from) = self.end_from_face(&scur)?;
        match self.limit_solids((&f_until, or_u), (&f_from, or_f))? {
            Ok(l_tools) => self.cut_by_tools(vrai_prism, &l_tools),
            Err(e) => {
                self.form.fail(e);
                Ok(())
            }
        }
    }
}
