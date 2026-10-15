// Rust translation of BRepFeat_MakeDPrism::Perform(From, Until) and PerformFromEnd of
// ModelingAlgorithms/TKFeat/BRepFeat/BRepFeat_MakeDPrism.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The drafted prisms limited on both sides: between two shapes, and from the end of the
//! basis up to a shape.
use super::BRepFeatMakeDPrism;
use super::perform::{oriented_curve, require_face};
use super::statics::{height_max, sens_of_prism, test_curve};
use crate::brep_feat::{PerfSelection, StatusError, tool};
use crate::errors::construction_error;
use crate::loc_ope::{CSIntersector, DPrism};
use rustycad_tkbo::brep_algo_api::BRepAlgoAPICut;
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::topods::{Compound, Face, Shape};
use rustycad_tkernel::ncollection::Sequence;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::{GeomCurve, GeomRectangularTrimmedSurface};
use rustycad_tkg3d::top_abs::{self, Orientation};
use rustycad_tktopalgo::brep_lib::BRepLibMakeFace;

/// The intersector of `s` with the curve `c`.
fn intersect(s: &Shape, c: &Handle<dyn GeomCurve>) -> Result<CSIntersector, Failure> {
    let mut scur = Sequence::default();
    scur.append(c.clone());
    let mut asi = CSIntersector::with_shape(s);
    asi.perform_curves(&scur)?;
    Ok(asi)
}

/// Whether the intersector met its shape.
fn hit(asi: &CSIntersector) -> Result<bool, Failure> {
    Ok(asi.is_done() && asi.nb_points(1)? >= 1)
}

impl BRepFeatMakeDPrism {
    /// The compound of the solids of the shapes Until and From limited by their faces,
    /// or the status error of the null one.
    fn limit_compound(
        &self,
        until: (&Face, Orientation),
        from: (&Face, Orientation),
    ) -> Result<Result<Shape, StatusError>, Failure> {
        let b = BRepBuilder::default();
        let mut comp = Compound::default();
        b.make_compound(&mut comp);
        let mut comp = comp.into_shape();
        let s = tool(&self.form.s_until, until.0, until.1)?;
        if s.is_null() {
            return Ok(Err(StatusError::NullToolU));
        }
        b.add(&mut comp, &s)?;
        let ss = tool(&self.form.s_from, from.0, from.1)?;
        if ss.is_null() {
            return Ok(Err(StatusError::NullToolF));
        }
        b.add(&mut comp, &ss)?;
        Ok(Ok(comp))
    }

    /// The drafted prism cut by the solids of the limits, then fused with or cut from the
    /// basis (the history of that cut not kept, as natively).
    fn cut_by_limits(
        &mut self,
        vrai: &Shape,
        until: (&Face, Orientation),
        from: (&Face, Orientation),
    ) -> Result<(), Failure> {
        match self.limit_compound(until, from)? {
            Ok(comp) => {
                let tr_p = BRepAlgoAPICut::from_shapes(vrai, &comp)?;
                let cut = tr_p.shape().clone();
                self.form.finish_with(&cut, false)
            }
            Err(e) => {
                self.form.fail(e);
                Ok(())
            }
        }
    }

    /// The case starting or ending on the sketch face: the drafted prism up to the other
    /// shape, true when its gluing was kept.
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

    /// `Perform(From, Until)`: the drafted prism between the shapes `from` and `until`;
    /// `dprism.perform_from_until(&from, &until)?`.
    pub fn perform_from_until(&mut self, from: &Shape, until: &Shape) -> Result<(), Failure> {
        if from.is_null() || until.is_null() {
            return Err(construction_error(
                "BRepFeat_MakeDPrism::Perform",
                "null limiting shape",
            ));
        }
        if self.on_sketch_face(from, until)? {
            return Ok(());
        }
        let Some(trff) = self.set_limits(from, until)? else {
            return Ok(());
        };
        let c = test_curve(&self.pbase)?;
        let sens = if from.is_same(until) {
            1
        } else {
            sens_of_prism(&c, &self.form.s_until)?
        };
        let f = &self.form;
        let height = (sens as f64) * height_max(&f.sbase, &self.pbase, &f.s_from, &f.s_until)?;
        let the_dprism = DPrism::with_heights(&self.pbase, height, height, self.angle)?;
        if !trff {
            // Make systematically dprism
            self.take_dprism(&the_dprism)?;
            self.form.glued_faces_valid();
            // topologic reconstruction
            return self.global_with(&the_dprism);
        }
        let vrai = self.map_dprism(&the_dprism)?;
        self.between_supports(&vrai, &oriented_curve(&c, sens)?)
    }

    /// The drafted prism between two supports met by the test curve `c1`.
    fn between_supports(
        &mut self,
        vrai: &Shape,
        c1: &Handle<dyn GeomCurve>,
    ) -> Result<(), Failure> {
        let asi1 = intersect(&self.form.s_until, c1)?;
        let asi2 = intersect(&self.form.s_from, c1)?;
        // direction of dprism
        if !hit(&asi1)? {
            self.form.fail(StatusError::NoIntersectU);
            return Ok(());
        }
        let k = if self.form.fuse {
            1
        } else {
            asi1.nb_points(1)?
        };
        let or_u = asi1.point(1, k)?.orientation();
        let f_until = asi1.point(1, 1)?.face().clone();
        if !hit(&asi2)? {
            self.form.fail(StatusError::NoIntersectF);
            return Ok(());
        }
        let or_f = top_abs::reverse(asi2.point(1, 1)?.orientation());
        let f_from = asi2.point(1, 1)?.face().clone();
        self.cut_by_limits(vrai, (&f_until, or_u), (&f_from, or_f))
    }

    /// `PerformFromEnd(Until)`: the drafted prism from the far end of the basis up to the
    /// shape `until`; `dprism.perform_from_end(&face)?`.
    pub fn perform_from_end(&mut self, until: &Shape) -> Result<(), Failure> {
        if until.is_null() {
            return Err(construction_error(
                "BRepFeat_MakeDPrism::PerformFromEnd",
                "null limiting shape",
            ));
        }
        if !self.form.skface.is_null() && until.is_same(&self.form.skface) {
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
        let c = test_curve(&self.pbase)?;
        let sens = sens_of_prism(&c, &self.form.s_until)?;
        let f = &self.form;
        let height = (sens as f64) * height_max(&f.sbase, &f.skface, &f.s_from, &f.s_until)?;
        let the_dprism = DPrism::with_heights(&self.pbase, height, height, self.angle)?;
        if the_dprism.shape()?.is_null() {
            self.form.fail(StatusError::NullRealTool);
            return Ok(());
        }
        if !trf {
            // case finite face
            self.take_dprism(&the_dprism)?;
            self.form.glued_f.clear(false);
            self.form.glued_faces_valid();
            return self.global_with(&the_dprism);
        }
        // case support
        let vrai = self.map_dprism(&the_dprism)?;
        self.end_support(&vrai, &oriented_curve(&c, sens)?)
    }

    /// The first basis face met backward by the test curve: its face and orientation
    /// (reversed), its untrimmed surface extended into the shape From.
    fn end_from_face(
        &mut self,
        c2: &Handle<dyn GeomCurve>,
    ) -> Result<(Orientation, Face), Failure> {
        let asi2 = intersect(&self.form.sbase, c2)?;
        if !hit(&asi2)? {
            return Ok((Orientation::Forward, Face::default()));
        }
        let jj = asi2.nb_points(1)?;
        let mut prm = asi2.point(1, 1)?.parameter();
        let mut f_from = asi2.point(1, 1)?.face().clone();
        let mut or_f = top_abs::reverse(asi2.point(1, 1)?.orientation());
        for iii in 1..=jj {
            let p = asi2.point(1, iii)?;
            if p.parameter() < prm {
                prm = p.parameter();
                f_from = p.face().clone();
                or_f = top_abs::reverse(p.orientation());
            }
        }
        let mut s = BRepTool::surface(&f_from)?;
        let trimmed: Handle<GeomRectangularTrimmedSurface> = s.cast();
        if let Some(t) = trimmed.get() {
            s = t.basis_surface();
        }
        let fac = BRepLibMakeFace::from_surface(&s, precision::confusion())?;
        self.form.s_from = fac.face()?.into_shape();
        self.form.transform_shape_fu(0)?;
        // FFrom stays the face of the basis (natively `FFrom = TopoDS::Face(mySFrom)` is
        // disabled)
        Ok((or_f, f_from))
    }

    /// The drafted prism of `PerformFromEnd` on a support.
    fn end_support(&mut self, vrai: &Shape, c2: &Handle<dyn GeomCurve>) -> Result<(), Failure> {
        let asi1 = intersect(&self.form.s_until, c2)?;
        let (mut or_u, mut f_until) = (Orientation::Forward, Face::default());
        if hit(&asi1)? {
            let p = asi1.point(1, 1)?;
            or_u = p.orientation();
            if p.parameter() < 0. {
                or_u = top_abs::reverse(or_u);
            }
            f_until = p.face().clone();
        }
        let (or_f, f_from) = self.end_from_face(c2)?;
        self.cut_by_limits(vrai, (&f_until, or_u), (&f_from, or_f))
    }
}
