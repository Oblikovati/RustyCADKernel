// Rust translation of BRepFeat_MakeRevol::Perform(From, Until) of
// ModelingAlgorithms/TKFeat/BRepFeat/BRepFeat_MakeRevol.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The revolution between two shapes: a full turn cut by the solids of both limits, the
//! piece whose parametric barycentre lies between them kept.
use super::BRepFeatMakeRevol;
use super::perform::require_face;
use crate::brep_feat::{PerfSelection, StatusError, parametric_barycenter, tool};
use crate::errors::construction_error;
use crate::loc_ope::{CSIntersector, Revol};
use rustycad_tkbo::brep_algo_api::BRepAlgoAPICut;
use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{Compound, Shape};
use rustycad_tkernel::ncollection::Sequence;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::{self, ShapeEnum};
use rustycad_tkmath::elclib;
use std::f64::consts::PI;

impl BRepFeatMakeRevol {
    /// The case starting or ending on the sketch face: the revolution up to the other
    /// shape (the axis reversed when ending on it), true when its gluing was kept.
    fn on_sketch_face(&mut self, from: &Shape, until: &Shape) -> Result<bool, Failure> {
        if self.form.skface.is_null() {
            return Ok(false);
        }
        if from.is_same(&self.form.skface) {
            self.form.just_gluer = true;
            self.perform_until(until)?;
        } else if until.is_same(&self.form.skface) {
            self.form.just_gluer = true;
            self.axis.reverse();
            self.perform_until(from)?;
        } else {
            return Ok(false);
        }
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

    /// `Perform(From, Until)`: the revolution between the shapes `from` and `until`;
    /// `revol.perform_from_until(&from, &until)?`.
    pub fn perform_from_until(&mut self, from: &Shape, until: &Shape) -> Result<(), Failure> {
        if from.is_null() || until.is_null() {
            return Err(construction_error(
                "BRepFeat_MakeRevol::Perform",
                "null limiting shape",
            ));
        }
        if self.on_sketch_face(from, until)? {
            return Ok(());
        }
        let Some(trff) = self.set_limits(from, until)? else {
            return Ok(());
        };
        let the_revol = Revol::with_axis(&self.pbase, self.axis, 2. * PI)?;
        let vrai = self.map_revol(&the_revol)?;
        if !trff {
            self.set_generated(&vrai);
            self.form.glued_faces_valid();
            return self.global_with(&the_revol);
        }
        self.keep_curves(&the_revol)?;
        self.between_supports(&vrai)
    }

    /// The parameters on the circle of the barycentre where it enters Until (`pr_u`)
    /// and leaves From (`pr_f`, brought into the turn before `pr_u`), with the solids of
    /// both limits; or the status error.
    #[allow(clippy::neg_cmp_op_on_partial_ord)] // `!(PrU > PrF)` natively: NaN fails
    fn limit_solids(&self) -> Result<Result<(f64, f64, Shape), StatusError>, Failure> {
        let mut scur = Sequence::default();
        scur.append(self.b_curve.clone());
        let mut asi1 = CSIntersector::with_shape(&self.form.s_until);
        let mut asi2 = CSIntersector::with_shape(&self.form.s_from);
        asi1.perform_curves(&scur)?;
        asi2.perform_curves(&scur)?;
        if !asi1.is_done() || asi1.nb_points(1)? < 1 {
            return Ok(Err(StatusError::NoIntersectU));
        }
        let pu = asi1.point(1, 1)?;
        let (or_u, f_until, pr_u) = (pu.orientation(), pu.face().clone(), pu.parameter());
        if !asi2.is_done() || asi2.nb_points(1)? < 1 {
            return Ok(Err(StatusError::NoIntersectF));
        }
        let pr1 = elclib::in_period(asi2.point(1, 1)?.parameter(), pr_u - 2. * PI, pr_u);
        let pr2 = elclib::in_period(
            asi2.point(1, asi2.nb_points(1)?)?.parameter(),
            pr_u - 2. * PI,
            pr_u,
        );
        let or_f = top_abs::reverse(or_u);
        let f_from = asi2.point(1, 1)?.face().clone();
        let pr_f = pr1.max(pr2);
        if !(pr_u > pr_f) {
            return Ok(Err(StatusError::IncParameter));
        }
        let b = BRepBuilder::default();
        let mut comp = Compound::default();
        b.make_compound(&mut comp);
        let mut comp = comp.into_shape();
        for (s, face, or) in [
            (&self.form.s_from, &f_from, or_f),
            (&self.form.s_until, &f_until, or_u),
        ] {
            let sol = tool(s, face, or)?;
            if !sol.is_null() {
                b.add(&mut comp, &sol)?;
            }
        }
        Ok(Ok((pr_u, pr_f, comp)))
    }

    /// The revolution between two supports: the piece of the full turn cut by their
    /// solids whose parametric barycentre lies between the limits (else the first one).
    fn between_supports(&mut self, vrai: &Shape) -> Result<(), Failure> {
        let (pr_u, pr_f, comp) = match self.limit_solids()? {
            Ok(limits) => limits,
            Err(e) => {
                self.form.fail(e);
                return Ok(());
            }
        };
        let tr_p = BRepAlgoAPICut::from_shapes(vrai, &comp)?;
        let mut ex = Explorer::new(tr_p.shape(), ShapeEnum::Solid, ShapeEnum::Shape)?;
        let mut chosen = ex.current().clone();
        for sol in &mut ex {
            let pr_cur = parametric_barycenter(&sol, &self.b_curve)?;
            if pr_f <= pr_cur && pr_u >= pr_cur {
                chosen = sol;
                break;
            }
        }
        self.form.finish_with(&chosen, true)
    }
}
